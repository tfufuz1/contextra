//! # MVCC und Praefix-Scan Tests (Ring 1)
//!
//! ZWECK: Pruefung von MVCC-Snapshot-Isolation und Praefix-Filterung.

use super::*;

#[tokio::test]
async fn test_get_nonexistent() {
    let (storage, _tmp) = test_storage().await;
    let val = storage.get(b"nonexistent").await.expect("get");
    assert_eq!(val, None);
}

#[tokio::test]
async fn test_get_records_read_op() {
    let (storage, _tmp) = test_storage().await;

    let res1 = storage.get(b"nonexistent_key").await.expect("get");
    assert_eq!(res1, None);

    let tx1 = TxId::new(1);
    storage.put(tx1, b"existing_key", b"value").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let res2 = storage.get(b"existing_key").await.expect("get");
    assert_eq!(res2, Some(bytes::Bytes::from_static(b"value")));
}

#[tokio::test]
async fn test_mvcc_snapshot_isolation() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"k1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();
    let seq1 = storage.last_seq_no().await.unwrap();

    let tx2 = TxId::new(2);
    storage.put(tx2, b"k1", b"v2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    let val1 = storage.get_at_seq(b"k1", seq1).await.unwrap();
    assert_eq!(val1, Some(bytes::Bytes::from_static(b"v1")));

    let val2 = storage.get(b"k1").await.unwrap();
    assert_eq!(val2, Some(bytes::Bytes::from_static(b"v2")));
}

#[tokio::test]
async fn test_scan_prefix_at_uncommitted_isolation() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"doc:1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();
    let seq1 = storage.last_seq_no().await.unwrap();

    let tx2 = TxId::new(2);
    storage.put(tx2, b"doc:2", b"v2").await.unwrap();

    let res = storage.scan_prefix_at(b"doc:", seq1).await.unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].0, b"doc:1");
}

#[tokio::test]
async fn test_get_at_seq_mvcc_sequence_correctness() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();
    let seq1 = storage.last_seq_no().await.unwrap();

    let tx2 = TxId::new(2);
    storage.put(tx2, b"key1", b"v2").await.unwrap();
    storage.commit(tx2).await.unwrap();
    let seq2 = storage.last_seq_no().await.unwrap();

    let tx3 = TxId::new(3);
    storage.delete(tx3, b"key1").await.unwrap();
    storage.commit(tx3).await.unwrap();
    let seq3 = storage.last_seq_no().await.unwrap();

    assert_eq!(
        storage.get_at_seq(b"key1", seq1).await.unwrap(),
        Some(bytes::Bytes::from_static(b"v1"))
    );
    assert_eq!(
        storage.get_at_seq(b"key1", seq2).await.unwrap(),
        Some(bytes::Bytes::from_static(b"v2"))
    );
    assert_eq!(storage.get_at_seq(b"key1", seq3).await.unwrap(), None);
}

#[tokio::test]
async fn test_scan_prefix_memtable_shadows_sstable() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"pfx:a", b"old").await.unwrap();
    storage.commit(tx1).await.unwrap();
    storage.force_flush().await.unwrap();

    let stats = storage.stats().await.unwrap();
    assert!(stats.num_segments > 0, "SSTable segment must exist");

    let tx2 = TxId::new(2);
    storage.put(tx2, b"pfx:a", b"new").await.unwrap();
    storage.commit(tx2).await.unwrap();

    let results = storage.scan_prefix(b"pfx:").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, b"pfx:a");
    assert_eq!(results[0].1, b"new");
}

#[tokio::test]
async fn test_scan_prefix_at_snapshot_isolation() {
    let (storage, _tmp) = test_storage().await;
    let tx1 = TxId::new(1);
    storage.put(tx1, b"col:doc1", b"v1").await.unwrap();
    storage.commit(tx1).await.unwrap();
    let seq_after_tx1 = storage.last_seq_no().await.unwrap();

    let tx2 = TxId::new(2);
    storage.put(tx2, b"col:doc2", b"v2").await.unwrap();
    storage.commit(tx2).await.unwrap();

    let results = storage
        .scan_prefix_at(b"col:", seq_after_tx1)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, b"col:doc1");
}

#[tokio::test]
async fn test_scan_prefix_at_mvcc_sequence_filtering() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"pfx:1", b"v1").await.unwrap();
    storage.put(tx1, b"pfx:2", b"v2").await.unwrap();
    storage.commit(tx1).await.unwrap();
    let seq1 = storage.last_seq_no().await.unwrap();

    let tx2 = TxId::new(2);
    storage.put(tx2, b"pfx:1", b"v1_new").await.unwrap();
    storage.delete(tx2, b"pfx:2").await.unwrap();
    storage.commit(tx2).await.unwrap();
    let seq2 = storage.last_seq_no().await.unwrap();

    let tx3 = TxId::new(3);
    storage.put(tx3, b"pfx:3", b"v3").await.unwrap();
    storage.commit(tx3).await.unwrap();

    let res_seq1 = storage.scan_prefix_at(b"pfx:", seq1).await.unwrap();
    assert_eq!(res_seq1.len(), 2);
    let map1: std::collections::HashMap<_, _> = res_seq1.into_iter().collect();
    assert_eq!(map1.get(&b"pfx:1"[..]), Some(&b"v1"[..].to_vec()));
    assert_eq!(map1.get(&b"pfx:2"[..]), Some(&b"v2"[..].to_vec()));

    let res_seq2 = storage.scan_prefix_at(b"pfx:", seq2).await.unwrap();
    assert_eq!(res_seq2.len(), 1);
    assert_eq!(res_seq2[0].0, b"pfx:1");
    assert_eq!(res_seq2[0].1, b"v1_new");
}

#[tokio::test]
async fn test_scan_prefix_at_tombstone_isolation() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"pfx:a", b"val_a").await.unwrap();
    storage.commit(tx1).await.unwrap();
    let seq1 = storage.last_seq_no().await.unwrap();

    storage.force_flush().await.unwrap();

    let tx2 = TxId::new(2);
    storage.delete(tx2, b"pfx:a").await.unwrap();
    storage.commit(tx2).await.unwrap();
    let seq2 = storage.last_seq_no().await.unwrap();

    let res_seq1 = storage.scan_prefix_at(b"pfx:", seq1).await.unwrap();
    assert_eq!(res_seq1.len(), 1);
    assert_eq!(res_seq1[0].0, b"pfx:a");
    assert_eq!(res_seq1[0].1, b"val_a");

    let res_seq2 = storage.scan_prefix_at(b"pfx:", seq2).await.unwrap();
    assert!(res_seq2.is_empty());
}
