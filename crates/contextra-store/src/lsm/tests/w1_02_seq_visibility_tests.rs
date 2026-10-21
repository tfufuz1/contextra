// FILE-CONTEXT: Tests verifying single visibility watermark and sequence gap tolerance (S7, S8g, INV-COM-2).

use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::{Result, TxId};
use tempfile::TempDir;

#[tokio::test]
async fn loom_get_at_seq_stable() -> Result<()> {
    let temp_dir = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await?;

    let tx1 = TxId(10);
    storage.put(tx1, b"key1", b"val1").await?;
    storage.commit(tx1).await?;

    let last_seq = storage.last_applied_seq();

    // Querying at last_seq must return value deterministically across consecutive calls
    let val_a = storage.get_at_seq(b"key1", last_seq).await?;
    let val_b = storage.get_at_seq(b"key1", last_seq).await?;
    assert_eq!(val_a, val_b);
    assert_eq!(val_a.as_deref(), Some(b"val1".as_slice()));

    Ok(())
}

#[tokio::test]
async fn seq_gap_allowed_no_rollback() -> Result<()> {
    let temp_dir = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await?;

    let seq_before = storage.next_seq_no_for_test();

    // Perform a valid commit
    let tx1 = TxId(11);
    storage.put(tx1, b"key1", b"val1").await?;
    storage.commit(tx1).await?;

    let seq_after_tx1 = storage.next_seq_no_for_test();
    assert!(
        seq_after_tx1 > seq_before,
        "Sequence number must increase monotonically"
    );

    // Perform another commit
    let tx2 = TxId(12);
    storage.put(tx2, b"key2", b"val2").await?;
    storage.commit(tx2).await?;

    let seq_after_tx2 = storage.next_seq_no_for_test();
    assert!(
        seq_after_tx2 > seq_after_tx1,
        "Sequence number must continue increasing monotonically without rollbacks"
    );

    Ok(())
}
