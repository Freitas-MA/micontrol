//! AI usage tracking — local-only tracking of AI requests and token usage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::util::panic::lock_or_recover;

/// AI usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AiUsageStats {
    pub total_requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub estimated_cost_usd: f64,
    /// Number of AI analyses performed today.
    pub today_count: u64,
    /// Day number (days since Unix epoch) when `today_count` was last reset.
    pub last_reset_day: u64,
    /// Per-model request count (model name → number of requests).
    /// `#[serde(default)]` ensures backward compatibility with old JSON files
    /// that don't have this field — they default to an empty HashMap.
    #[serde(default)]
    pub model_usage: HashMap<String, u64>,
}

/// Cost per 1M tokens (example rates — adjust based on actual provider).
const COST_PER_1M_INPUT_TOKENS: f64 = 0.10;
const COST_PER_1M_OUTPUT_TOKENS: f64 = 0.30;

static USAGE: Mutex<Option<AiUsageStats>> = Mutex::new(None);

/// Returns the current day number (days since Unix epoch).
fn today_day_number() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0)
}

/// Reset `today_count` if a new day has started.
fn maybe_reset_daily(stats: &mut AiUsageStats) {
    let today = today_day_number();
    if stats.last_reset_day != today {
        stats.today_count = 0;
        stats.last_reset_day = today;
    }
}

/// Returns the path to the AI usage file: `%LOCALAPPDATA%\MiControl\ai_usage.json`.
fn usage_file_path() -> Option<std::path::PathBuf> {
    let local_appdata = std::env::var("LOCALAPPDATA").ok()?;
    let dir = std::path::PathBuf::from(local_appdata).join("MiControl");
    Some(dir.join("ai_usage.json"))
}

impl AiUsageStats {
    /// Save usage statistics to `%LOCALAPPDATA%\MiControl\ai_usage.json`.
    pub fn save_to_file(&self) {
        let Some(path) = usage_file_path() else {
            log::warn!("LOCALAPPDATA not set, cannot save AI usage stats");
            return;
        };
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                log::warn!("Failed to create AI usage dir: {e}");
                return;
            }
        }
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    log::warn!("Failed to write AI usage file: {e}");
                } else {
                    // S26-002: Restrict ACL on ai_usage.json — contains usage/cost data.
                    if let Err(e) = crate::util::auth::restrict_file_acl(&path) {
                        log::warn!("Failed to restrict ACL on ai_usage.json: {e}");
                    }
                }
            }
            Err(e) => log::warn!("Failed to serialize AI usage stats: {e}"),
        }
    }

    /// Load usage statistics from `%LOCALAPPDATA%\MiControl\ai_usage.json`.
    /// Returns `Default` if the file doesn't exist or can't be parsed.
    pub fn load_from_file() -> AiUsageStats {
        let Some(path) = usage_file_path() else {
            return AiUsageStats::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => AiUsageStats::default(),
        }
    }
}

/// Load persisted usage stats into the global static on startup (S24-016).
pub fn load_on_startup() {
    let mut usage = lock_or_recover(&USAGE);
    let mut stats = AiUsageStats::load_from_file();
    maybe_reset_daily(&mut stats);
    *usage = Some(stats);
}

/// Check if the daily analysis limit has been reached (S24-016).
///
/// Returns `Ok(())` if under the limit, or an error message if exceeded.
/// A `daily_limit` of 0 means unlimited.
pub fn check_daily_limit(daily_limit: u64) -> Result<(), String> {
    let mut usage = lock_or_recover(&USAGE);
    let stats = usage.get_or_insert_with(|| {
        let mut s = AiUsageStats::load_from_file();
        maybe_reset_daily(&mut s);
        s
    });
    maybe_reset_daily(stats);
    if daily_limit > 0 && stats.today_count >= daily_limit {
        return Err(format!(
            "Daily AI analysis limit reached ({}/{}). Please try again tomorrow.",
            stats.today_count, daily_limit
        ));
    }
    Ok(())
}

/// Record an AI request's token usage.
///
/// `model` is the model name used for this request (e.g. `"gpt-4o"`).
pub fn record_usage(model: &str, input_tokens: u64, output_tokens: u64) {
    let mut usage = lock_or_recover(&USAGE);
    let stats = usage.get_or_insert_with(AiUsageStats::default);
    maybe_reset_daily(stats);
    stats.total_requests += 1;
    stats.today_count += 1;
    stats.total_input_tokens += input_tokens;
    stats.total_output_tokens += output_tokens;
    stats.estimated_cost_usd += (input_tokens as f64 / 1_000_000.0) * COST_PER_1M_INPUT_TOKENS
        + (output_tokens as f64 / 1_000_000.0) * COST_PER_1M_OUTPUT_TOKENS;
    *stats.model_usage.entry(model.to_string()).or_insert(0) += 1;
    #[cfg(not(test))]
    stats.save_to_file();
}

/// Get current usage statistics.
pub fn get_usage() -> AiUsageStats {
    let mut usage = lock_or_recover(&USAGE);
    let stats = usage.get_or_insert_with(|| {
        let mut s = AiUsageStats::load_from_file();
        maybe_reset_daily(&mut s);
        s
    });
    maybe_reset_daily(stats);
    stats.clone()
}

