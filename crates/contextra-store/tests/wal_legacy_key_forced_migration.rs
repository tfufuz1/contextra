use contextra_core::TxId;
use contextra_store::wal::{Wal, WalEntry, WalOp, WAL_V3_HEADER};
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_forced_legacy_key_migration_creates_secure_key_and_marker() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("legacy_forced.wal");

    // 1. Manually construct a legacy WAL segment signed exclusively with the legacy integrity key
    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"legacy_key_1".to_vec(),
        value: b"legacy_val_1".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry1 = WalEntry::try_new(op1, 1, &legacy_key, [0u8; 32]).expect("legacy entry 1");

    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"legacy_key_2".to_vec(),
        value: b"legacy_val_2".to_vec(),
    };
    let legacy_entry2 =
        WalEntry::try_new(op2, 2, &legacy_key, legacy_entry1.checksum).expect("legacy entry 2");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry1.to_bytes().expect("to_bytes entry 1"));
    wal_bytes.extend_from_slice(&legacy_entry2.to_bytes().expect("to_bytes entry 2"));

    fs::write(&wal_path, wal_bytes)
        .await
        .expect("write legacy WAL");

    // Confirm .wal_integrity_key and .rekeyed marker do NOT exist before migration
    let key_path = dir.path().join(".wal_integrity_key");
    let marker_path = dir.path().join("legacy_forced.wal.rekeyed");
    assert!(
        !key_path.exists(),
        ".wal_integrity_key sidecar must not exist before migration"
    );
    assert!(
        !marker_path.exists(),
        ".rekeyed marker must not exist before migration"
    );

    // 2. Open via open_for_legacy_migration()
    let wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("open_for_legacy_migration must succeed");

    // 3. Verify .wal_integrity_key file was created and contains a 32-byte secure key
    assert!(
        key_path.exists(),
        ".wal_integrity_key sidecar must be persisted during rekeying"
    );
    let key_bytes = fs::read(&key_path).await.expect("read key file");
    assert_eq!(key_bytes.len(), 32);
    assert_ne!(
        key_bytes.as_slice(),
        &legacy_key[..],
        "Persisted key must NOT be equal to the legacy static XOR key"
    );

    // Verify atomic .rekeyed marker file was written
    assert!(
        marker_path.exists(),
        ".rekeyed migration marker must exist after migration"
    );

    // 4. Verify entries can be replayed and match original operations
    let entries = wal.replay().await.expect("replay migrated wal");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].1.seq_no, 1);
    assert_eq!(entries[1].1.seq_no, 2);

    // 5. Append new entry using the migrated WAL handle
    let op3 = WalOp::Put {
        tx_id: TxId::new(3),
        key: b"post_mig_key".to_vec(),
        value: b"post_mig_val".to_vec(),
    };
    let (batch, _prev_hmac) = wal
        .prepare_batch(vec![(op3, 3)])
        .await
        .expect("prepare_batch");
    wal.append_batch(batch).await.expect("append_batch");

    let entries_after = wal.replay().await.expect("replay after append");
    assert_eq!(entries_after.len(), 3);
}

#[tokio::test]
async fn test_standard_wal_open_succeeds_post_migration() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("legacy_standard.wal");

    // Construct legacy WAL segment
    let op = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"key_before_migration".to_vec(),
        value: b"val_before_migration".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 10, &legacy_key, [0u8; 32]).expect("entry");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));
    fs::write(&wal_path, wal_bytes).await.expect("write wal");

    // 1. First open with open_for_legacy_migration() to migrate
    let mig_wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("open_for_legacy_migration");
    drop(mig_wal);

    // 2. Second open using STANDARD Wal::open (without legacy fallback flag) MUST succeed
    let std_wal = Wal::open(&wal_path)
        .await
        .expect("Standard Wal::open must succeed on migrated WAL");

    let entries = std_wal.replay().await.expect("replay standard open");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.seq_no, 10);

    // 3. Write additional entries via standard open handle
    let op2 = WalOp::Put {
        tx_id: TxId::new(11),
        key: b"key_after_migration".to_vec(),
        value: b"val_after_migration".to_vec(),
    };
    let (batch, _) = std_wal.prepare_batch(vec![(op2, 11)]).await.expect("prep");
    std_wal.append_batch(batch).await.expect("append");

    let final_entries = std_wal.replay().await.expect("final replay");
    assert_eq!(final_entries.len(), 2);
}

