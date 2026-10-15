//! FILE-CONTEXT: STAND/ZWECK/INVARIANTEN
//! Stand: 2026-10-06
//! Zweck: P01-A WAL flusher fsync failure page-cache truncation and panic propagation regression tests.
//! Invarianten:
//! - I-2 (Atomarer Group-Commit): Ungefluste/unbestätigte Einträge dürfen nicht in der physischen Datei verbleiben.
//! - I-3 (Crash-Konsistenz): Nach fsync-Fehler muss die physische Datei auf size_before zurückgeschnitten sein.
//! - I-5 (Poison-State Isolation): Ungefluste Writes hinterlassen das WAL-Handle im poisoned-Zustand.

use contextra_core::{ContextraError, Result, TxId};
use contextra_store::wal::{Wal, WalOp};
use tempfile::tempdir;

#[tokio::test]
#[cfg_attr(not(feature = "fault-injection"), ignore)]
async fn test_wal_fsync_failure_truncates_page_cache() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("fsync_truncation.wal");

    let wal = Wal::open(&wal_path).await?;

    // Step 1: Write a valid confirmed entry
    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"confirmed_key".to_vec(),
        value: b"confirmed_val".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    wal.append_batch(batch1).await?;

    let size_before = wal.size();
    assert!(size_before > 0, "WAL size after batch 1 must be > 0");

    // Step 2: Trigger fsync failure on second batch append
    #[cfg(feature = "fault-injection")]
    contextra_store::wal::FAIL_SYNC_ONCE.store(true, std::sync::atomic::Ordering::SeqCst);

    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"unconfirmed_key".to_vec(),
        value: b"unconfirmed_val".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;
    let append_res = wal.append_batch(batch2).await;

    assert!(append_res.is_err(), "Append must fail when fsync fails");
    assert!(
        wal.is_poisoned(),
        "WAL handle must be poisoned after fsync failure"
    );

    // Step 3: Verify that the physical file size was truncated back to size_before
    let meta = tokio::fs::metadata(&wal_path).await?;
    assert_eq!(
        meta.len(),
        size_before,
        "Physical file length after failed fsync append must be truncated back to size_before ({size_before})"
    );

    // Step 4: Reopen the WAL file and verify that unconfirmed entry #2 is NOT present in replay
    wal.close().await?;
    let wal_reopened = Wal::open(&wal_path).await?;
    let replayed = wal_reopened.replay().await?;
    assert_eq!(
        replayed.len(),
        1,
        "Replayed entries must only contain confirmed entry #1 (unconfirmed entry #2 must NOT be resurrected)"
    );
    if let WalOp::Put { key, .. } = &replayed[0].1.op {
        assert_eq!(key, b"confirmed_key");
    } else {
        panic!("Expected Put operation");
    }

    Ok(())
}

#[tokio::test]
async fn test_close_propagates_flusher_panic() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("close_panic.wal");

    let wal = Wal::open(&wal_path).await?;

    // Abort the flusher task join handle via helper method
    wal.abort_flusher_for_test();

    let close_res = wal.close().await;
    assert!(
        close_res.is_err(),
        "close() must return Err when flusher task panicked or was aborted"
    );
    if let Err(err) = close_res {
        assert!(
            matches!(err, ContextraError::Internal(_)),
            "Error must be ContextraError::Internal, got: {err:?}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_open_with_config_truncates_garbage_tail() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("garbage_tail.wal");

    let wal = Wal::open(&wal_path).await?;
    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"valid_key".to_vec(),
        value: b"valid_val".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    wal.append_batch(batch1).await?;
    let valid_size = wal.size();
    wal.close().await?;

    // Append garbage / zeroes to the file
    let mut file_bytes = tokio::fs::read(&wal_path).await?;
    file_bytes.extend_from_slice(&[0xFF, 0x00, 0xDE, 0xAD, 0xBE, 0xEF]);
    tokio::fs::write(&wal_path, &file_bytes).await?;

    // Reopen with open_with_config
    let wal_reopened = Wal::open(&wal_path).await?;
    assert_eq!(
        wal_reopened.size(),
        valid_size,
        "WAL size on reopen must be truncated back to the last HMAC-verified frame position"
    );

    let replayed = wal_reopened.replay().await?;
    assert_eq!(replayed.len(), 1);

    Ok(())
}
