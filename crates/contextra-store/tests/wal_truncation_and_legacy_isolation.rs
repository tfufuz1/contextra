use contextra_core::{ContextraError, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::manifest::{Manifest, ManifestEntry};
use contextra_store::wal::{Wal, WalEntry, WalOp, WAL_V3_HEADER};
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_external_wal_truncation_attack_detected() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let manifest_path = dir.path().join("MANIFEST");
    let wal_path = dir.path().join("wal.log");

    // Write SALT file so directory pristine check passes when LsmStorage opens
    fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .expect("write salt");

    // 1. Write 2 WAL entries and record high-water-mark of entry 2 in MANIFEST
    let hwm_hmac = {
        let wal = Wal::open(&wal_path).await.expect("open wal");
        let op1 = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        };
        let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await.expect("batch 1");
        wal.append_batch(batch1).await.expect("append 1");

        let op2 = WalOp::Put {
            tx_id: TxId::new(2),
            key: b"k2".to_vec(),
            value: b"v2".to_vec(),
        };
        let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await.expect("batch 2");
        wal.append_batch(batch2).await.expect("append 2");

        let hwm = wal.last_hmac_snapshot().await;
        drop(wal);

        let manifest = Manifest::open(&manifest_path).await.expect("manifest open");
        manifest
            .append(&ManifestEntry::WalCheckpoint { hmac: hwm })
            .await
            .expect("checkpoint append");

        hwm
    };

    // 2. Truncate wal.log to remove entry 2 cleanly (external truncation attack)
    let wal_read = Wal::open(&wal_path).await.expect("open wal read");
    let entries = wal_read.replay().await.expect("replay entries");
    assert_eq!(entries.len(), 2);
    let (_, _, offset_first) = entries[0];
    drop(wal_read);

    let mut wal_bytes = fs::read(&wal_path).await.expect("read wal");
    wal_bytes.truncate(offset_first as usize);
    fs::write(&wal_path, wal_bytes)
        .await
        .expect("write truncated wal");

    // 3. Attempting to open LSM storage must fail with WalTruncationDetected
    let reopen_res = LsmStorage::new(config).await;
    assert!(
        reopen_res.is_err(),
        "LsmStorage startup MUST detect external WAL truncation against MANIFEST high water mark"
    );
    let err = reopen_res.err().unwrap();
    match err {
        ContextraError::WalTruncationDetected {
            expected_hmac,
            actual_hmac,
        } => {
            assert_eq!(expected_hmac, hwm_hmac);
            assert_ne!(actual_hmac, expected_hmac);
        }
        _ => panic!("Expected WalTruncationDetected error, got: {:?}", err),
    }
}

#[tokio::test]
async fn test_crash_incomplete_tail_write_recovered_without_truncation_error() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let manifest_path = dir.path().join("MANIFEST");
    let wal_path = dir.path().join("wal.log");

    // Write SALT file so directory pristine check passes when LsmStorage opens
    fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .expect("write salt");

    // 1. Write 2 valid WAL entries and record high-water-mark of entry 2 in MANIFEST
    {
        let wal = Wal::open(&wal_path).await.expect("open wal");
        let op1 = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        };
        let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await.expect("batch 1");
        wal.append_batch(batch1).await.expect("append 1");

        let op2 = WalOp::Put {
            tx_id: TxId::new(2),
            key: b"k2".to_vec(),
            value: b"v2".to_vec(),
        };
        let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await.expect("batch 2");
        wal.append_batch(batch2).await.expect("append 2");

        let hwm = wal.last_hmac_snapshot().await;
        drop(wal);

        let manifest = Manifest::open(&manifest_path).await.expect("manifest open");
        manifest
            .append(&ManifestEntry::WalCheckpoint { hmac: hwm })
            .await
            .expect("checkpoint append");
    }

    // 2. Append torn/incomplete entry bytes at the tail simulating a process crash mid-write after entry 2
    let mut wal_bytes = fs::read(&wal_path).await.expect("read wal");
    wal_bytes.extend_from_slice(&[0x00, 0x00, 0x10, 0x00, 0xDE, 0xAD, 0xBE, 0xEF]); // incomplete payload
    fs::write(&wal_path, wal_bytes)
        .await
        .expect("write wal with torn tail");

    // 3. Opening LSM storage MUST succeed by ignoring torn tail write without WalTruncationDetected
    let reopen_res = LsmStorage::new(config).await;
    assert!(
        reopen_res.is_ok(),
        "Incomplete torn write at tail from crash MUST be recovered cleanly without truncation error: {:?}",
        reopen_res.err()
    );
}

#[tokio::test]
async fn test_legacy_key_migration_isolation() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("legacy.wal");

    // 1. Create a non-legacy integrity key for the active WAL directory
    let non_legacy_key = [0xAAu8; 32];
    fs::write(dir.path().join(".wal_integrity_key"), non_legacy_key)
        .await
        .expect("write non-legacy key");

    // 2. Write a legacy WAL file signed with the legacy static key
    {
        let legacy_key = Wal::legacy_integrity_key_for_test();
        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"legacy_k".to_vec(),
            value: b"legacy_v".to_vec(),
        };
        let legacy_entry = WalEntry::try_new(op, 1, &legacy_key, [0u8; 32]).expect("entry");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&WAL_V3_HEADER);
        bytes.extend_from_slice(&legacy_entry.to_bytes().expect("to_bytes"));
        fs::write(&wal_path, bytes)
            .await
            .expect("write legacy file");
    }

    // 3. Standard open MUST fail because active key != legacy key and fallback is false
    let std_res = Wal::open(&wal_path).await;
    let replay_failed = match std_res {
        Ok(wal) => wal.replay().await.is_err(),
        Err(_) => true,
    };
    assert!(
        replay_failed,
        "Standard Wal::open MUST fail on legacy WAL without explicit migration opt-in"
    );

    // 4. Migration open MUST succeed by using legacy fallback
    let mig_wal = Wal::open_for_legacy_migration(&wal_path, None)
        .await
        .expect("open_for_legacy_migration");
    let entries = mig_wal.replay().await.expect("replay migration wal");
    assert_eq!(entries.len(), 1);
}
