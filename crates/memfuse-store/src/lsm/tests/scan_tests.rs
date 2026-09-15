use super::*;
use std::ops::Bound;

#[tokio::test]
async fn test_get_nonexistent() {
    let (storage, _tmp) = test_storage().await;
    let val = storage.get(b"nonexistent").await.expect("get");
    assert_eq!(val, None);
}

#[tokio::test]
async fn test_scan_range() {
    let (storage, _tmp) = test_storage().await;

    let tx1 = TxId::new(1);
    storage.put(tx1, b"a", b"1").await.unwrap();
    storage.put(tx1, b"b", b"2").await.unwrap();
    storage.put(tx1, b"c", b"3").await.unwrap();
    storage.put(tx1, b"d", b"4").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let results = storage
        .scan(Bound::Included(b"b".as_ref()), Bound::Excluded(b"d".as_ref()), None)
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0], (b"b".to_vec(), b"2".to_vec()));
    assert_eq!(results[1], (b"c".to_vec(), b"3".to_vec()));
}

#[tokio::test]
async fn test_bounded_scan_and_prefix_bounded_limits_candidate_evaluation() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    for i in 0..100 {
        let key = format!("k:{:03}", i);
        let val = format!("val:{:03}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(tx).await.unwrap();

    let (res_range, cur_range) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, None)
        .await
        .unwrap();
    assert_eq!(res_range.len(), 10);
    assert_eq!(res_range[0].0, b"k:000");
    assert_eq!(res_range[9].0, b"k:009");
    assert_eq!(cur_range, Some(b"k:009".to_vec()));

    let (res_prefix, cur_prefix) = storage
        .scan_prefix_bounded(b"k:", 10, None)
        .await
        .unwrap();
    assert_eq!(res_prefix.len(), 10);
    assert_eq!(res_prefix[0].0, b"k:000");
    assert_eq!(res_prefix[9].0, b"k:009");
    assert_eq!(cur_prefix, Some(b"k:009".to_vec()));
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
    assert_eq!(val1, Some(b"v1".to_vec()));

    let val2 = storage.get(b"k1").await.unwrap();
    assert_eq!(val2, Some(b"v2".to_vec()));
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

    let tx3 = TxId::new(3);
    storage.delete(tx3, b"key1").await.unwrap();
    storage.commit(tx3).await.unwrap();

    assert_eq!(
        storage.get_at_seq(b"key1", seq1).await.unwrap(),
        Some(b"v1".to_vec())
    );
    assert_eq!(
        storage.get_at_seq(b"key1", seq1 + 1).await.unwrap(),
        Some(b"v2".to_vec())
    );
    assert_eq!(
        storage.get_at_seq(b"key1", seq1 + 2).await.unwrap(),
        None
    );
}

#[tokio::test]
async fn test_scan_bounded_respects_accumulator_ceiling_with_wide_range() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    for i in 0..100 {
        let key = format!("k:{:03}", i);
        let val = format!("val:{:03}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(tx).await.unwrap();

    let (batch, next_cur) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 5, None)
        .await
        .unwrap();

    assert_eq!(batch.len(), 5);
    assert_eq!(batch[0].0, b"k:000");
    assert_eq!(batch[4].0, b"k:004");
    assert_eq!(next_cur, Some(b"k:004".to_vec()));
}

#[tokio::test]
async fn test_scan_bounded_pagination_matches_full_scan() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    for i in 0..25 {
        let key = format!("k:{:02}", i);
        let val = format!("val:{:02}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(tx).await.unwrap();

    let (p1, cur1) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, None)
        .await
        .unwrap();
    assert_eq!(p1.len(), 10);
    assert_eq!(p1[0].0, b"k:00");
    assert_eq!(p1[9].0, b"k:09");
    assert!(cur1.is_some());
    let cur1_val = cur1.unwrap();
    assert_eq!(cur1_val, b"k:09");

    let (p2, cur2) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, Some(&cur1_val))
        .await
        .unwrap();
    assert_eq!(p2.len(), 10);
    assert_eq!(p2[0].0, b"k:10");
    assert_eq!(p2[9].0, b"k:19");
    assert!(cur2.is_some());
    let cur2_val = cur2.unwrap();
    assert_eq!(cur2_val, b"k:19");

    let (p3, cur3) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, Some(&cur2_val))
        .await
        .unwrap();
    assert_eq!(p3.len(), 5);
    assert_eq!(p3[0].0, b"k:20");
    assert_eq!(p3[4].0, b"k:24");
    assert!(cur3.is_none());
}

