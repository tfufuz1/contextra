use contextra_core::TxId;
use contextra_store::wal::{Wal, WalEntry, WalOp, WAL_V3_HEADER};
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_legacy_migration_crash_recovery_step_by_step() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("crash_recovery_matrix.wal");

    let op = WalOp::Put {
        tx_id: TxId::new(42),
        key: b"migration_crash_key".to_vec(),
        value: b"migration_crash_val".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 42, &legacy_key, [0u8; 32]).expect("legacy entry");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));
    fs::write(&wal_path, &wal_bytes).await.expect("write wal");

    let key_path = dir.path().join(".wal_integrity_key");
    let marker_path = dir.path().join("crash_recovery_matrix.wal.rekeyed");

    // Case 1: Interrupted prior to migration (neither key file nor marker exists)
    assert!(!key_path.exists());
    assert!(!marker_path.exists());

    let wal1 = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("migration step 1");
    drop(wal1);

    assert!(key_path.exists());
    assert!(marker_path.exists());

    // Case 2: Post-migration re-open via standard Wal::open must succeed and NOT require legacy key
    let std_wal = Wal::open(&wal_path)
        .await
        .expect("standard open post-migration must succeed");

    let entries = std_wal.replay().await.expect("replay post-migration");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.seq_no, 42);

    // Case 3: Re-opening with open_for_legacy_migration when marker is present must bypass legacy key
    let re_mig_wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("open_for_legacy_migration on already migrated WAL");
    let re_mig_entries = re_mig_wal.replay().await.expect("re-migration replay");
    assert_eq!(re_mig_entries.len(), 1);
}
