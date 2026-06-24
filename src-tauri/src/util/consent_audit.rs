//! Audit log for consent grant/revoke events (GDPR Art.30).

use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

/// The current privacy policy version. Bump this when the privacy policy changes.
pub const POLICY_VERSION: u32 = 2;

/// Build the path to the audit log file (%LOCALAPPDATA%\MiControl\consent_audit.log).
fn audit_log_path() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
        let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into());
        format!("{}\\AppData\\Local", home)
    });
    PathBuf::from(base)
        .join("MiControl")
        .join("consent_audit.log")
}

/// Format the current time as a Unix epoch timestamp (seconds).
fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Log a consent event to the audit log.
pub fn log_consent_event(event: &str, policy_version: u32) {
    let path = audit_log_path();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let ts = unix_timestamp();
    let entry = format!("{ts}\t{event}\tpolicy_version={policy_version}\n");

    // Append to the audit log
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(entry.as_bytes()) {
                log::error!("Failed to write consent audit log: {e}");
            }
        }
        Err(e) => {
            log::error!("Failed to open consent audit log: {e}");
        }
    }
}

#[allow(dead_code)]
/// Log that consent was granted.
pub fn log_consent_granted(policy_version: u32) {
    log_consent_event("CONSENT_GRANTED", policy_version);
}

#[allow(dead_code)]
/// Log that consent was revoked.
pub fn log_consent_revoked(policy_version: u32) {
    log_consent_event("CONSENT_REVOKED", policy_version);
}

/// Read the audit log entries.
pub fn read_audit_log() -> Vec<String> {
    match std::fs::read_to_string(audit_log_path()) {
        Ok(content) => content.lines().map(|l| l.to_string()).collect(),
        Err(_) => Vec::new(),
    }
}
