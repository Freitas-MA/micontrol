//! AI usage tracking — local-only tracking of AI requests and token usage.

use serde::{Deserialize, Serialize};
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
pub fn record_usage(input_tokens: u64, output_tokens: u64) {
    let mut usage = lock_or_recover(&USAGE);
    let stats = usage.get_or_insert_with(AiUsageStats::default);
    maybe_reset_daily(stats);
    stats.total_requests += 1;
    stats.today_count += 1;
    stats.total_input_tokens += input_tokens;
    stats.total_output_tokens += output_tokens;
    stats.estimated_cost_usd += (input_tokens as f64 / 1_000_000.0) * COST_PER_1M_INPUT_TOKENS
        + (output_tokens as f64 / 1_000_000.0) * COST_PER_1M_OUTPUT_TOKENS;
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
        record_usage(100, 200);
        let stats = get_usage();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.total_input_tokens, 100);
        assert_eq!(stats.total_output_tokens, 200);
    }

    #[test]
    fn test_get_usage_returns_correct_stats() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        record_usage(50, 60);
        record_usage(70, 80);
        let stats = get_usage();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.total_input_tokens, 120);
        assert_eq!(stats.total_output_tokens, 140);
        // Cost: (120/1M * 0.10) + (140/1M * 0.30)
        let expected_cost = (120.0 / 1_000_000.0) * COST_PER_1M_INPUT_TOKENS
            + (140.0 / 1_000_000.0) * COST_PER_1M_OUTPUT_TOKENS;
        assert!((stats.estimated_cost_usd - expected_cost).abs() < 1e-10);
    }

    #[test]
    fn test_reset_usage_clears_counters() {
        let _lock = USAGE_LOCK.lock().unwrap();
        record_usage(100, 200);
        record_usage(300, 400);
        reset_usage();
        let stats = get_usage();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.total_input_tokens, 0);
        assert_eq!(stats.total_output_tokens, 0);
        assert_eq!(stats.estimated_cost_usd, 0.0);
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
                        record_usage(10, 20);
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
        record_usage(10, 20);
        record_usage(10, 20);
        assert!(check_daily_limit(5).is_ok());
    }

    #[test]
    fn test_check_daily_limit_at_limit() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        for _ in 0..5 {
            record_usage(10, 20);
        }
        assert!(check_daily_limit(5).is_err());
    }

    #[test]
    fn test_check_daily_limit_zero_means_unlimited() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        record_usage(10, 20);
        assert!(check_daily_limit(0).is_ok());
    }

    #[test]
    fn test_daily_limit_error_message() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        record_usage(10, 20);
        let err = check_daily_limit(1).unwrap_err();
        assert!(err.contains("Daily AI analysis limit reached"));
        assert!(err.contains("1/1"));
    }

    #[test]
    fn test_today_count_increments_with_record_usage() {
        let _lock = USAGE_LOCK.lock().unwrap();
        reset_usage();
        assert_eq!(get_usage().today_count, 0);
        record_usage(10, 20);
        assert_eq!(get_usage().today_count, 1);
        record_usage(10, 20);
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
        record_usage(100, 200);
        let stats = get_usage();
        stats.save_to_file();

        let loaded = AiUsageStats::load_from_file();
        assert_eq!(loaded.total_requests, 1);
        assert_eq!(loaded.today_count, 1);
        assert_eq!(loaded.total_input_tokens, 100);
        assert_eq!(loaded.total_output_tokens, 200);

        match orig {
            Some(v) => std::env::set_var("LOCALAPPDATA", v),
            None => std::env::remove_var("LOCALAPPDATA"),
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
