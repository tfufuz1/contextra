// FILE-CONTEXT
// STAND: 2026-10-07
// ZWECK: P02 Acceptance Tests — Mandatory Arc<dyn SnapshotFloor> for CompactionEngine and MemTable Flush.
// INVARIANTEN: GC Floor must protect active readers during Compaction and Flush while preserving Tombstone purging when no readers exist.

#![forbid(unsafe_code)]

use contextra_core::{StorageEngine, TxId};
use contextra_store::compaction::CompactionConfig;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::TempDir;

#[tokio::test]
async fn test_compaction_with_reader_at_seq_80_retains_put_and_tombstone() {
    let tmp = TempDir::new().expect("tempdir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        compaction: CompactionConfig {
            min_sstables_per_tier: 2,
            enable_adaptive_compaction: false,
            ..Default::default()
        },
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await.expect("new storage");

    // Tx1: Put key1 = val50
    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val50").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let seq1 = storage.last_seq_no().await.unwrap();

    // Flush Put to SSTable 1
    storage.force_flush().await.unwrap();

    // Tx2: Delete key1 (Tombstone)
    let tx2 = TxId::new(2);
    storage.delete(tx2, b"key1").await.unwrap();
    storage.commit(tx2).await.unwrap();

    // Flush Tombstone to SSTable 2
    storage.force_flush().await.unwrap();

    // Active reader holding a snapshot lease at seq1 (before the tombstone)
    let lease = storage.snapshot_registry.acquire(seq1);

    // Trigger compaction (merges SSTable 1 and SSTable 2)
    let compacted = storage.maybe_compact().await.unwrap();
    assert!(compacted, "Compaction should execute");

    // Reader with lease at seq1 must still see val50
    let val_at_reader = storage.get_at_seq(b"key1", seq1).await.unwrap();
    assert_eq!(
        val_at_reader,
        Some(bytes::Bytes::from_static(b"val50")),
        "Value at seq1 must remain visible to active reader after compaction"
    );

    // Query without snapshot (at current HEAD) should see None (deleted by tombstone)
    let val_at_head = storage.get(b"key1").await.unwrap();
    assert_eq!(val_at_head, None, "HEAD query must return None due to tombstone");

    drop(lease);
}

#[tokio::test]
async fn test_flush_with_reader_retains_version_below_tombstone() {
    let tmp = TempDir::new().expect("tempdir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await.expect("new storage");

    // Tx1: Put key1 = val_old
    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val_old").await.unwrap();
    storage.commit(tx1).await.unwrap();

    // Acquire lease at old sequence number before tombstone
    let old_seq = storage.last_seq_no().await.unwrap();
    let lease = storage.snapshot_registry.acquire(old_seq);

    // Tx2: Delete key1 in MemTable
    let tx2 = TxId::new(2);
    storage.delete(tx2, b"key1").await.unwrap();
    storage.commit(tx2).await.unwrap();

    // Flush MemTable to SSTable while reader lease is held
    storage.force_flush().await.unwrap();

    // Active reader must still be able to query old_seq after flush
    let val_at_reader = storage.get_at_seq(b"key1", old_seq).await.unwrap();
    assert_eq!(
        val_at_reader,
        Some(bytes::Bytes::from_static(b"val_old")),
        "Value at old_seq must remain visible after MemTable flush"
    );

    drop(lease);
}

#[tokio::test]
async fn test_compaction_without_reader_purges_tombstone() {
    let tmp = TempDir::new().expect("tempdir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        compaction: CompactionConfig {
            min_sstables_per_tier: 2,
            enable_adaptive_compaction: false,
            ..Default::default()
        },
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await.expect("new storage");

    // Tx1: Put key1 = val1
    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val1").await.unwrap();
    storage.commit(tx1).await.unwrap();
    storage.force_flush().await.unwrap();

    // Tx2: Delete key1
    let tx2 = TxId::new(2);
    storage.delete(tx2, b"key1").await.unwrap();
    storage.commit(tx2).await.unwrap();
    storage.force_flush().await.unwrap();

    // Ensure NO readers are active in snapshot registry
    assert_eq!(storage.snapshot_registry.min_active_seqno(), u64::MAX);

    // Trigger compaction
    let compacted = storage.maybe_compact().await.unwrap();
    assert!(compacted, "Compaction should execute");

    // At HEAD, key1 is deleted
    assert_eq!(storage.get(b"key1").await.unwrap(), None);
}
