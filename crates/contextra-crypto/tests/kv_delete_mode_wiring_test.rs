use contextra_crypto::{
    crypto::KeyManager,
    kv_segment::{KvDeleteMode, KvSegmentConfig, KvSegmentManager},
    kv_shredding::KeyRegistry,
    CryptoError,
};
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

    let segment = manager
        .write_segment(group_id, plaintext)
        .expect("write encrypted segment");
    assert!(segment.is_encrypted);
    assert_ne!(segment.payload.as_slice(), plaintext);

    assert!(registry.is_key_active(group_id));

    let read_back = manager
        .read_segment(&segment)
        .expect("read before revocation");
    assert_eq!(read_back.as_slice(), plaintext);

    let revoked = manager.delete_segment(group_id).expect("delete_segment");
    assert!(revoked, "subkey must be revoked");

    assert!(!registry.is_key_active(group_id));

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
        Err(CryptoError::Crypto(msg)) => {
            assert!(
                msg.contains("CryptoShred"),
                "error message must mention CryptoShred restriction, got: {msg}"
            );
        }
        res => panic!("expected CryptoError::Crypto with restriction message, got {res:?}"),
    }
}

#[test]
fn test_default_config_crypto_shred_end_to_end() {
    let master_key = Arc::new(
        KeyManager::try_new("test-passphrase-default", b"salt-5678")
            .expect("master key initialization"),
    );
    let registry = Arc::new(KeyRegistry::new());

    let config = KvSegmentConfig::default();
    assert_eq!(config.delete_mode, KvDeleteMode::CryptoShred);

    let manager = KvSegmentManager::new(config, Arc::clone(&registry), Some(master_key));

    let group_id = 3003;
    let plaintext = b"End to end default config test payload";

    let segment = manager
        .write_segment(group_id, plaintext)
        .expect("write segment");

    let read_data = manager
        .read_segment(&segment)
        .expect("read segment before deletion");
    assert_eq!(read_data.as_slice(), plaintext);

    let proof_before = manager
        .generate_deletion_proof(group_id)
        .expect("deletion proof before delete");
    assert!(!proof_before);

    let deleted = manager.delete_segment(group_id).expect("delete_segment");
    assert!(deleted, "delete segment must return true");

    let proof_after = manager
        .generate_deletion_proof(group_id)
        .expect("deletion proof after delete");
    assert!(proof_after);

    let read_after_delete = manager.read_segment(&segment);
    assert!(
        read_after_delete.is_err(),
        "read segment after deletion must fail"
    );
}
