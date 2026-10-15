//! Segment Retirement (C5.5) Witness Integration Tests.
//!
//! Verifies that `retire_segments_up_to` only removes segments with max_tx <= checkpoint_tx,
//! verifies that rotation alone does not delete files, and tests crash consistency.

use contextra_core::{ContextraError, TxId};
use contextra_store::wal::{retire_segments_up_to, Wal, WalOp};
use tempfile::tempdir;

#[tokio::test]
async fn test_wal_retire_segments_basic() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;

    // Segment 1 (sealed): max tx = 10
    let seg1_path = dir.path().join("wal_seg1.wal");
    {
        let wal1 = Wal::open(&seg1_path).await?;
        let op1 = WalOp::Put {
            tx_id: TxId::new(10),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        };
        let (batch1, _) = wal1.prepare_batch(vec![(op1, 1)]).await?;
        wal1.append_batch(batch1).await?;
        wal1.rotate_and_seal().await?;
    }

    // Segment 2 (sealed): max tx = 25
    let seg2_path = dir.path().join("wal_seg2.wal");
    {
        let wal2 = Wal::open(&seg2_path).await?;
        let op2 = WalOp::Put {
            tx_id: TxId::new(25),
            key: b"k2".to_vec(),
            value: b"v2".to_vec(),
        };
        let (batch2, _) = wal2.prepare_batch(vec![(op2, 2)]).await?;
        wal2.append_batch(batch2).await?;
        wal2.rotate_and_seal().await?;
    }

    // Retire up to checkpoint_tx = 15
    let report = retire_segments_up_to(dir.path(), 15).await?;

    if report.segments_removed != 1 || report.bytes_removed == 0 {
        return Err(ContextraError::Storage("retire report mismatch".into()));
    }

    // Segment 1 should be gone, Segment 2 should remain
    let remaining_files: Vec<String> = std::fs::read_dir(dir.path())
        .map_err(|e| ContextraError::Storage(e.to_string()))?
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().to_string()))
        .collect();

    if remaining_files.iter().any(|f| f.contains("wal_seg1")) {
        return Err(ContextraError::Storage("wal_seg1 was not removed".into()));
    }
    if !remaining_files.iter().any(|f| f.contains("wal_seg2")) {
        return Err(ContextraError::Storage(
            "wal_seg2 was unexpectedly removed".into(),
        ));
    }

    Ok(())
}

#[tokio::test]
async fn test_wal_rotation_is_not_deletion() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let seg_path = dir.path().join("wal_rotate.wal");

    let wal = Wal::open(&seg_path).await?;
    let op = WalOp::Put {
        tx_id: TxId::new(5),
        key: b"k_rot".to_vec(),
        value: b"v_rot".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.append_batch(batch).await?;

    // Rotate and seal
    let sealed_path = wal.rotate_and_seal().await?;

    // Rotation alone MUST NOT delete the file
    if !sealed_path.exists() {
        return Err(ContextraError::Storage(
            "sealed path missing after rotation".into(),
        ));
    }

    // Now call retire_segments_up_to(checkpoint_tx = 10)
    let report = retire_segments_up_to(dir.path(), 10).await?;
    if report.segments_removed != 1 || sealed_path.exists() {
        return Err(ContextraError::Storage(
            "sealed path was not retired".into(),
        ));
    }

    Ok(())
}

#[tokio::test]
async fn test_wal_retire_crash_consistency() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;

    // Active segment
    let seg_path = dir.path().join("crash_test.wal");
    let wal = Wal::open(&seg_path).await?;
    let op = WalOp::Put {
        tx_id: TxId::new(5),
        key: b"crash_k".to_vec(),
        value: b"crash_v".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 1)]).await?;
    wal.append_batch(batch).await?;

    // Simulate crash after rotation (sealed segment created on disk) before durable_remove
    let sealed_path = wal.rotate_and_seal().await?;
    if !sealed_path.exists() {
        return Err(ContextraError::Storage("sealed path missing".into()));
    }

    // Replay against sealed segment
    let wal_sealed = Wal::open_read_only(&sealed_path, None).await?;
    let entries = wal_sealed.replay().await?;
    if entries.len() != 1 {
        return Err(ContextraError::Storage("replay entries mismatch".into()));
    }

    Ok(())
}
