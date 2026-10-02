use super::*;
use contextra_core::TxId;
use tempfile::tempdir;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn test_open_heal_truncated_tail_plaintext() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("heal_plain.wal");

    // 1. Create WAL and append 3 entries
    {
        let wal = Wal::open(&path).await?;
        for i in 1..=3 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("key_{i}").into_bytes(),
                value: format!("val_{i}").into_bytes(),
            };
            let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
            wal.append_batch(batch).await?;
        }
    }

    // 2. Corrupt file by chopping off 10 bytes from the end (torn tail)
    let original_bytes = fs::read(&path).await?;
    let truncated_len = original_bytes.len() - 10;
    fs::write(&path, &original_bytes[..truncated_len]).await?;

    // 3. Open WAL (Open-Heal should truncate the torn tail on open)
    let wal = Wal::open(&path).await?;
    let size_after_heal = wal.size();
    assert!(
        size_after_heal < truncated_len as u64,
        "WAL size after open-heal should be truncated to verified end, got {size_after_heal}"
    );

    // 4. Append 1 new entry (entry 4)
    let op4 = WalOp::Put {
        tx_id: TxId::new(4),
        key: b"key_4".to_vec(),
        value: b"val_4".to_vec(),
    };
    let (batch4, _) = wal.prepare_batch(vec![(op4, 4)]).await?;
    wal.append_batch(batch4).await?;
    drop(wal);

    // 5. Reopen and verify all 3 valid entries (2 original + 1 new) are present with valid HMAC chain
    let wal_reopened = Wal::open(&path).await?;
    let replayed = wal_reopened.replay().await?;
    assert_eq!(
        replayed.len(),
        3,
        "Expected 3 valid entries after heal + append"
    );
    assert_eq!(replayed[0].1.seq_no, 1);
    assert_eq!(replayed[1].1.seq_no, 2);
    assert_eq!(replayed[2].1.seq_no, 4);

    Ok(())
}

#[tokio::test]
async fn test_open_heal_huge_len_garbage_tail() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("heal_huge_len.wal");

    // 1. Create WAL and append 2 entries
    {
        let wal = Wal::open(&path).await?;
        for i in 1..=2 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("k{i}").into_bytes(),
                value: format!("v{i}").into_bytes(),
            };
            let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
            wal.append_batch(batch).await?;
        }
    }

    // 2. Append huge length prefix + garbage bytes to tail
    {
        let mut file = fs::OpenOptions::new().append(true).open(&path).await?;
        file.write_all(&100_000_000u32.to_le_bytes()).await?;
        file.write_all(b"garbage_tail_data_1234567890").await?;
        file.flush().await?;
    }

    // 3. Open WAL and append 1 entry
    let wal = Wal::open(&path).await?;
    let op3 = WalOp::Put {
        tx_id: TxId::new(3),
        key: b"k3".to_vec(),
        value: b"v3".to_vec(),
    };
    let (batch3, _) = wal.prepare_batch(vec![(op3, 3)]).await?;
    wal.append_batch(batch3).await?;
    drop(wal);

    // 4. Reopen and verify all 3 entries are replayed cleanly
    let wal_reopened = Wal::open(&path).await?;
    let replayed = wal_reopened.replay().await?;
    assert_eq!(replayed.len(), 3);
    assert_eq!(replayed[0].1.seq_no, 1);
    assert_eq!(replayed[1].1.seq_no, 2);
    assert_eq!(replayed[2].1.seq_no, 3);

    Ok(())
}

#[tokio::test]
async fn test_open_heal_encrypted_wal() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("heal_encrypted.wal");
    let km = Arc::new(KeyManager::try_new(
        "heal_passphrase",
        b"salt123456789012345678901234567890",
    )?);

    let config = WalConfig {
        key_manager: Some(km.clone()),
        ..Default::default()
    };

    // 1. Write 3 encrypted entries
    {
        let wal = Wal::open_with_config(&path, config.clone()).await?;
        for i in 1..=3 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("enc_k{i}").into_bytes(),
                value: format!("enc_v{i}").into_bytes(),
            };
            let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
            wal.append_batch(batch).await?;
        }
    }

    // 2. Truncate 15 bytes off encrypted frame tail
    let original_bytes = fs::read(&path).await?;
    let truncated_len = original_bytes.len() - 15;
    fs::write(&path, &original_bytes[..truncated_len]).await?;

    // 3. Open WAL with Open-Heal and append entry 4
    let wal = Wal::open_with_config(&path, config.clone()).await?;
    let op4 = WalOp::Put {
        tx_id: TxId::new(4),
        key: b"enc_k4".to_vec(),
        value: b"enc_v4".to_vec(),
    };
    let (batch4, _) = wal.prepare_batch(vec![(op4, 4)]).await?;
    wal.append_batch(batch4).await?;
    drop(wal);

    // 4. Reopen and verify
    let wal_reopened = Wal::open_with_config(&path, config).await?;
    let replayed = wal_reopened.replay().await?;
    assert_eq!(
        replayed.len(),
        3,
        "Expected 2 recovered entries + 1 appended entry"
    );
    assert_eq!(replayed[0].1.seq_no, 1);
    assert_eq!(replayed[1].1.seq_no, 2);
    assert_eq!(replayed[2].1.seq_no, 4);

    Ok(())
}

