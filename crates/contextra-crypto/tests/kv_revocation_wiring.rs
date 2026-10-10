//! Integration tests for KvSegmentCipher and RevocationLog wiring in contextra-crypto.

#![forbid(unsafe_code)]

use contextra_crypto::{
    crypto::KeyManager,
    kv_cipher::KvSegmentCipher,
    kv_shredding::KeyRegistry,
    revocation_log::{RevocationLog, RevocationTarget},
};
use contextra_ports::SystemClock;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_kv_segment_cipher_with_revocation_log_persistence() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let log_path = temp_dir.path().join("revocation.log");

    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    let clock = Arc::new(SystemClock::new());

    let log = RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk), vk)
        .expect("Failed to create revocation log");
    let log_arc = Arc::new(log);

    let master_km = KeyManager::try_new("master-passphrase", b"master-salt")
        .expect("Failed to create KeyManager");
    let cipher = KvSegmentCipher::new(master_km, log_arc.clone());

    let group_id = 999;

    // Revoke group_id via cipher registry
    let revoked = cipher
        .registry()
        .revoke_group(group_id)
        .expect("revoke_group failed");
    assert!(revoked, "Group 999 must be revoked");

    // Close log by dropping references and re-open from disk
    drop(cipher);
    drop(log_arc);

    let reopened_log = RevocationLog::open_or_create(&log_path, clock, None, vk)
        .expect("Failed to reopen revocation log");

    assert!(
        reopened_log.is_revoked(&RevocationTarget::Group(group_id)),
        "Reopened log must contain revoked group 999"
    );
    assert!(
        reopened_log.verify_integrity().is_ok(),
        "Reopened log chain integrity must be valid"
    );
}

#[test]
fn test_unknown_group_id_remains_unregistered_after_revoke() {
    let clock = Arc::new(contextra_ports::SystemClock::new());
    let sk = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
    let vk = sk.verifying_key();
    let registry = KeyRegistry::new_in_memory(clock, Some(sk), vk);
    let unknown_group_id = 12345;

    assert!(
        !registry.was_group_ever_registered(unknown_group_id),
        "Unknown group must start as unregistered"
    );

    let revoked = registry
        .revoke_group(unknown_group_id)
        .expect("revoke_group on unknown group failed");

    assert!(
        revoked,
        "revoke_group returns true as it logs/marks revoked"
    );
    assert!(
        !registry.was_group_ever_registered(unknown_group_id),
        "Unknown group MUST NOT be marked as ever registered after revoke_group"
    );
}
