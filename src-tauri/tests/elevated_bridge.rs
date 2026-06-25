//! Integration tests for the elevated bridge HMAC authentication protocol.
//!
//! These tests verify the full round-trip of HMAC signing and verification
//! used by the elevated bridge protocol. The `auth` module functions are
//! all `pub`, so they can be called directly from integration tests.
//!
//! The crate library exports `pub mod util` which contains `pub mod auth`.

use micontrol_lib::util::auth;

// ── HMAC compute / verify round-trips ───────────────────────────────────────

#[test]
fn test_hmac_round_trip() {
    let key = b"test-key-32-bytes-long-1234567890";
    let data = b"hello world";
    let tag = auth::compute_hmac(key, data).expect("HMAC should succeed");
    assert!(auth::verify_hmac(key, data, &tag));
}

#[test]
fn test_hmac_rejection_with_wrong_key() {
    let key1 = b"test-key-32-bytes-long-1234567890";
    let key2 = b"different-key-32-bytes-long-123456";
    let data = b"hello world";
    let tag = auth::compute_hmac(key1, data).expect("HMAC should succeed");
    assert!(!auth::verify_hmac(key2, data, &tag));
}

#[test]
fn test_hmac_rejection_with_tampered_data() {
    let key = b"test-key-32-bytes-long-1234567890";
    let data = b"hello world";
    let tag = auth::compute_hmac(key, data).expect("HMAC should succeed");
    assert!(!auth::verify_hmac(key, b"hello worle", &tag));
}

#[test]
fn test_hmac_rejection_with_empty_tag() {
    let key = b"test-key-32-bytes-long-1234567890";
    let data = b"hello world";
    assert!(!auth::verify_hmac(key, data, ""));
}

#[test]
fn test_hmac_rejection_with_truncated_tag() {
    let key = b"test-key-32-bytes-long-1234567890";
    let data = b"hello world";
    let tag = auth::compute_hmac(key, data).expect("HMAC should succeed");
    // Truncate the first character
    assert!(!auth::verify_hmac(key, data, &tag[1..]));
}

// ── Nonce uniqueness ────────────────────────────────────────────────────────

#[test]
fn test_nonce_uniqueness() {
    let n1 = auth::generate_nonce();
    let n2 = auth::generate_nonce();
    assert_ne!(n1, n2, "Nonces should be unique");
}

#[test]
fn test_nonce_length() {
    let nonce = auth::generate_nonce();
    assert_eq!(nonce.len(), 32, "16 bytes = 32 hex characters");
}

#[test]
fn test_nonce_is_hex() {
    let nonce = auth::generate_nonce();
    assert!(
        nonce.chars().all(|c| c.is_ascii_hexdigit()),
        "Nonce should be a hex string"
    );
}

// ── Timestamp freshness ────────────────────────────────────────────────────

#[test]
fn test_timestamp_fresh_now() {
    let now = auth::now_ms();
    assert!(auth::is_timestamp_fresh(now));
}

#[test]
fn test_timestamp_stale_rejected() {
    let old = auth::now_ms() - auth::MAX_COMMAND_AGE_MS - 1000;
    assert!(!auth::is_timestamp_fresh(old));
}

#[test]
fn test_timestamp_future_rejected() {
    let future = auth::now_ms() + auth::MAX_COMMAND_AGE_MS + 1000;
    assert!(!auth::is_timestamp_fresh(future));
}

#[test]
fn test_timestamp_fresh_at_boundary() {
    // Exactly at the boundary (MAX_COMMAND_AGE_MS ago) — should be accepted
    let boundary = auth::now_ms() - auth::MAX_COMMAND_AGE_MS;
    assert!(auth::is_timestamp_fresh(boundary));
}

// ── Full payload sign / verify ─────────────────────────────────────────────

#[test]
fn test_sign_and_verify_payload_round_trip() {
    let key = b"test-key-32-bytes-long-1234567890";
    let mut payload = serde_json::json!({
        "cmd": "set_brightness",
        "args": {"level": 80},
        "created_at_ms": auth::now_ms(),
        "nonce": auth::generate_nonce(),
    });
    auth::sign_payload(&mut payload, key);
    assert!(
        payload.get("hmac").is_some(),
        "Payload should have hmac after signing"
    );

    let mut payload2 = payload.clone();
    let result = auth::verify_payload(&mut payload2, key);
    assert!(result.is_ok(), "Valid signed payload should verify");
}

