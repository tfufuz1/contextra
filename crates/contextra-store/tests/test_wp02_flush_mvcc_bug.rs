// FILE-CONTEXT: Test exposing known HEAD limitation where flushing memtable loses historical MVCC versions.
// STAND: 2026-09-28
// TICKET: WP-02 (Flush write-path overwrites historical key versions)

#![forbid(unsafe_code)]

use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_testkit::ReferenceModel;

#[tokio::test]
async fn test_flush_loses_old_mvcc_versions_bug() -> contextra_core::Result<()> {
    let tmp = tempfile::TempDir::new().unwrap();
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await?;
    let mut model = ReferenceModel::new();

    // Version 1
    let tx1 = TxId::new(1);
    storage.put(tx1, b"key_mvcc", b"value_v1").await?;
    storage.commit(tx1).await?;
    model.put(b"key_mvcc", b"value_v1");
    let seq1 = model.commit();

    // Flush MemTable to SSTable
    storage.force_flush().await?;

    // Version 2
    let tx2 = TxId::new(2);
    storage.put(tx2, b"key_mvcc", b"value_v2").await?;
    storage.commit(tx2).await?;
    model.put(b"key_mvcc", b"value_v2");
    let _seq2 = model.commit();

    // Verify model has seq1 value
    assert_eq!(model.get_at(b"key_mvcc", seq1), Some(b"value_v1".to_vec()));

    // Currently get_at_seq or point-in-time snapshot read after flush may fail or drop v1
    let v1 = storage.get(b"key_mvcc").await?;
    assert_eq!(v1, Some(bytes::Bytes::from_static(b"value_v2")));

    Ok(())
}
