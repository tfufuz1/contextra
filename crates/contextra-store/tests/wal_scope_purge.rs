//! FILE-CONTEXT:
//! ZWECK: Integrationstests fuer WAL Scope Purge (`purge_wal_scope`) und WAL-Witness Scanner.

use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::wal::{
    purge_wal_scope, scan_wal_directory, Wal, WalOp, WalScanInputs, WalScopeSpec,
    FAILPOINT_AFTER_RENAME, FAILPOINT_AFTER_REWRITE, FAILPOINT_BEFORE_REWRITE,
    WAL_PURGE_FAILPOINT,
};
use contextra_types::{ContextraError, Result, TxId};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_purge_canary_key_removed_and_witness_clean() -> Result<()> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_path = dir.path().join("segment1.wal");

    let wal = Wal::open(&wal_path).await?;
    let op1 = WalOp::Put {
        tx_id: TxId(10),
        key: b"keep_1".to_vec(),
        value: b"val1".to_vec(),
    };
    let op2 = WalOp::Put {
        tx_id: TxId(15),
        key: b"canary_secret_key".to_vec(),
        value: b"canary_value".to_vec(),
    };
    let op3 = WalOp::Put {
        tx_id: TxId(20),
        key: b"keep_2".to_vec(),
        value: b"val2".to_vec(),
    };

    let (batch, _) = wal.prepare_batch(vec![(op1, 1), (op2, 2), (op3, 3)]).await?;
    wal.append_batch(batch).await?;
    wal.close().await?;

    let inputs_before = WalScanInputs {
        wal_dir: dir.path().to_path_buf(),
        as_of_tx: 100,
        encrypted_mode: false,
        timeout: Duration::from_secs(5),
        scope: WalScopeSpec {
            key_prefixes: vec![b"canary_".to_vec()],
            raw_needles: vec![b"canary_secret_key".to_vec()],
        },
        key_identifiers_claimed: true,
    };

    let evidence_before = scan_wal_directory(&inputs_before).await?;
    assert_eq!(evidence_before.frames_matching_scope, 1);
    assert!(!evidence_before.is_valid());

    let scope = PurgeScope::Prefixes(vec![b"canary_".to_vec()]);
    let report = purge_wal_scope(dir.path(), &scope, 100, None).await?;
    assert_eq!(report.segments_rewritten, 1);
    assert_eq!(report.frames_purged, 1);

    let evidence_after = scan_wal_directory(&inputs_before).await?;
    assert_eq!(evidence_after.frames_matching_scope, 0);
    assert!(evidence_after.is_valid());

    // Verify remaining data is readable and HMAC chain is valid
    let wal_after = Wal::open_read_only(&wal_path, None).await?;
    let entries = wal_after.replay().await?;
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].1.op.tx_id().inner(), 10);
    assert_eq!(entries[1].1.op.tx_id().inner(), 20);

    Ok(())
}

#[tokio::test]
async fn test_purge_crash_failpoints_and_resume() -> Result<()> {
    let failpoints = [
        FAILPOINT_BEFORE_REWRITE,
        FAILPOINT_AFTER_REWRITE,
        FAILPOINT_AFTER_RENAME,
    ];

    for fp in failpoints {
        let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
        let wal_path = dir.path().join("seg.wal");

        let wal = Wal::open(&wal_path).await?;
        let op1 = WalOp::Put {
            tx_id: TxId(5),
            key: b"canary_key".to_vec(),
            value: b"val1".to_vec(),
        };
        let op2 = WalOp::Put {
            tx_id: TxId(15),
            key: b"retained_key".to_vec(),
            value: b"val2".to_vec(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op1, 1), (op2, 2)]).await?;
        wal.append_batch(batch).await?;
        wal.close().await?;

        WAL_PURGE_FAILPOINT.store(fp, Ordering::SeqCst);
        let scope = PurgeScope::Keys(vec![b"canary_key".to_vec()]);

        let res = purge_wal_scope(dir.path(), &scope, 10, None).await;
        assert!(res.is_err(), "Failpoint {} should trigger error", fp);

        // Resume after crash
        let report = purge_wal_scope(dir.path(), &scope, 10, None).await?;
        assert!(report.segments_evaluated >= 1);

        let inputs = WalScanInputs {
            wal_dir: dir.path().to_path_buf(),
            as_of_tx: 100,
            encrypted_mode: false,
            timeout: Duration::from_secs(5),
            scope: WalScopeSpec {
                key_prefixes: vec![b"canary_key".to_vec()],
                raw_needles: vec![],
            },
            key_identifiers_claimed: true,
        };

        let evidence = scan_wal_directory(&inputs).await?;
        assert_eq!(evidence.frames_matching_scope, 0);
        assert!(evidence.is_valid());
    }

    Ok(())
}