#[test]
fn test_payload_missing_hmac_rejected() {
    let key = b"test-key-32-bytes-long-1234567890";
    let mut payload = serde_json::json!({
        "cmd": "set_brightness",
        "args": {"level": 80},
        "created_at_ms": auth::now_ms(),
        "nonce": auth::generate_nonce(),
    });
    // No sign_payload → no hmac field
    let result = auth::verify_payload(&mut payload, key);
    assert!(result.is_err(), "Missing hmac should be rejected");
    let err = result.unwrap_err();
    assert!(
        err.to_lowercase().contains("hmac"),
        "Error message should mention HMAC"
    );
}

#[test]
fn test_payload_with_wrong_key_rejected() {
    let key1 = b"test-key-32-bytes-long-1234567890";
    let key2 = b"attacker-key-32-bytes-long-1234567";
    let mut payload = serde_json::json!({
        "cmd": "set_brightness",
        "args": {"level": 80},
        "created_at_ms": auth::now_ms(),
        "nonce": auth::generate_nonce(),
    });
    auth::sign_payload(&mut payload, key1);
    let result = auth::verify_payload(&mut payload, key2);
    assert!(result.is_err(), "Wrong key should be rejected");
}

#[test]
fn test_payload_with_tampered_command_rejected() {
    let key = b"test-key-32-bytes-long-1234567890";
    let mut payload = serde_json::json!({
        "cmd": "set_brightness",
        "args": {"level": 80},
        "created_at_ms": auth::now_ms(),
        "nonce": auth::generate_nonce(),
    });
    auth::sign_payload(&mut payload, key);

    // Tamper with the command after signing (simulates file swap attack)
    payload["cmd"] = serde_json::json!("set_charging_threshold");
    payload["args"] = serde_json::json!({"threshold": 100});

    // Re-verify after tampering
    let result = auth::verify_payload(&mut payload, key);
    assert!(
        result.is_err(),
        "Tampered command payload should be rejected"
    );
}

#[test]
fn test_payload_with_stale_timestamp_rejected() {
    let key = b"test-key-32-bytes-long-1234567890";
    let old_ts = auth::now_ms() - auth::MAX_COMMAND_AGE_MS - 5000;
    let mut payload = serde_json::json!({
        "cmd": "set_brightness",
        "args": {"level": 80},
        "created_at_ms": old_ts,
        "nonce": auth::generate_nonce(),
    });
    auth::sign_payload(&mut payload, key);
    let result = auth::verify_payload(&mut payload, key);
    assert!(
        result.is_err(),
        "Payload with stale timestamp should be rejected"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_lowercase().contains("stale") || err.to_lowercase().contains("timestamp"),
        "Error should mention stale or timestamp"
    );
}

#[test]
fn test_payload_non_object_rejected() {
    let key = b"test-key-32-bytes-long-1234567890";
    let mut payload = serde_json::json!("not_an_object");
    // verify_payload expects an object — a bare string should be rejected.
    let result = auth::verify_payload(&mut payload, key);
    assert!(result.is_err(), "Non-object payload should be rejected");
}

// ── Edge cases ─────────────────────────────────────────────────────────────

#[test]
fn test_hmac_with_empty_key() {
    let data = b"some data";
    let tag = auth::compute_hmac(b"", data).expect("HMAC should succeed");
    assert!(auth::verify_hmac(b"", data, &tag));
}

#[test]
fn test_hmac_with_empty_data() {
    let key = b"test-key-32-bytes-long-1234567890";
    let tag = auth::compute_hmac(key, b"").expect("HMAC should succeed");
    assert!(auth::verify_hmac(key, b"", &tag));
}

#[test]
fn test_hmac_deterministic() {
    let key = b"test-key-32-bytes-long-1234567890";
    let data = b"deterministic test";
    let tag1 = auth::compute_hmac(key, data).expect("HMAC should succeed");
    let tag2 = auth::compute_hmac(key, data).expect("HMAC should succeed");
    assert_eq!(tag1, tag2, "HMAC should be deterministic");
}

#[test]
fn test_now_ms_monotonic() {
    let t1 = auth::now_ms();
    let t2 = auth::now_ms();
    assert!(
        t2 >= t1,
        "now_ms should be monotonic (t2={}, t1={})",
        t2,
        t1
    );
}

#[test]
fn test_max_command_age_constant() {
    assert_eq!(auth::MAX_COMMAND_AGE_MS, 30_000);
}
