//! Regression test for generate_deletion_proof on unregistered vs registered/revoked groups.

#![forbid(unsafe_code)]

use contextra_core::ContextraError;
use contextra_crypto::{crypto::KeyManager, kv_shredding::KeyRegistry};
use contextra_store::kv::{
    delete_mode::KvDeleteMode,
    segment::{KvSegmentConfig, KvSegmentManager},
};
use std::sync::Arc;

#[test]
fn test_unregistered_group_returns_error() {
    let kp = contextra_crypto::DeletionProofKeyPair::generate();
    let clock = Arc::new(contextra_ports::SystemClock::new());
    let registry = Arc::new(KeyRegistry::new_in_memory(
        clock,
        Some(kp.signing_key().clone()),
        kp.verifying_key,
    ));
    let config = KvSegmentConfig {
        delete_mode: KvDeleteMode::CryptoShred,
    };
    let manager = KvSegmentManager::new(config, registry, None);

    // Group 4242 was never registered.
    let result = manager.generate_deletion_proof(4242);
    assert!(
        matches!(result, Err(ContextraError::KvDeleteModeConfig(_))),
        "Expected Err(ContextraError::KvDeleteModeConfig) for unregistered group 4242, got: {result:?}"
    );
}

#[test]
fn test_registered_then_revoked_group_returns_ok_true() {
    let kp = contextra_crypto::DeletionProofKeyPair::generate();
    let clock = Arc::new(contextra_ports::SystemClock::new());
    let registry = Arc::new(KeyRegistry::new_in_memory(
        clock,
        Some(kp.signing_key().clone()),
        kp.verifying_key,
    ));
    let master_km = Arc::new(
        KeyManager::try_new("test-passphrase", b"test-salt").expect("Failed to create KeyManager"),
    );
    let config = KvSegmentConfig {
        delete_mode: KvDeleteMode::CryptoShred,
    };
    let manager = KvSegmentManager::new(config, Arc::clone(&registry), Some(master_km));

    // Register group 7 via write_segment.
    manager
        .write_segment(7, b"payload for group 7")
        .expect("write_segment failed");

    // Revoke group 7 through normal path (delete_segment).
    let revoked = manager.delete_segment(7);
    assert!(
        revoked,
        "delete_segment should return true for registered active key"
    );

    // Calling generate_deletion_proof for revoked group 7 should return Ok(true).
    let proof_result = manager.generate_deletion_proof(7);
    assert_eq!(
        proof_result.expect("generate_deletion_proof failed"),
        true,
        "Expected Ok(true) for revoked group 7"
    );
}
