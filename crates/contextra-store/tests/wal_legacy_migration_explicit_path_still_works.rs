use contextra_core::TxId;
use contextra_store::wal::{Wal, WalEntry, WalOp};
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_open_for_legacy_migration_v1_works() {
    let dir = tempdir().expect("create tempdir");
    let wal_path = dir.path().join("v1_legacy.wal");

    let op = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"v1_migration_key".to_vec(),
        value: b"v1_migration_val".to_vec(),
    };
    let entry = WalEntry::try_new(op, 1, &Wal::legacy_integrity_key_for_test(), [0u8; 32])
        .expect("create v1 entry");
    let entry_bytes = entry.to_bytes().expect("serialize entry");

    fs::write(&wal_path, &entry_bytes)
        .await
        .expect("write v1 wal");

    let wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("open_for_legacy_migration should succeed for V1 WAL");

    assert!(
        wal.was_legacy_rekeyed(),
        "V1 WAL segment should be marked as rekeyed after open_for_legacy_migration"
    );

    let entries = wal.replay().await.expect("replay entries after migration");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.tx_id(), TxId::new(10));
}

#[tokio::test]
async fn test_migrate_legacy_wal_v1_works() {
    let dir = tempdir().expect("create tempdir");
    let wal_path = dir.path().join("v1_migrate.wal");

    let op = WalOp::Put {
        tx_id: TxId::new(11),
        key: b"v1_migrate_key".to_vec(),
        value: b"v1_migrate_val".to_vec(),
    };
    let entry = WalEntry::try_new(op, 1, &Wal::legacy_integrity_key_for_test(), [0u8; 32])
        .expect("create v1 entry");
    let entry_bytes = entry.to_bytes().expect("serialize entry");

    fs::write(&wal_path, &entry_bytes)
        .await
        .expect("write v1 wal");

    let migrated = Wal::migrate_legacy_wal(&wal_path, None)
        .await
        .expect("migrate_legacy_wal should succeed for V1 WAL");

    assert!(
        migrated,
        "migrate_legacy_wal should return true when rekeying V1 WAL"
    );

    // Opening migrated file via standard Wal::open should now succeed because it was converted to V3.
    let wal = Wal::open(&wal_path)
        .await
        .expect("opening migrated V3 WAL via standard Wal::open should succeed");

    let entries = wal.replay().await.expect("replay entries");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.tx_id(), TxId::new(11));
}

#[tokio::test]
async fn test_open_for_legacy_migration_v2_works() {
    let dir = tempdir().expect("create tempdir");
    let wal_path = dir.path().join("v2_legacy.wal");

    let op = WalOp::Put {
        tx_id: TxId::new(12),
        key: b"v2_migration_key".to_vec(),
        value: b"v2_migration_val".to_vec(),
    };
    let integrity_key = [2u8; 32];
    let key_path = dir.path().join(".wal_integrity_key");
    fs::write(&key_path, &integrity_key)
        .await
        .expect("write integrity key");

    let mut entry = WalEntry::try_new(op, 1, &integrity_key, [0u8; 32])
        .expect("create v2 entry");
    entry.checksum = WalEntry::compute_checksum_v2(&entry.op, entry.seq_no, &integrity_key, entry.prev_hmac)
        .expect("compute v2 checksum");
    let entry_bytes = entry.to_bytes().expect("serialize entry");

    let mut payload = Vec::new();
    payload.extend_from_slice(b"MFW2");
    payload.extend_from_slice(&entry_bytes);

    fs::write(&wal_path, &payload)
        .await
        .expect("write v2 wal");

    let wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("open_for_legacy_migration should succeed for V2 WAL");

    assert!(
        wal.was_legacy_rekeyed(),
        "V2 WAL segment should be marked as rekeyed after open_for_legacy_migration"
    );

    let entries = wal.replay().await.expect("replay entries after migration");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.tx_id(), TxId::new(12));
}
