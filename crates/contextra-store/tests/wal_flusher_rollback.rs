#![cfg(feature = "fault-injection")]

use contextra_core::TxId;
use contextra_store::wal::{
    Wal, WalOp, FAIL_APPEND_AFTER_PARTIAL_BYTES, FAIL_APPEND_PARTIAL_ONCE, FAIL_TRUNCATE_ONCE,
};
use tempfile::tempdir;

#[tokio::test]
async fn test_flusher_rollback_after_partial_append_failure() -> contextra_core::Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("flusher_rollback.wal");

    let wal = Wal::open(&wal_path).await?;

    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    wal.append_batch(batch1).await?;

    let meta_before = tokio::fs::metadata(&wal_path).await?;
    let size_before = meta_before.len();

    // Trigger partial write failure in flusher
    FAIL_APPEND_AFTER_PARTIAL_BYTES.store(20, std::sync::atomic::Ordering::SeqCst);
    FAIL_APPEND_PARTIAL_ONCE.store(true, std::sync::atomic::Ordering::SeqCst);

    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;
    let res = wal.append_batch(batch2).await;
    assert!(res.is_err(), "Partial append write must fail");

    // After flusher rollback, poisoned MUST be false and file size MUST equal size_before
    assert!(
        !wal.is_poisoned(),
        "WAL handle must NOT be poisoned after successful flusher rollback"
    );

    let meta_after = tokio::fs::metadata(&wal_path).await?;
    assert_eq!(
        meta_after.len(),
        size_before,
        "File length on disk must be restored to size_before after rollback"
    );

    // Subsequent append must succeed
    let op3 = WalOp::Put {
        tx_id: TxId::new(3),
        key: b"k3".to_vec(),
        value: b"v3".to_vec(),
    };
    let (batch3, _) = wal.prepare_batch(vec![(op3, 3)]).await?;
    wal.append_batch(batch3).await?;

    drop(wal);

    // Reopen and replay: must see op1 and op3 cleanly
    let wal_reopened = Wal::open(&wal_path).await?;
    let replayed = wal_reopened.replay().await?;
    assert_eq!(replayed.len(), 2);
    assert_eq!(replayed[0].1.seq_no, 1);
    assert_eq!(replayed[1].1.seq_no, 3);

    Ok(())
}

#[tokio::test]
async fn test_flusher_truncate_failure_preserves_state() -> contextra_core::Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("truncate_fail.wal");

    let wal = Wal::open(&wal_path).await?;

    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    wal.append_batch(batch1).await?;

    let size_before = wal.size();
    let hmac_before = wal.last_hmac_snapshot().await;

    FAIL_TRUNCATE_ONCE.store(true, std::sync::atomic::Ordering::SeqCst);

    let trunc_res = wal.truncate(4, [0u8; 32]).await;
    assert!(
        trunc_res.is_err(),
        "Truncate must fail when FAIL_TRUNCATE_ONCE is set"
    );

    // In-memory size and last_hmac must remain unchanged
    assert_eq!(
        wal.size(),
        size_before,
        "Size must remain unchanged on truncate failure"
    );
    assert_eq!(
        wal.last_hmac_snapshot().await,
        hmac_before,
        "HMAC must remain unchanged on truncate failure"
    );

    Ok(())
}