#[tokio::test]
async fn test_open_read_only_never_modifies_file() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("read_only_heal.wal");

    // 1. Create WAL and write 2 entries
    {
        let wal = Wal::open(&path).await?;
        for i in 1..=2 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("ro_k{i}").into_bytes(),
                value: format!("ro_v{i}").into_bytes(),
            };
            let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
            wal.append_batch(batch).await?;
        }
    }

    // 2. Append 20 garbage bytes to tail
    {
        let mut file = fs::OpenOptions::new().append(true).open(&path).await?;
        file.write_all(b"garbage_tail_bytes_1234").await?;
        file.flush().await?;
    }

    let meta_before = fs::metadata(&path).await?;
    let len_before = meta_before.len();

    // 3. Open read-only
    let wal_ro = Wal::open_read_only(&path, None).await?;
    let replayed = wal_ro.replay().await?;
    assert_eq!(replayed.len(), 2);

    let meta_after = fs::metadata(&path).await?;
    assert_eq!(
        meta_after.len(),
        len_before,
        "open_read_only MUST NOT modify the file size or truncate physical bytes"
    );

    Ok(())
}

#[tokio::test]
async fn test_truncate_uncommitted_tail_with_and_without_tx_end() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("uncommitted_tail.wal");

    let wal = Wal::open(&path).await?;

    // Write TxEnd (committed) for tx 1, then Puts for tx 2 (no TxEnd)
    let op1 = WalOp::TxEnd {
        tx_id: TxId::new(1),
        committed: true,
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    wal.append_batch(batch1).await?;

    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"uncommitted_key".to_vec(),
        value: b"uncommitted_val".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;
    wal.append_batch(batch2).await?;

    // Call truncate_uncommitted_tail
    let removed_bytes = wal.truncate_uncommitted_tail().await?;
    assert!(
        removed_bytes > 0,
        "Should remove uncommitted bytes after last TxEnd"
    );

    // Replay should show only 1 entry (TxEnd)
    let replayed = wal.replay().await?;
    assert_eq!(replayed.len(), 1);
    assert_eq!(replayed[0].1.seq_no, 1);

    // Truncate again when no uncommitted tail exists
    let removed_again = wal.truncate_uncommitted_tail().await?;
    assert_eq!(removed_again, 0, "No-op when no uncommitted tail exists");

    Ok(())
}

#[tokio::test]
async fn test_middle_corruption_does_not_truncate() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("middle_corruption.wal");

    // Write 3 entries
    {
        let wal = Wal::open(&path).await?;
        for i in 1..=3 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("k{i}").into_bytes(),
                value: format!("v{i}").into_bytes(),
            };
            let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
            wal.append_batch(batch).await?;
        }
    }

    let original_bytes = fs::read(&path).await?;
    let orig_len = original_bytes.len();

    // Mutate a byte in the middle (e.g., position 20)
    let mut corrupted_bytes = original_bytes.clone();
    corrupted_bytes[20] ^= 0xFF;
    fs::write(&path, &corrupted_bytes).await?;

    // Open should fail with WalCorruption
    let open_res = Wal::open(&path).await;
    assert!(open_res.is_err(), "Open must fail on middle corruption");

    // Verify physical file is byte-identical and NOT truncated
    let bytes_after = fs::read(&path).await?;
    assert_eq!(
        bytes_after.len(),
        orig_len,
        "File length must remain unchanged on replay/corruption error"
    );
    assert_eq!(
        bytes_after, corrupted_bytes,
        "File contents must remain byte-identical on replay error"
    );

    Ok(())
}

#[cfg(feature = "fault-injection")]
#[tokio::test]
async fn test_fault_injection_recover_from_poison() -> Result<()> {
    use std::sync::atomic::Ordering;

    let dir = tempdir()?;
    let path = dir.path().join("fault_poison.wal");

    let wal = Wal::open(&path).await?;

    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    wal.append_batch(batch1).await?;

    // Arm fault injection
    FAIL_APPEND_PARTIAL_ONCE.store(true, Ordering::SeqCst);
    FAIL_APPEND_AFTER_PARTIAL_BYTES.store(10, Ordering::SeqCst);

    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;
    let append_res = wal.append_batch(batch2).await;
    assert!(append_res.is_err());
    assert!(wal.is_poisoned());

    // Recover from poison
    wal.recover_from_poison().await?;
    assert!(!wal.is_poisoned());

    // Write new entry
    let op3 = WalOp::Put {
        tx_id: TxId::new(3),
        key: b"k3".to_vec(),
        value: b"v3".to_vec(),
    };
    let (batch3, _) = wal.prepare_batch(vec![(op3, 3)]).await?;
    wal.append_batch(batch3).await?;
    drop(wal);

    let wal_reopened = Wal::open(&path).await?;
    let replayed = wal_reopened.replay().await?;
    assert_eq!(replayed.len(), 2);
    assert_eq!(replayed[0].1.seq_no, 1);
    assert_eq!(replayed[1].1.seq_no, 3);

    Ok(())
}