/// Reset usage statistics.
pub fn reset_usage() {
    let mut usage = lock_or_recover(&USAGE);
    let stats = AiUsageStats::default();
    #[cfg(not(test))]
    stats.save_to_file();
    *usage = Some(stats);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serialize tests that modify the global USAGE counter.
    static USAGE_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_record_usage_increments_counters() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        record_usage("test-model", 100, 200);
        let stats = get_usage();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.total_input_tokens, 100);
        assert_eq!(stats.total_output_tokens, 200);
        assert_eq!(stats.model_usage.get("test-model"), Some(&1));
    }

    #[test]
    fn test_get_usage_returns_correct_stats() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        record_usage("model-a", 50, 60);
        record_usage("model-b", 70, 80);
        let stats = get_usage();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.total_input_tokens, 120);
        assert_eq!(stats.total_output_tokens, 140);
        assert_eq!(stats.model_usage.get("model-a"), Some(&1));
        assert_eq!(stats.model_usage.get("model-b"), Some(&1));
        // Cost: (120/1M * 0.10) + (140/1M * 0.30)
        let expected_cost = (120.0 / 1_000_000.0) * COST_PER_1M_INPUT_TOKENS
            + (140.0 / 1_000_000.0) * COST_PER_1M_OUTPUT_TOKENS;
        assert!((stats.estimated_cost_usd - expected_cost).abs() < 1e-10);
    }

    #[test]
    fn test_reset_usage_clears_counters() {
        let _lock = USAGE_LOCK.lock().unwrap();
        record_usage("test-model", 100, 200);
        record_usage("test-model", 300, 400);
        reset_usage();
        let stats = get_usage();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.total_input_tokens, 0);
        assert_eq!(stats.total_output_tokens, 0);
        assert_eq!(stats.estimated_cost_usd, 0.0);
        assert!(stats.model_usage.is_empty());
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();

        let handles: Vec<_> = (0..4)
            .map(|_| {
                thread::spawn(|| {
                    for _ in 0..100 {
                        record_usage("concurrent-model", 10, 20);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let stats = get_usage();
        assert_eq!(stats.total_requests, 400);
        assert_eq!(stats.total_input_tokens, 4000);
        assert_eq!(stats.total_output_tokens, 8000);
    }

    #[test]
    fn test_check_daily_limit_under_limit() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        assert!(check_daily_limit(5).is_ok());
        record_usage("test-model", 10, 20);
        record_usage("test-model", 10, 20);
        assert!(check_daily_limit(5).is_ok());
    }

    #[test]
    fn test_check_daily_limit_at_limit() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        for _ in 0..5 {
            record_usage("test-model", 10, 20);
        }
        assert!(check_daily_limit(5).is_err());
    }

    #[test]
    fn test_check_daily_limit_zero_means_unlimited() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        record_usage("test-model", 10, 20);
        assert!(check_daily_limit(0).is_ok());
    }

    #[test]
    fn test_daily_limit_error_message() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        record_usage("test-model", 10, 20);
        let err = check_daily_limit(1).unwrap_err();
        assert!(err.contains("Daily AI analysis limit reached"));
        assert!(err.contains("1/1"));
    }

    #[test]
    fn test_today_count_increments_with_record_usage() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        assert_eq!(get_usage().today_count, 0);
        record_usage("test-model", 10, 20);
        assert_eq!(get_usage().today_count, 1);
        record_usage("test-model", 10, 20);
        assert_eq!(get_usage().today_count, 2);
    }

    #[test]
    fn test_save_and_load_from_file() {
        let _lock = USAGE_LOCK.lock().unwrap();
        let orig = std::env::var("LOCALAPPDATA").ok();
        let tmp = std::env::temp_dir().join("micontrol_test_ai_usage_save_load");
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("LOCALAPPDATA", &tmp);

        reset_usage();
        record_usage("save-load-model", 100, 200);
        let stats = get_usage();
        stats.save_to_file();

        let loaded = AiUsageStats::load_from_file();
        assert_eq!(loaded.total_requests, 1);
        assert_eq!(loaded.today_count, 1);
        assert_eq!(loaded.total_input_tokens, 100);
        assert_eq!(loaded.total_output_tokens, 200);
        assert_eq!(loaded.model_usage.get("save-load-model"), Some(&1));

        match orig {
            Some(v) => std::env::set_var("LOCALAPPDATA", v),
            None => std::env::remove_var("LOCALAPPDATA"),
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_backward_compat_missing_model_usage() {
        // Simulate an old JSON file that doesn't have the `model_usage` field.
        let old_json = r#"{
            "total_requests": 5,
            "total_input_tokens": 1000,
            "total_output_tokens": 2000,
            "estimated_cost_usd": 0.001,
            "today_count": 3,
            "last_reset_day": 20000
        }"#;
        let stats: AiUsageStats = serde_json::from_str(old_json).unwrap();
        assert_eq!(stats.total_requests, 5);
        assert!(stats.model_usage.is_empty());
    }

    #[test]
    fn test_model_usage_tracks_multiple_models() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        record_usage("gpt-4o", 100, 200);
        record_usage("gpt-4o", 50, 60);
        record_usage("gpt-4o-mini", 30, 40);
        let stats = get_usage();
        assert_eq!(stats.model_usage.get("gpt-4o"), Some(&2));
        assert_eq!(stats.model_usage.get("gpt-4o-mini"), Some(&1));
        assert_eq!(stats.total_requests, 3);
    }
}