#[tokio::test]
async fn test_re_migration_open_does_not_use_legacy_key() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("legacy_remigrate.wal");

    // Construct legacy WAL segment
    let op = WalOp::Put {
        tx_id: TxId::new(100),
        key: b"remigrate_k".to_vec(),
        value: b"remigrate_v".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 100, &legacy_key, [0u8; 32]).expect("entry");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));
    fs::write(&wal_path, wal_bytes).await.expect("write wal");

    // 1. Initial migration
    let wal1 = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("first migration");
    drop(wal1);

    // Read the secure key persisted after first migration
    let key_path = dir.path().join(".wal_integrity_key");
    let secure_key_after_first = fs::read(&key_path).await.expect("read key path");

    // 2. Re-call open_for_legacy_migration on the already migrated WAL
    let wal2 = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("second open_for_legacy_migration");

    // Verify key was NOT overwritten or reverted to legacy
    let secure_key_after_second = fs::read(&key_path).await.expect("read key path");
    assert_eq!(
        secure_key_after_first, secure_key_after_second,
        "Secure integrity key must remain unchanged on subsequent opens"
    );

    let entries = wal2.replay().await.expect("replay after second open");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.seq_no, 100);
}

#[tokio::test]
async fn test_crash_recovery_step1_interrupted_before_rewrite() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("crash_step1.wal");

    // Prepare legacy WAL file
    let op = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"crash_step1_key".to_vec(),
        value: b"crash_step1_val".to_vec(),
    };
    let legacy_key = Wal::legacy_integrity_key_for_test();
    let legacy_entry = WalEntry::try_new(op, 1, &legacy_key, [0u8; 32]).expect("entry");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));
    fs::write(&wal_path, wal_bytes).await.expect("write wal");

    // Simulate crash after Step 1: .wal_integrity_key exists, but WAL file is STILL in legacy format and no .rekeyed marker exists
    let mock_secure_key = [0x77u8; 32];
    let key_path = dir.path().join(".wal_integrity_key");
    fs::write(&key_path, mock_secure_key)
        .await
        .expect("write mock key");

    let marker_path = dir.path().join("crash_step1.wal.rekeyed");
    assert!(!marker_path.exists());

    // Call open_for_legacy_migration: must succeed, replay using legacy key fallback, rewrite as V3 with secure key, and create marker
    let wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("open_for_legacy_migration after step1 crash");

    assert!(
        marker_path.exists(),
        "Marker must be created during recovery"
    );

    let entries = wal.replay().await.expect("replay");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.seq_no, 1);
}

#[tokio::test]
async fn test_crash_recovery_step2_interrupted_after_rewrite_before_marker() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("crash_step2.wal");

    // Simulate crash after Step 3: WAL file WAS rewritten with secure key and .wal_integrity_key exists, BUT process crashed before .rekeyed marker was written
    let secure_key = [0x88u8; 32];
    let key_path = dir.path().join(".wal_integrity_key");
    fs::write(&key_path, secure_key)
        .await
        .expect("write secure key");

    let op = WalOp::Put {
        tx_id: TxId::new(20),
        key: b"crash_step2_key".to_vec(),
        value: b"crash_step2_val".to_vec(),
    };
    let secure_entry = WalEntry::try_new(op, 20, &secure_key, [0u8; 32]).expect("secure entry");

    let mut wal_bytes = Vec::new();
    wal_bytes.extend_from_slice(&WAL_V3_HEADER);
    wal_bytes.extend_from_slice(&secure_entry.to_bytes().expect("to_bytes"));
    fs::write(&wal_path, wal_bytes).await.expect("write wal");

    let marker_path = dir.path().join("crash_step2.wal.rekeyed");
    assert!(!marker_path.exists());

    // Call open_for_legacy_migration: since entries verify with secure_key, legacy_key_used remains false, marker is written, and handle opens cleanly
    let wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("open_for_legacy_migration after step2 crash");

    assert!(
        marker_path.exists(),
        "Marker must be written after step2 crash recovery"
    );

    let entries = wal.replay().await.expect("replay");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1.seq_no, 20);
}
