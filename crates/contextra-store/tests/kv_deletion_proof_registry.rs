//! Integration tests for KvSegmentManager::generate_deletion_proof in contextra-store.

#![forbid(unsafe_code)]

use contextra_core::ContextraError;
use contextra_crypto::{crypto::KeyManager, kv_shredding::KeyRegistry};
use contextra_store::kv::{
    delete_mode::KvDeleteMode,
    segment::{KvSegmentConfig, KvSegmentManager},
};
use std::sync::Arc;

#[test]
fn test_generate_deletion_proof_behavior() {
    let master_km = Arc::new(
        KeyManager::try_new("master-passphrase", b"master-salt")
            .expect("Failed to create KeyManager"),
    );
    let registry = Arc::new(KeyRegistry::new());
    let config = KvSegmentConfig {
        delete_mode: KvDeleteMode::CryptoShred,
    };
    let manager = KvSegmentManager::new(config, registry, Some(master_km));

    let unknown_group_id = 99999;

    // 1. Never-registered group_id MUST return Err(ContextraError::KvDeleteModeConfig)
    let proof_res = manager.generate_deletion_proof(unknown_group_id);
    match proof_res {
        Err(ContextraError::KvDeleteModeConfig(msg)) => {
            assert!(
                msg.contains("nie registriert"),
                "Expected message indicating group was never registered, got: {msg}"
            );
        }
        res => panic!("Expected KvDeleteModeConfig error for unknown group_id, got: {res:?}"),
    }

    // 2. Active group_id created via write_segment MUST return Ok(false)
    let active_group_id = 42;
    manager
        .write_segment(active_group_id, b"sensitive segment payload")
        .expect("write_segment failed");

    let proof_active = manager
        .generate_deletion_proof(active_group_id)
        .expect("generate_deletion_proof failed for active group");
    assert!(
        !proof_active,
        "Active group_id must return Ok(false) for deletion proof"
    );

    // 3. Revoked group_id after delete_segment MUST return Ok(true)
    let deleted = manager.delete_segment(active_group_id);
    assert!(deleted, "delete_segment must return true");

    let proof_deleted = manager
        .generate_deletion_proof(active_group_id)
        .expect("generate_deletion_proof failed for deleted group");
    assert!(
        proof_deleted,
        "Deleted group_id must return Ok(true) for deletion proof"
    );
}

#[test]
fn test_tombstone_only_mode_returns_kv_delete_mode_config_error() {
    let registry = Arc::new(KeyRegistry::new());
    let config = KvSegmentConfig {
        delete_mode: KvDeleteMode::TombstoneOnly,
    };
    let manager = KvSegmentManager::new(config, registry, None);

    let res = manager.generate_deletion_proof(100);
    assert!(
        matches!(res, Err(ContextraError::KvDeleteModeConfig(_))),
        "TombstoneOnly mode MUST return KvDeleteModeConfig error"
    );
}
