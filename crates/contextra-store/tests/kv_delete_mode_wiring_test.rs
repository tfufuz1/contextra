// FILE-CONTEXT
// ZWECK: Tests for KV Delete Mode & Crypto-Shredding wiring in contextra-store.
// INVARIANTEN:
// - INV-KV-DELETE-1: Löschbeweis auf Key-Value-Seite ist ausschließlich für CryptoShred-Segmente möglich, niemals für TombstoneOnly.

use contextra_core::ContextraError;
use contextra_crypto::{crypto::KeyManager, kv_shredding::KeyRegistry};
use contextra_store::kv::{KvDeleteMode, KvSegmentConfig, KvSegmentManager};
use std::sync::Arc;

#[test]
fn test_crypto_shred_revoke_subkey_read_fails() {
    let master_key = Arc::new(
        KeyManager::try_new("test-passphrase-shred", b"salt-1234")
            .expect("master key initialization"),
    );
    let registry = Arc::new(KeyRegistry::new());

    let config = KvSegmentConfig {
        delete_mode: KvDeleteMode::CryptoShred,
    };
    let manager = KvSegmentManager::new(config, Arc::clone(&registry), Some(master_key));

    let group_id = 1001;
    let plaintext = b"Confidential Payload Data";

    // Write encrypted segment
    let segment = manager
        .write_segment(group_id, plaintext)
        .expect("write encrypted segment");
    assert!(segment.is_encrypted);
    assert_ne!(segment.payload.as_slice(), plaintext);

    // Verify key is active in registry
    assert!(registry.is_key_active(group_id));

    // Decryption works initially
    let read_back = manager
        .read_segment(&segment)
        .expect("read before revocation");
    assert_eq!(read_back.as_slice(), plaintext);

    // Revoke subkey directly or via manager
    let revoked = manager.delete_segment(group_id);
    assert!(revoked, "subkey must be revoked");

    // Key is no longer active
    assert!(!registry.is_key_active(group_id));

    // Read attempt on physically remaining ciphertext must fail
    let read_res = manager.read_segment(&segment);
    assert!(
        read_res.is_err(),
        "read attempt on revoked subkey segment must fail"
    );
}

#[test]
fn test_tombstone_only_deletion_proof_fails_with_kv_delete_mode_config() {
    let registry = Arc::new(KeyRegistry::new());
    let config = KvSegmentConfig {
        delete_mode: KvDeleteMode::TombstoneOnly,
    };
    let manager = KvSegmentManager::new(config, registry, None);

    let group_id = 2002;
    let proof_res = manager.generate_deletion_proof(group_id);

    match proof_res {
        Err(ContextraError::KvDeleteModeConfig(msg)) => {
            assert!(
                msg.contains("CryptoShred"),
                "error message must mention CryptoShred restriction, got: {msg}"
            );
        }
        res => panic!("expected ContextraError::KvDeleteModeConfig, got {res:?}"),
    }
}

#[test]
fn test_default_config_crypto_shred_end_to_end() {
    let master_key = Arc::new(
        KeyManager::try_new("test-passphrase-default", b"salt-5678")
            .expect("master key initialization"),
    );
    let registry = Arc::new(KeyRegistry::new());

    // Default configuration uses KvDeleteMode::CryptoShred
    let config = KvSegmentConfig::default();
    assert_eq!(config.delete_mode, KvDeleteMode::CryptoShred);

    let manager = KvSegmentManager::new(config, Arc::clone(&registry), Some(master_key));

    let group_id = 3003;
    let plaintext = b"End to end default config test payload";

    // 1. Write segment
    let segment = manager
        .write_segment(group_id, plaintext)
        .expect("write segment");

    // 2. Read (succeeds)
    let read_data = manager
        .read_segment(&segment)
        .expect("read segment before deletion");
    assert_eq!(read_data.as_slice(), plaintext);

    // Deletion proof before delete should report not revoked (false)
    let proof_before = manager
        .generate_deletion_proof(group_id)
        .expect("deletion proof before delete");
    assert!(!proof_before);

    // 3. Delete segment
    let deleted = manager.delete_segment(group_id);
    assert!(deleted, "delete segment must return true");

    // Deletion proof after delete should report revoked (true)
    let proof_after = manager
        .generate_deletion_proof(group_id)
        .expect("deletion proof after delete");
    assert!(proof_after);

    // 4. Read (fails)
    let read_after_delete = manager.read_segment(&segment);
    assert!(
        read_after_delete.is_err(),
        "read segment after deletion must fail"
    );
}
