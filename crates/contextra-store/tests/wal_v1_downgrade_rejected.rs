use contextra_core::TxId;
use contextra_store::wal::{Wal, WalEntry, WalOp, WAL_V2_HEADER, WAL_V3_HEADER};
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_v1_header_tampered_entry_rejected_on_default_config() {
    let dir = tempdir().expect("tempdir creation failed");
    let wal_path = dir.path().join("tampered_v1.wal");
    let bak_path = dir.path().join("tampered_v1.wal.v1.bak");

    // 1. Construct a V1 WAL file on disk containing a tampered entry (using legacy integrity key or wrong payload)
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"secret_key".to_vec(),
        value: b"tampered_val".to_vec(),
    };
    // Construct V1 entry (no MFW3/MFW2 header)
    let entry = WalEntry::try_new(op, 1, &Wal::legacy_integrity_key_for_test(), [0u8; 32])
        .expect("create v1 entry");
    let entry_bytes = entry.to_bytes().expect("serialize entry");

    fs::write(&wal_path, &entry_bytes)
        .await
        .expect("write v1 wal");

    let original_bytes = fs::read(&wal_path).await.expect("read original bytes");

    // 2. Opening with default WalConfig (min_wal_version = V3, allow_legacy_integrity_key_fallback = false)
    // MUST be rejected with an error!
    let open_res = Wal::open(&wal_path).await;

    assert!(
        open_res.is_err(),
        "Opening V1 WAL with default WalConfig MUST return an error"
    );

    // 3. Verify file on disk remains byte-identical and no .v1.bak backup file is created
    assert!(
        !bak_path.exists(),
        "No backup file *.v1.bak should be created when V1 open is rejected"
    );

    let current_bytes = fs::read(&wal_path).await.expect("read current bytes");
    assert_eq!(
        original_bytes, current_bytes,
        "WAL file on disk must remain byte-identical after rejected open"
    );
}

#[tokio::test]
async fn test_v3_header_tampered_to_v1_rejected() {
    let dir = tempdir().expect("tempdir creation failed");
    let wal_path = dir.path().join("tampered_header_v1.wal");

    // 1. Create a valid V3 WAL file with 1 entry
    {
        let wal = Wal::open(&wal_path).await.expect("create v3 wal");
        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await.expect("prepare");
        wal.append_batch(batch).await.expect("append");
        wal.close().await.expect("close wal");
    }

    let mut disk_data = fs::read(&wal_path).await.expect("read v3 disk data");
    assert_eq!(&disk_data[0..4], &WAL_V3_HEADER);

    // 2. Strip or overwrite the V3 header bytes so it appears as V1
    disk_data.drain(0..4);
    fs::write(&wal_path, &disk_data)
        .await
        .expect("write tampered v1 data");

    let original_tampered_bytes = fs::read(&wal_path).await.expect("read tampered bytes");

    // 3. Attempt to open with default config MUST fail
    let open_res = Wal::open(&wal_path).await;
    assert!(
        open_res.is_err(),
        "Replaying V3 WAL tampered to V1 header MUST fail under default config"
    );

    let current_bytes = fs::read(&wal_path).await.expect("read current bytes");
    assert_eq!(
        original_tampered_bytes, current_bytes,
        "File on disk must remain untouched when opening tampered V1 header fails"
    );
}

#[tokio::test]
async fn test_v3_header_tampered_to_v2_rejected() {
    let dir = tempdir().expect("tempdir creation failed");
    let wal_path = dir.path().join("tampered_header_v2.wal");

    // 1. Create a valid V3 WAL file with 1 entry
    {
        let wal = Wal::open(&wal_path).await.expect("create v3 wal");
        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await.expect("prepare");
        wal.append_batch(batch).await.expect("append");
        wal.close().await.expect("close wal");
    }

    let mut disk_data = fs::read(&wal_path).await.expect("read v3 disk data");
    assert_eq!(&disk_data[0..4], &WAL_V3_HEADER);

    // 2. Change header to WAL_V2_HEADER ("MFW2")
    disk_data[0..4].copy_from_slice(&WAL_V2_HEADER);
    fs::write(&wal_path, &disk_data)
        .await
        .expect("write tampered v2 header");

    let original_tampered_bytes = fs::read(&wal_path).await.expect("read tampered bytes");

    // 3. Attempt to open with default config MUST fail
    let open_res = Wal::open(&wal_path).await;
    assert!(
        open_res.is_err(),
        "Replaying V3 WAL tampered to V2 header MUST fail under default config"
    );

    let current_bytes = fs::read(&wal_path).await.expect("read current bytes");
    assert_eq!(
        original_tampered_bytes, current_bytes,
        "File on disk must remain untouched when opening tampered V2 header fails"
    );
}

#[tokio::test]
async fn test_explicit_migration_path_succeeds() {
    let dir = tempdir().expect("tempdir creation failed");
    let wal_path = dir.path().join("explicit_migrate.wal");

    // 1. Construct V1 WAL file
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"mig_key".to_vec(),
        value: b"mig_val".to_vec(),
    };
    let entry = WalEntry::try_new(op, 1, &Wal::legacy_integrity_key_for_test(), [0u8; 32])
        .expect("create v1 entry");
    let entry_bytes = entry.to_bytes().expect("serialize entry");

    fs::write(&wal_path, &entry_bytes)
        .await
        .expect("write v1 wal");

    // 2. Open via explicit migration helper
    let wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("open_for_legacy_migration should succeed for explicit migration");

    let entries = wal.replay().await.expect("replay migrated wal");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.seq_no, 1);
}