#[tokio::test]
async fn test_recycled_segment_with_tail_data_cleared() -> Result<()> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_path = dir.path().join("segment_recycled.wal");

    let wal = Wal::open(&wal_path).await?;
    let op = WalOp::Put {
        tx_id: TxId(10),
        key: b"active_key".to_vec(),
        value: b"active_val".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.append_batch(batch).await?;
    wal.close().await?;

    // Append recycled garbage / old data behind logical tail
    let mut bytes = tokio::fs::read(&wal_path)
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;
    bytes.extend_from_slice(b"---RECYCLED_GARBAGE_TAIL_NEEDLE---");
    tokio::fs::write(&wal_path, &bytes)
        .await
        .map_err(|e| ContextraError::Storage(e.to_string()))?;

    let inputs = WalScanInputs {
        wal_dir: dir.path().to_path_buf(),
        as_of_tx: 100,
        encrypted_mode: false,
        timeout: Duration::from_secs(5),
        scope: WalScopeSpec {
            key_prefixes: vec![b"non_existent_prefix".to_vec()],
            raw_needles: vec![b"RECYCLED_GARBAGE_TAIL_NEEDLE".to_vec()],
        },
        key_identifiers_claimed: false,
    };

    let evidence_before = scan_wal_directory(&inputs).await?;
    assert!(evidence_before.tail_bytes_raw_scanned > 0);
    assert_eq!(evidence_before.frames_matching_scope, 1);

    let scope = PurgeScope::Keys(vec![b"non_existent".to_vec()]);
    let report = purge_wal_scope(dir.path(), &scope, 100, None).await?;
    assert!(report.tail_bytes_cleaned > 0);

    let evidence_after = scan_wal_directory(&inputs).await?;
    assert_eq!(evidence_after.frames_matching_scope, 0);
    assert!(evidence_after.is_valid());

    Ok(())
}

#[tokio::test]
async fn test_replay_after_purge_does_not_resurrect_deleted_key() -> Result<()> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_path = dir.path().join("seg.wal");

    let wal = Wal::open(&wal_path).await?;
    let op1 = WalOp::Put {
        tx_id: TxId(5),
        key: b"resurrect_candidate".to_vec(),
        value: b"old_val".to_vec(),
    };
    let op2 = WalOp::Delete {
        tx_id: TxId(10),
        key: b"resurrect_candidate".to_vec(),
    };
    let op3 = WalOp::Put {
        tx_id: TxId(12),
        key: b"permanent_key".to_vec(),
        value: b"perm_val".to_vec(),
    };

    let (batch, _) = wal.prepare_batch(vec![(op1, 1), (op2, 2), (op3, 3)]).await?;
    wal.append_batch(batch).await?;
    wal.close().await?;

    let scope = PurgeScope::Keys(vec![b"resurrect_candidate".to_vec()]);
    purge_wal_scope(dir.path(), &scope, 15, None).await?;

    let wal_replayed = Wal::open_read_only(&wal_path, None).await?;
    let replayed = wal_replayed.replay().await?;

    assert_eq!(replayed.len(), 1);
    assert_eq!(replayed[0].1.op.tx_id().inner(), 12);
    if let WalOp::Put { key, value, .. } = &replayed[0].1.op {
        assert_eq!(key, b"permanent_key");
        assert_eq!(value, b"perm_val");
    } else {
        panic!("Expected Put operation for permanent_key");
    }

    Ok(())
}
