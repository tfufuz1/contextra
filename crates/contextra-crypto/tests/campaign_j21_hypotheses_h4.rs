use contextra_crypto::{
    crypto::KeyManager,
    kv_segment::{KvDeleteMode, KvSegmentConfig, KvSegmentManager},
    kv_shredding::KeyRegistry,
};
use std::sync::Arc;

#[test]
fn test_h4_kv_segment_deletion_proof_state_binding() {
    let registry = Arc::new(KeyRegistry::new());
    let master_key = Arc::new(
        KeyManager::try_new("master-passphrase-shredding", b"master-salt-123").expect("KeyManager"),
    );

    let mgr_tombstone = KvSegmentManager::new(
        KvSegmentConfig {
            delete_mode: KvDeleteMode::TombstoneOnly,
        },
        registry.clone(),
        Some(master_key.clone()),
    );

    let proof_tombstone = mgr_tombstone.generate_deletion_proof(100);
    assert!(
        proof_tombstone.is_err(),
        "TombstoneOnly mode MUST reject deletion proof generation"
    );

    let mgr_shred = KvSegmentManager::new(
        KvSegmentConfig {
            delete_mode: KvDeleteMode::CryptoShred,
        },
        registry.clone(),
        Some(master_key.clone()),
    );

    let group_id = 42u64;
    let payload = mgr_shred.write_segment(group_id, b"secret_data").unwrap();

    let proof_before = mgr_shred.generate_deletion_proof(group_id).unwrap();
    assert!(
        !proof_before,
        "Deletion proof must be false when segment key is still active"
    );

    let read_back = mgr_shred.read_segment(&payload).unwrap();
    assert_eq!(read_back, b"secret_data");

    let revoked = mgr_shred.delete_segment(group_id).unwrap();
    assert!(revoked, "Subkey revocation must return true");

    let proof_after = mgr_shred.generate_deletion_proof(group_id).unwrap();
    assert!(
        proof_after,
        "Deletion proof must be true after crypto shredding"
    );

    assert!(mgr_shred.read_segment(&payload).is_err());
}
