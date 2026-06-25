//! AI response cache (S28-012).
//!
//! Caches AI analysis responses keyed by a SHA-256 hash of the system context
//! (CPU, memory, battery, fan data).  Identical system contexts within the
//! TTL window return the cached response instead of making a new HTTP request
//! to the AI provider, saving tokens and reducing cost.
//!
//! The cache is an in-memory `HashMap` guarded by a `Mutex`, initialised once
//! via `OnceLock`.  Entries expire after [`CACHE_TTL`] (5 minutes).  Because
//! the cache key is derived from the *current* hardware state, any significant
//! change in CPU / memory / battery / fan readings naturally produces a
//! different key, effectively invalidating the stale entry.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::util::panic::lock_or_recover;

/// How long a cached AI response remains valid.
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

/// Maximum number of entries retained in the cache.
///
/// Prevents unbounded memory growth if many distinct system contexts are seen.
const CACHE_MAX_ENTRIES: usize = 64;

/// A single cached AI response: the timestamp it was stored and the response text.
type CacheEntry = (Instant, String);

/// Global cache singleton — `OnceLock<Mutex<HashMap<key, entry>>>`.
fn cache() -> &'static Mutex<HashMap<String, CacheEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Compute the SHA-256 hash of the system context, returned as a hex string.
///
/// The key is derived from the *content* of the system context (CPU, memory,
/// battery, fan data), so any significant hardware state change produces a
/// different key, naturally invalidating the previous cached response.
pub fn cache_key(system_context: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(system_context.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// Look up a cached AI response for the given system context.
///
/// Returns `Some(response)` if a valid (non-expired) entry exists, or `None`
/// if there is no entry or the entry has exceeded the TTL.
///
/// Logs cache hits and misses at `debug` level for troubleshooting.
pub fn get(system_context: &str) -> Option<String> {
    let key = cache_key(system_context);
    let mut map = lock_or_recover(cache());
    if let Some((stored_at, response)) = map.get(&key) {
        if stored_at.elapsed() < CACHE_TTL {
            log::debug!("AI cache hit");
            return Some(response.clone());
        }
        // Entry expired — remove it.
        log::debug!("AI cache miss (expired)");
        map.remove(&key);
    } else {
        log::debug!("AI cache miss");
    }
    None
}

/// Store an AI response in the cache, keyed by the system context.
///
/// If the cache has reached [`CACHE_MAX_ENTRIES`], the oldest entry is evicted
/// before inserting the new one.
pub fn put(system_context: &str, response: &str) {
    let key = cache_key(system_context);
    let mut map = lock_or_recover(cache());

    // Evict expired entries first (cheap sweep).
    map.retain(|_, (stored_at, _)| stored_at.elapsed() < CACHE_TTL);

    // If still at capacity, remove the single oldest entry.
    if map.len() >= CACHE_MAX_ENTRIES {
        if let Some(oldest_key) = map
            .iter()
            .min_by_key(|(_, (stored_at, _))| *stored_at)
            .map(|(k, _)| k.clone())
        {
            map.remove(&oldest_key);
        }
    }

    map.insert(key, (Instant::now(), response.to_string()));
}

/// Clear all cached entries.
pub fn clear() {
    let mut map = lock_or_recover(cache());
    map.clear();
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    /// Serialise tests that mutate the global cache.
    static CACHE_LOCK: StdMutex<()> = StdMutex::new(());

    #[test]
    fn test_cache_key_deterministic() {
        let ctx = "cpu=50,memory=80,battery=100,fan=2000";
        assert_eq!(cache_key(ctx), cache_key(ctx));
    }

    #[test]
    fn test_cache_key_differs_for_different_context() {
        let a = cache_key("cpu=50,memory=80");
        let b = cache_key("cpu=90,memory=80");
        assert_ne!(a, b);
    }

    #[test]
    fn test_cache_miss_then_hit() {
        let _lock = CACHE_LOCK.lock().unwrap();
        clear();

        let ctx = "cpu=50,memory=80,battery=100,fan=2000";
        assert!(get(ctx).is_none(), "expected miss on empty cache");

        put(ctx, "analysis result");
        assert_eq!(get(ctx).as_deref(), Some("analysis result"));
    }

    #[test]
    fn test_cache_different_contexts() {
        let _lock = CACHE_LOCK.lock().unwrap();
        clear();

        let ctx_a = "cpu=50,memory=80";
        let ctx_b = "cpu=90,memory=80";

        put(ctx_a, "result-a");
        put(ctx_b, "result-b");

        assert_eq!(get(ctx_a).as_deref(), Some("result-a"));
        assert_eq!(get(ctx_b).as_deref(), Some("result-b"));
    }

    #[test]
    fn test_cache_overwrite() {
        let _lock = CACHE_LOCK.lock().unwrap();
        clear();

        let ctx = "cpu=50,memory=80";
        put(ctx, "first");
        put(ctx, "second");
        assert_eq!(get(ctx).as_deref(), Some("second"));
    }

    #[test]
    fn test_cache_clear() {
        let _lock = CACHE_LOCK.lock().unwrap();
        clear();

        let ctx = "cpu=50,memory=80";
        put(ctx, "result");
        assert!(get(ctx).is_some());
        clear();
        assert!(get(ctx).is_none());
    }

    #[test]
    fn test_cache_eviction_at_capacity() {
        let _lock = CACHE_LOCK.lock().unwrap();
        clear();

        // Insert CACHE_MAX_ENTRIES + 1 distinct contexts.
        for i in 0..(CACHE_MAX_ENTRIES + 1) {
            let ctx = format!("context-{i}");
            put(&ctx, &format!("result-{i}"));
        }

        // The very first entry should have been evicted.
        assert!(
            get("context-0").is_none(),
            "oldest entry should have been evicted"
        );
        // The most recent entry should still be present.
        let last = format!("context-{}", CACHE_MAX_ENTRIES);
        assert!(get(&last).is_some());
    }
}