#[tokio::test]
async fn test_scan_prefix_bounded_pagination() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    for i in 0..25 {
        let key = format!("k:{:02}", i);
        let val = format!("val:{:02}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(tx).await.unwrap();

    let (p1, cur1) = storage
        .scan_prefix_bounded(b"k:", 10, None)
        .await
        .unwrap();
    assert_eq!(p1.len(), 10);
    assert_eq!(p1[0].0, b"k:00");
    assert_eq!(p1[9].0, b"k:09");
    assert!(cur1.is_some());
    let cur1_val = cur1.unwrap();
    assert_eq!(cur1_val, b"k:09");

    let (p2, cur2) = storage
        .scan_prefix_bounded(b"k:", 10, Some(&cur1_val))
        .await
        .unwrap();
    assert_eq!(p2.len(), 10);
    assert_eq!(p2[0].0, b"k:10");
    assert_eq!(p2[9].0, b"k:19");
    assert!(cur2.is_some());
    let cur2_val = cur2.unwrap();
    assert_eq!(cur2_val, b"k:19");

    let (p3, cur3) = storage
        .scan_prefix_bounded(b"k:", 10, Some(&cur2_val))
        .await
        .unwrap();
    assert_eq!(p3.len(), 5);
    assert_eq!(p3[0].0, b"k:20");
    assert_eq!(p3[4].0, b"k:24");
    assert!(cur3.is_none());
}

#[tokio::test]
async fn test_scan_bounded_respects_limit_and_cursor() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    for i in 0..25 {
        let key = format!("k:{:02}", i);
        let val = format!("val:{:02}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(tx).await.unwrap();

    let (p1, cur1) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, None)
        .await
        .unwrap();
    assert_eq!(p1.len(), 10);
    assert_eq!(p1[0].0, b"k:00");
    assert_eq!(p1[9].0, b"k:09");
    assert!(cur1.is_some());
    let cur1_val = cur1.unwrap();
    assert_eq!(cur1_val, b"k:09");

    let (p2, cur2) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, Some(&cur1_val))
        .await
        .unwrap();
    assert_eq!(p2.len(), 10);
    assert_eq!(p2[0].0, b"k:10");
    assert_eq!(p2[9].0, b"k:19");
    assert!(cur2.is_some());
    let cur2_val = cur2.unwrap();
    assert_eq!(cur2_val, b"k:19");

    let (p3, cur3) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, Some(&cur2_val))
        .await
        .unwrap();
    assert_eq!(p3.len(), 5);
    assert_eq!(p3[0].0, b"k:20");
    assert_eq!(p3[4].0, b"k:24");
    assert!(cur3.is_none());
}

#[tokio::test]
async fn test_scan_bounded_rejects_oversized_internal_merge() {
    let (storage, _tmp) = test_storage().await;
    let tx = TxId::new(1);

    for i in 0..100 {
        let key = format!("k:{:03}", i);
        let val = format!("val:{:03}", i);
        storage
            .put(tx, key.as_bytes(), val.as_bytes())
            .await
            .unwrap();
    }
    storage.commit(tx).await.unwrap();

    let (batch, next_cur) = storage
        .scan_bounded(Bound::Unbounded, Bound::Unbounded, 5, None)
        .await
        .unwrap();
    assert_eq!(batch.len(), 5);
    assert!(next_cur.is_some());
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

#[test]
fn prop_lsm_scan_prefix_at_consistency() {
    use proptest::prelude::*;

    #[derive(Debug, Clone)]
    enum Op {
        Put(u8, Vec<u8>),
        Delete(u8),
    }

    let op_strategy = proptest::collection::vec(
        prop_oneof![
            (1u8..10, proptest::collection::vec(any::<u8>(), 1..10))
                .prop_map(|(k, v)| Op::Put(k, v)),
            (1u8..10).prop_map(Op::Delete),
        ],
        10..60,
    );

    proptest!(ProptestConfig::with_cases(20), |(ops in op_strategy)| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let tmp = tempfile::TempDir::new().unwrap();
            let config = LsmConfig {
                path: tmp.path().to_path_buf(),
                memtable_size_limit: 1024 * 1024,
                max_ram_mb: 64,
                tx_timeout: Duration::from_secs(60),
                compaction: CompactionConfig::default(),
                encryption_passphrase: None,
                ..Default::default()
            };
            let storage = LsmStorage::new(config).await.unwrap();

            let mut current_tx = 1u64;
            let mut tx_checkpoints = Vec::new();

            for op in ops {
                let tx = TxId::new(current_tx);
                match op {
                    Op::Put(key_id, val) => {
                        let key = format!("pfx:{}", key_id);
                        let _ = storage.put(tx, key.as_bytes(), &val).await;
                    }
                    Op::Delete(key_id) => {
                        let key = format!("pfx:{}", key_id);
                        let _ = storage.delete(tx, key.as_bytes()).await;
                    }
                }
                if storage.commit(tx).await.is_ok() {
                    let seq = storage.last_seq_no().await.unwrap();
                    tx_checkpoints.push((current_tx, seq));
                    current_tx += 1;
                }
            }

            for &(_tx_num, target_seq) in &tx_checkpoints {
                let scanned = storage.scan_prefix_at(b"pfx:", target_seq).await.unwrap();
                let actual_map: std::collections::BTreeMap<_, _> = scanned.into_iter().collect();

                let mut ref_map = std::collections::BTreeMap::new();
                let state = storage.state.read().await;

                let mut all_entries = Vec::new();
                for (k, v, seq, _tx) in state.memtable.iter() {
                    all_entries.push((k.to_vec(), v.to_vec(), seq));
                }
                for mt in &state.immutable_memtables {
                    for (k, v, seq, _tx) in mt.iter() {
                        all_entries.push((k.to_vec(), v.to_vec(), seq));
                    }
                }
                drop(state);

                let sstables = storage.sstables.read().await;
                for sst in sstables.iter() {
                    let sst_entries = sst.scan_prefix(b"pfx:").await.unwrap();
                    for (k, v, seq, _tx) in sst_entries {
                        all_entries.push((k.to_vec(), v.to_vec(), seq));
                    }
                }
                drop(sstables);

                all_entries.sort_by_key(|e| e.2 & !TOMBSTONE_BIT);

                for (k, v, seq) in all_entries {
                    let raw_seq = seq & !TOMBSTONE_BIT;
                    if raw_seq <= target_seq && k.starts_with(b"pfx:") {
                        if (seq & TOMBSTONE_BIT) != 0 {
                            ref_map.remove(&k);
                        } else {
                            ref_map.insert(k, v);
                        }
                    }
                }

                prop_assert_eq!(actual_map, ref_map, "scan_prefix_at at seq {} must match reference model", target_seq);
            }
            Ok(())
        }).unwrap();
    });
}
