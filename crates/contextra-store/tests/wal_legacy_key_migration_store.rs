// ZWECK: Integrationstests für Store-Level Legacy WAL Migration (`migrate_legacy_wal_keys`, `has_pending_legacy_wal_migration`)
// INVARIANTEN: Regular open rejects legacy WAL without migration (INV-WAL-LEGACY-KEY-1); explicit migration rekeys & logs via Clock.

#![cfg(feature = "legacy-wal-key")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::TxId;
use contextra_ports::{Clock, SystemClock};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::wal::{Wal, WalEntry, WalOp, WAL_V3_HEADER};
use std::sync::Arc;
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_regular_open_rejects_legacy_wal_without_migration() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().to_path_buf();
    let wal_path = db_path.join("wal.log");

    // Construct a legacy WAL file signed with the legacy key
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"legacy_key".to_vec(),
        value: b"legacy_val".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 1, &legacy_key, [0u8; 32]).expect("legacy entry");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));
    fs::write(&wal_path, wal_bytes).await.expect("write wal");

    // 1. Check pending status before migration
    let has_pending = LsmStorage::has_pending_legacy_wal_migration_path(&db_path)
        .await
        .expect("has_pending_legacy_wal_migration_path");
    assert!(
        has_pending,
        "Database must indicate pending legacy WAL migration"
    );

    // 2. Regular LsmStorage::open without migration must fail
    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };
    let open_res = LsmStorage::open(config).await;
    assert!(
        open_res.is_err(),
        "Regular LsmStorage::open must refuse unmigrated legacy WAL file"
    );
}

#[tokio::test]
async fn test_explicit_migration_rekeys_and_clears_pending_status() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().to_path_buf();
    let wal_path = db_path.join("wal.log");

    // Construct legacy WAL file
    let op = WalOp::Put {
        tx_id: TxId::new(42),
        key: b"k_mig".to_vec(),
        value: b"v_mig".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 42, &legacy_key, [0u8; 32]).expect("legacy entry");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));
    fs::write(&wal_path, wal_bytes).await.expect("write wal");

    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    // 1. Perform explicit migration
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
    let migrated = LsmStorage::migrate_legacy_wal_keys(&config, clock)
        .await
        .expect("migrate_legacy_wal_keys");
    assert_eq!(migrated, 1, "Exactly 1 WAL segment should be migrated");

    // 2. Check pending status post-migration
    let has_pending = LsmStorage::has_pending_legacy_wal_migration_path(&db_path)
        .await
        .expect("has_pending");
    assert!(
        !has_pending,
        "has_pending_legacy_wal_migration_path must return false after migration"
    );

    // 3. Regular LsmStorage::open must now succeed
    let db = LsmStorage::open(config)
        .await
        .expect("LsmStorage::open after migration");
    assert!(
        !db.has_pending_legacy_wal_migration()
            .await
            .expect("has_pending"),
        "has_pending_legacy_wal_migration() must return false on active db"
    );
    db.close().await.expect("close");
}
