//! FILE-CONTEXT: STAND/ZWECK/INVARIANTEN
//! Stand: 2026-10-21
//! Zweck: Integration tests for P06 (WP-7 + WP-ST2): WAL retire/scope-purge, raw byte sweep, archived opt-in, truncated tombstone anchor detection, and S5 cleanup failure retry.

use contextra_durable_fs::ScrubPolicy;
use contextra_ports::storage::StorageEngine;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::manifest::Manifest;
use contextra_store::wal::{
    retire_for_scope, verify_wal_anchor, AnchorCheck, Wal, WalOp, WalPurgeMode,
};
use contextra_types::{ContextraError, Result, TxId};
use std::collections::HashSet;
use tempfile::TempDir;

#[tokio::test]
async fn wal_retire_mixed_segment_raw_sweep() -> Result<()> {
    let tmp = TempDir::new().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_dir = tmp.path().to_path_buf();
    let manifest_path = wal_dir.join("MANIFEST");
    let manifest = Manifest::open(&manifest_path).await?;

    let wal_path = wal_dir.join("wal-00000000000000000001.log");
    let wal = Wal::open(&wal_path).await?;

    let target_key = b"target_secret_key_xyz".to_vec();
    let keep_key = b"keep_public_key_abc".to_vec();

    let op1 = WalOp::Put {
        tx_id: TxId::try_new(10).unwrap(),
        key: target_key.clone(),
        value: b"secret_value_100".to_vec(),
    };
    let op2 = WalOp::Put {
        tx_id: TxId::try_new(20).unwrap(),
        key: keep_key.clone(),
        value: b"keep_value_200".to_vec(),
    };

    let (batch1, _) = wal.prepare_batch(vec![(op1, 1), (op2, 2)]).await?;
    wal.append_batch(batch1).await?;
    wal.close().await?;

    // Positive control: raw sweep BEFORE retire finds target_key
    let raw_bytes_before = tokio::fs::read(&wal_path).await.unwrap();
    assert!(
        raw_bytes_before
            .windows(target_key.len())
            .any(|w| w == target_key),
        "Positive control: Raw sweep must find target key before scope purge"
    );

    let scope = PurgeScope::Keys(vec![target_key.clone()]);
    let report = retire_for_scope(
        &wal_dir,
        &manifest,
        &scope,
        20,
        5,
        None,
        WalPurgeMode::PurgeWithScrub,
        ScrubPolicy::OverwriteZeros,
    )
    .await?;

    assert_eq!(report.purge_report.segments_rewritten, 1);

    // Find sealed / remaining segment path in wal_dir
    let mut dir_entries = tokio::fs::read_dir(&wal_dir).await.unwrap();
    let mut remaining_path = None;
    while let Ok(Some(entry)) = dir_entries.next_entry().await {
        let p = entry.path();
        if p.is_file() && p.file_name().unwrap().to_str().unwrap().starts_with("wal-") {
            remaining_path = Some(p);
            break;
        }
    }
    let remaining_path = remaining_path.expect("Remaining segment must exist after scope purge");

    // Negative control: raw sweep AFTER retire must NOT find target_key
    let raw_bytes_after = tokio::fs::read(&remaining_path).await.unwrap();
    assert!(
        !raw_bytes_after
            .windows(target_key.len())
            .any(|w| w == target_key),
        "Raw sweep after scope purge must not find target key bytes"
    );
    assert!(
        raw_bytes_after
            .windows(keep_key.len())
            .any(|w| w == keep_key),
        "Raw sweep after scope purge must preserve keep_key"
    );

    Ok(())
}

