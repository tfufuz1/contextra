use contextra_core::{Result, TxId};
use contextra_store::wal::{KeyManager, Wal, WalConfig, WalOp};
use tempfile::tempdir;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn test_integration_open_heal_torn_tail_recovery() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("integration_heal.wal");

    // 1. Write 3 valid entries
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

    // 2. Chop 12 bytes from the end (partial frame write)
    let original_bytes = fs::read(&path).await?;
    let truncated_len = original_bytes.len() - 12;
    fs::write(&path, &original_bytes[..truncated_len]).await?;

    // 3. Reopen (open-heal should truncate physical file to last valid offset)
    let wal = Wal::open(&path).await?;
    let size_healed = wal.size();
    assert!(size_healed < truncated_len as u64);

    // 4. Append entry 4 (with seq_no 3 following recovered entry 2)
    let op4 = WalOp::Put {
        tx_id: TxId::new(4),
        key: b"k4".to_vec(),
        value: b"v4".to_vec(),
    };
    let (batch4, _) = wal.prepare_batch(vec![(op4, 3)]).await?;
    wal.append_batch(batch4).await?;
    drop(wal);

    // 5. Reopen and verify replay yields 3 valid entries (1, 2, 3)
    let wal_reopened = Wal::open(&path).await?;
    let replayed = wal_reopened.replay().await?;
    assert_eq!(replayed.len(), 3);
    assert_eq!(replayed[0].1.seq_no, 1);
    assert_eq!(replayed[1].1.seq_no, 2);
    assert_eq!(replayed[2].1.seq_no, 3);

    Ok(())
}

#[tokio::test]
async fn test_integration_open_heal_huge_length_prefix() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("integration_huge_prefix.wal");

    {
        let wal = Wal::open(&path).await?;
        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"valid_key".to_vec(),
            value: b"valid_val".to_vec(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
        wal.append_batch(batch).await?;
    }

    // Append fake huge length header
    {
        let mut file = fs::OpenOptions::new().append(true).open(&path).await?;
        file.write_all(&50_000_000u32.to_le_bytes()).await?;
        file.write_all(b"fake_payload_data").await?;
        file.flush().await?;
    }

    // Open-heal and write new entry
    let wal = Wal::open(&path).await?;
    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"valid_key_2".to_vec(),
        value: b"valid_val_2".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;
    wal.append_batch(batch2).await?;
    drop(wal);

    let wal2 = Wal::open(&path).await?;
    let replayed = wal2.replay().await?;
    assert_eq!(replayed.len(), 2);
    assert_eq!(replayed[0].1.seq_no, 1);
    assert_eq!(replayed[1].1.seq_no, 2);

    Ok(())
}

#[tokio::test]
async fn test_integration_open_read_only_does_not_modify_disk() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("integration_ro.wal");

    {
        let wal = Wal::open(&path).await?;
        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
        wal.append_batch(batch).await?;
    }

    // Add garbage
    {
        let mut file = fs::OpenOptions::new().append(true).open(&path).await?;
        file.write_all(b"garbage_junk_tail").await?;
        file.flush().await?;
    }

    let len_with_garbage = fs::metadata(&path).await?.len();

    let ro = Wal::open_read_only(&path, None).await?;
    let replayed = ro.replay().await?;
    assert_eq!(replayed.len(), 1);

    let len_after_ro = fs::metadata(&path).await?.len();
    assert_eq!(len_with_garbage, len_after_ro);

    Ok(())
}

#[tokio::test]
async fn test_integration_encrypted_wal_open_heal() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("integration_enc_heal.wal");
    let km = std::sync::Arc::new(KeyManager::try_new(
        "passphrase",
        b"salt123456789012345678901234567890",
    )?);
    let mut config = WalConfig::default();
    config.key_manager = Some(km);

    {
        let wal = Wal::open_with_config(&path, config.clone()).await?;
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

    // Chop off 8 bytes
    let raw = fs::read(&path).await?;
    fs::write(&path, &raw[..raw.len() - 8]).await?;

    let wal = Wal::open_with_config(&path, config.clone()).await?;
    let op4 = WalOp::Put {
        tx_id: TxId::new(4),
        key: b"k4".to_vec(),
        value: b"v4".to_vec(),
    };
    let (batch4, _) = wal.prepare_batch(vec![(op4, 3)]).await?;
    wal.append_batch(batch4).await?;
    drop(wal);

    let wal2 = Wal::open_with_config(&path, config).await?;
    let replayed = wal2.replay().await?;
    assert_eq!(replayed.len(), 3);
    assert_eq!(replayed[0].1.seq_no, 1);
    assert_eq!(replayed[1].1.seq_no, 2);
    assert_eq!(replayed[2].1.seq_no, 3);

    Ok(())
}

#[tokio::test]
async fn test_integration_middle_corruption_rejects_open() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("integration_corrupt.wal");

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

    let mut raw = fs::read(&path).await?;
    let orig_len = raw.len();
    raw[15] ^= 0xAA;
    fs::write(&path, &raw).await?;

    let open_res = Wal::open(&path).await;
    assert!(open_res.is_err());

    let raw_after = fs::read(&path).await?;
    assert_eq!(raw_after.len(), orig_len);
    assert_eq!(raw_after, raw);

    Ok(())
}
