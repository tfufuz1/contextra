// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integrationstests für INV-WAL-LEGACY-KEY-1 (Legacy HMAC Key Isolation) sowie Replay/Batch/Truncation-Schutz.

use contextra_core::{Result, TxId};
use contextra_store::wal::{Wal, WalConfig, WalOp, WalVersion};
use tempfile::tempdir;

#[tokio::test]
async fn test_inv_wal_legacy_key_never_implicitly_active() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("wal_legacy.log");

    // Standard WalConfig default settings must have min_wal_version V3.
    let config = WalConfig::default();
    assert_eq!(config.min_wal_version, WalVersion::V3);

    // Create a legacy V1 WAL file manually
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&wal_path)?;
        f.write_all(b"WAL1_legacy_header_and_data")?;
        f.sync_all()?;
    }

    // Attempting to open a legacy file with default configuration must fail closed
    let open_res = Wal::open_with_config(&wal_path, config).await;
    assert!(
        open_res.is_err(),
        "Opening legacy WAL with default configuration must fail"
    );

    let err_msg = open_res.unwrap_err().to_string();
    assert!(
        err_msg.contains("disallowed") || err_msg.contains("migration required"),
        "Error message should indicate legacy WAL version disallowed: {}",
        err_msg
    );

    Ok(())
}

#[tokio::test]
async fn test_hmac_chain_across_batch_boundaries() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("wal_chain.log");

    let wal = Wal::open_with_config(&wal_path, WalConfig::default()).await?;

    // Append batch 1
    let ops1 = vec![
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k1".to_vec(),
                value: b"v1".to_vec(),
            },
            1,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(2),
                key: b"k2".to_vec(),
                value: b"v2".to_vec(),
            },
            2,
        ),
    ];
    let (batch1, _) = wal.prepare_batch(ops1).await?;
    wal.append_batch(batch1).await?;

    // Append batch 2
    let ops2 = vec![
        (
            WalOp::Put {
                tx_id: TxId::new(3),
                key: b"k3".to_vec(),
                value: b"v3".to_vec(),
            },
            3,
        ),
        (
            WalOp::Delete {
                tx_id: TxId::new(4),
                key: b"k1".to_vec(),
            },
            4,
        ),
    ];
    let (batch2, _) = wal.prepare_batch(ops2).await?;
    wal.append_batch(batch2).await?;

    wal.close().await?;

    // Replay and verify HMAC chain continuity across batch boundaries
    let wal_reopened = Wal::open_with_config(&wal_path, WalConfig::default()).await?;
    let entries = wal_reopened.replay().await?;
    assert_eq!(entries.len(), 4);

    let mut prev_hmac = [0u8; 32];
    for (_seq, entry, _pos) in &entries {
        #[cfg(feature = "wal-integrity")]
        {
            assert_eq!(
                entry.prev_hmac, prev_hmac,
                "HMAC chain prev_hmac mismatch across batches"
            );
            prev_hmac = entry.checksum;
        }
    }

    wal_reopened.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_half_written_batch_truncation_on_open() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("wal_half_written.log");

    let wal = Wal::open_with_config(&wal_path, WalConfig::default()).await?;
    let op = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"valid_key".to_vec(),
        value: b"valid_val".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 10)]).await?;
    wal.append_batch(batch).await?;
    wal.close().await?;

    // Append garbage / partial frame bytes to end of file
    let file_len = std::fs::metadata(&wal_path)?.len();
    {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new().append(true).open(&wal_path)?;
        file.write_all(b"\x00\x00\x00\x20incomplete_garbage_data_bytes_at_tail")?;
        file.sync_all()?;
    }
    let len_after_garbage = std::fs::metadata(&wal_path)?.len();
    assert!(len_after_garbage > file_len);

    // Opening WAL must detect partial tail, truncate back to verified_end (file_len)
    let wal_reopened = Wal::open_with_config(&wal_path, WalConfig::default()).await?;
    let entries = wal_reopened.replay().await?;
    assert_eq!(entries.len(), 1);

    let len_after_truncate = std::fs::metadata(&wal_path)?.len();
    assert_eq!(
        len_after_truncate, file_len,
        "Partial tail garbage must be truncated"
    );

    wal_reopened.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_truncation_detection_against_hwm() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("wal_hwm.log");

    let wal = Wal::open_with_config(&wal_path, WalConfig::default()).await?;
    let op1 = WalOp::Put {
        tx_id: TxId::new(100),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 100)]).await?;
    wal.append_batch(batch1).await?;

    let op2 = WalOp::Put {
        tx_id: TxId::new(101),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 101)]).await?;
    wal.append_batch(batch2).await?;

    let entries = wal.replay().await?;
    assert_eq!(entries.len(), 2);
    let hwm_hmac = entries[1].1.checksum;
    wal.close().await?;

    // Case 1: expected_hwm is set to the valid second entry's checksum -> Open succeeds
    let config_valid_hwm = WalConfig::default().with_expected_hwm(Some(hwm_hmac));
    let wal_valid = Wal::open_with_config(&wal_path, config_valid_hwm).await?;
    assert_eq!(wal_valid.replay().await?.len(), 2);
    wal_valid.close().await?;

    // Case 2: expected_hwm is set to a dummy unknown checksum -> Open fails with wal_truncation_detected
    let dummy_hwm = [0xAA; 32];
    let config_invalid_hwm = WalConfig::default().with_expected_hwm(Some(dummy_hwm));
    let open_res = Wal::open_with_config(&wal_path, config_invalid_hwm).await;
    assert!(open_res.is_err());
    let err_str = open_res.unwrap_err().to_string();
    assert!(
        err_str.contains("Truncation") || err_str.contains("truncation"),
        "Error message should report WAL truncation detection: {}",
        err_str
    );

    Ok(())
}