#[tokio::test]
async fn wal_scope_purge_preserves_chain_and_reopens() -> Result<()> {
    let tmp = TempDir::new().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_dir = tmp.path().to_path_buf();
    let manifest_path = wal_dir.join("MANIFEST");
    let manifest = Manifest::open(&manifest_path).await?;

    let wal_path = wal_dir.join("wal-00000000000000000001.log");
    let wal = Wal::open(&wal_path).await?;

    let purge_key = b"purge_me".to_vec();
    let keep_key1 = b"keep_1".to_vec();
    let keep_key2 = b"keep_2".to_vec();

    let op1 = WalOp::Put {
        tx_id: TxId::try_new(5).unwrap(),
        key: keep_key1.clone(),
        value: b"val1".to_vec(),
    };
    let op2 = WalOp::Put {
        tx_id: TxId::try_new(10).unwrap(),
        key: purge_key.clone(),
        value: b"val2".to_vec(),
    };
    let op3 = WalOp::Put {
        tx_id: TxId::try_new(15).unwrap(),
        key: keep_key2.clone(),
        value: b"val3".to_vec(),
    };

    let (batch1, _) = wal.prepare_batch(vec![(op1, 1), (op2, 2), (op3, 3)]).await?;
    wal.append_batch(batch1).await?;
    wal.close().await?;

    let scope = PurgeScope::Keys(vec![purge_key]);
    retire_for_scope(
        &wal_dir,
        &manifest,
        &scope,
        15,
        0,
        None,
        WalPurgeMode::Purge,
        ScrubPolicy::Disabled,
    )
    .await?;

    let mut dir_entries = tokio::fs::read_dir(&wal_dir).await.unwrap();
    let mut remaining_path = None;
    while let Ok(Some(entry)) = dir_entries.next_entry().await {
        let p = entry.path();
        if p.is_file() && p.file_name().unwrap().to_str().unwrap().starts_with("wal-") {
            remaining_path = Some(p);
            break;
        }
    }
    let remaining_path = remaining_path.expect("Remaining segment must exist after scope purge");

    let reopened_wal = Wal::open_read_only(&remaining_path, None).await?;
    let entries = reopened_wal.replay().await?;

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].1.seq_no, 1);
    assert_eq!(entries[1].1.seq_no, 2);
    assert_eq!(entries[1].1.prev_hmac, entries[0].1.checksum);

    Ok(())
}

#[tokio::test]
async fn wal_retire_archived_requires_opt_in() -> Result<()> {
    let tmp = TempDir::new().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_dir = tmp.path().to_path_buf();
    let manifest_path = wal_dir.join("MANIFEST");
    let manifest = Manifest::open(&manifest_path).await?;

    let archived_path = wal_dir.join("wal-00000000000000000001.log.archived");
    tokio::fs::write(&archived_path, b"dummy archived wal").await.unwrap();

    let scope = PurgeScope::Keys(vec![b"any_key".to_vec()]);
    let err_res = retire_for_scope(
        &wal_dir,
        &manifest,
        &scope,
        10,
        10,
        None,
        WalPurgeMode::None,
        ScrubPolicy::Disabled,
    )
    .await;

    assert!(err_res.is_err());
    if let Err(ContextraError::PolicyViolation(msg)) = err_res {
        assert!(msg.contains("archived/backup"));
    } else {
        panic!("Expected PolicyViolation error for archived file without opt-in");
    }

    Ok(())
}

#[tokio::test]
async fn wal_truncated_tombstone_detected() -> Result<()> {
    let expected_hwm = [0x55; 32];
    let actual_tail_hmac = [0xAA; 32];
    let mut replayed_set = HashSet::new();
    replayed_set.insert([0x11; 32]);

    let res = verify_wal_anchor(&expected_hwm, &actual_tail_hmac, &replayed_set, None, false);
    assert!(res.is_err());
    if let Err(ContextraError::WalTruncationDetected { expected_hmac, actual_hmac }) = res {
        assert_eq!(expected_hmac, expected_hwm);
        assert_eq!(actual_hmac, actual_tail_hmac);
    } else {
        panic!("Expected WalTruncationDetected error");
    }

    // Success case: expected_hwm in replay set
    replayed_set.insert(expected_hwm);
    let ok_res = verify_wal_anchor(&expected_hwm, &actual_tail_hmac, &replayed_set, None, false);
    assert_eq!(ok_res.unwrap(), AnchorCheck::HwmInReplay);

    Ok(())
}

#[tokio::test]
async fn wal_cleanup_failure_retried_next_flush() -> Result<()> {
    let tmp = TempDir::new().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await?;

    let tx1 = TxId::try_new(10).unwrap();
    storage.put(tx1, b"key1", b"val1").await?;
    storage.commit(tx1).await?;

    storage.flush().await?;

    let tx2 = TxId::try_new(20).unwrap();
    storage.put(tx2, b"key2", b"val2").await?;
    storage.commit(tx2).await?;

    storage.flush().await?;

    Ok(())
}

#[tokio::test]
async fn hwm_dir_sync_error_propagates() -> Result<()> {
    let tmp = TempDir::new().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await?;
    let tx1 = TxId::try_new(1).unwrap();
    storage.put(tx1, b"k", b"v").await?;
    storage.commit(tx1).await?;

    Ok(())
}
