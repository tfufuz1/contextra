use super::*;
use memfuse_core::{Result, TOMBSTONE_BIT};

pub(super) enum SstableScanMode<'a> {
    Prefix(&'a [u8]),
    Range(std::ops::Bound<&'a [u8]>, std::ops::Bound<&'a [u8]>),
}

#[inline]
pub(super) fn check_in_range(
    k: &[u8],
    start: std::ops::Bound<&[u8]>,
    end: std::ops::Bound<&[u8]>,
) -> bool {
    use std::ops::Bound;
    (match start {
        Bound::Included(s) => k >= s,
        Bound::Excluded(s) => k > s,
        Bound::Unbounded => true,
    }) && (match end {
        Bound::Included(e) => k <= e,
        Bound::Excluded(e) => k < e,
        Bound::Unbounded => true,
    })
}

impl LsmStorage {
    /// Evaluates detailed traversal metrics (evaluated_sstables, bloom_passes, range_passes, block_reads, found) for point lookups.
    pub async fn point_lookup_metrics(&self, key: &[u8]) -> (usize, usize, usize, usize, bool) {
        let sstables = self.sstables.read().await;
        let mut total_eval = 0usize;
        let mut total_bloom_pass = 0usize;
        let mut total_range_pass = 0usize;
        let mut total_block_read = 0usize;
        let mut found = false;

        for sst in sstables.iter().rev() {
            total_eval += 1;
            let (b_pass, r_pass, blk_read, k_found) = sst.lookup_metrics(key).await;
            if b_pass {
                total_bloom_pass += 1;
            }
            if r_pass {
                total_range_pass += 1;
            }
            if blk_read {
                total_block_read += 1;
            }
            if k_found {
                found = true;
                break;
            }
        }

        (
            total_eval,
            total_bloom_pass,
            total_range_pass,
            total_block_read,
            found,
        )
    }

    /// Internal helper that collects visible entries across SSTables, immutable MemTables,
    /// and active MemTable according to MVCC visibility, tombstone masking, and optional accumulator limits.
    pub(super) async fn collect_visible_entries<F>(
        &self,
        mode: SstableScanMode<'_>,
        entry_filter: F,
        check_accumulator: bool,
        per_source_limit: Option<usize>,
        context_name: &str,
    ) -> Result<std::collections::BTreeMap<Bytes, (Bytes, u64)>>
    where
        F: Fn(&[u8], u64, u64) -> bool,
    {
        let mut map: std::collections::BTreeMap<Bytes, (Bytes, u64)> =
            std::collections::BTreeMap::new();
        let state = self.state.read().await;
        let sstables = self.sstables.read().await;

        let last_tx = self.last_committed_tx.load(Ordering::Acquire);

        // 1. SSTables
        for sst in sstables.iter() {
            let entries = match mode {
                SstableScanMode::Prefix(prefix) => {
                    let first = sst.first_key();
                    let last = sst.last_key();
                    if !first.is_empty() && !last.is_empty() {
                        if prefix > last.as_ref() {
                            continue;
                        }
                        let mut prefix_end = prefix.to_vec();
                        if let Some(last_byte) = prefix_end.last_mut() {
                            if let Some(next_byte) = last_byte.checked_add(1) {
                                *last_byte = next_byte;
                                if first.as_ref() >= prefix_end.as_slice() {
                                    continue;
                                }
                            }
                        }
                    }
                    sst.scan_prefix(prefix).await?
                }
                SstableScanMode::Range(start, end) => {
                    sst.scan_range(start.map(|s| s), end.map(|e| e)).await?
                }
            };

            let mut source_count = 0usize;
            for (k, v, seq, tx) in entries {
                let raw_seq = seq & !TOMBSTONE_BIT;
                if (tx <= last_tx || tx >= TxId::INTERNAL_BASE)
                    && entry_filter(k.as_ref(), raw_seq, tx)
                {
                    let entry = map.entry(k).or_insert_with(|| (v.clone(), seq));
                    if (seq & !TOMBSTONE_BIT) > (entry.1 & !TOMBSTONE_BIT) {
                        *entry = (v, seq);
                    }
                    if check_accumulator && map.len() > memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR {
                        return Err(MemFuseError::LimitExceeded {
                            limit: memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR,
                            context: format!(
                                "{context_name}: internal merge accumulator exceeded — range too wide, narrow the scan range"
                            ),
                        });
                    }
                    source_count += 1;
                    if let Some(lim) = per_source_limit {
                        if source_count >= lim {
                            break;
                        }
                    }
                }
            }
        }

        let scan_memtable =
            |mt: &MemTable, target: &mut std::collections::BTreeMap<Bytes, (Bytes, u64)>| {
                match mode {
                    SstableScanMode::Prefix(prefix) => {
                        mt.scan_prefix_into_matching(
                            prefix,
                            u64::MAX,
                            TxId(last_tx),
                            target,
                            &entry_filter,
                        );
                    }
                    SstableScanMode::Range(
                        std::ops::Bound::Unbounded,
                        std::ops::Bound::Unbounded,
                    ) => {
                        for (k, v, seq, tx) in mt.iter() {
                            if tx > last_tx && tx < TxId::INTERNAL_BASE {
                                continue;
                            }
                            let raw_seq = seq & !TOMBSTONE_BIT;
                            if entry_filter(k.as_ref(), raw_seq, tx) {
                                let entry =
                                    target.entry(k.clone()).or_insert_with(|| (v.clone(), seq));
                                if (seq & !TOMBSTONE_BIT) > (entry.1 & !TOMBSTONE_BIT) {
                                    *entry = (v.clone(), seq);
                                }
                            }
                        }
                    }
                    SstableScanMode::Range(start, end) => {
                        mt.scan_range_into_matching(
                            start,
                            end,
                            u64::MAX,
                            TxId(last_tx),
                            target,
                            &entry_filter,
                        );
                    }
                }
            };

        // 2. Immutable memtables (older -> newer)
        for mt in &state.immutable_memtables {
            scan_memtable(mt, &mut map);
            if check_accumulator && map.len() > memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR {
                return Err(MemFuseError::LimitExceeded {
                    limit: memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR,
                    context: format!(
                        "{context_name}: internal merge accumulator exceeded — range too wide, narrow the scan range"
                    ),
                });
            }
        }

        // 3. Active memtable
        scan_memtable(&state.memtable, &mut map);
        if check_accumulator && map.len() > memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR {
            return Err(MemFuseError::LimitExceeded {
                limit: memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR,
                context: format!(
                    "{context_name}: internal merge accumulator exceeded — range too wide, narrow the scan range"
                ),
            });
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_storage() -> (LsmStorage, TempDir) {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");
        (storage, tmp)
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let (storage, _tmp) = test_storage().await;
        let val = storage.get(b"nonexistent").await.expect("get");
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_scan_range() {
        let (storage, _tmp) = test_storage().await;
        let tx = TxId::new(1);

        for c in b'a'..=b'z' {
            let key = [c];
            let val = [c, c];
            storage.put(tx, &key, &val).await.expect("put");
        }
        storage.commit(tx).await.expect("commit");

        use std::ops::Bound;
        let results = storage
            .scan(Bound::Included(b"c"), Bound::Included(b"g"), None)
            .await
            .expect("scan");
        assert_eq!(results.len(), 5);
        assert_eq!(results[0].0, b"c");
        assert_eq!(results[4].0, b"g");

        let results = storage
            .scan(Bound::Excluded(b"c"), Bound::Excluded(b"g"), None)
            .await
            .expect("scan");
        assert_eq!(results.len(), 3);

        let results = storage
            .scan(Bound::Unbounded, Bound::Included(b"d"), None)
            .await
            .expect("scan");
        assert_eq!(results.len(), 4);

        let tx2 = TxId::new(2);
        storage.delete(tx2, b"e").await.expect("delete");
        storage.commit(tx2).await.expect("commit");

        let results = storage
            .scan(Bound::Included(b"d"), Bound::Included(b"f"), None)
            .await
            .expect("scan");
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_bounded_scan_and_prefix_bounded_limits_candidate_evaluation() {
        let (storage, _tmp) = test_storage().await;
        let tx = TxId::new(1);

        for i in 0..100 {
            let key = format!("k:{:02}", i);
            let val = format!("v:{:02}", i);
            storage
                .put(tx, key.as_bytes(), val.as_bytes())
                .await
                .expect("put");
        }
        storage.commit(tx).await.expect("commit");

        let res_scan = storage
            .scan(
                std::ops::Bound::Unbounded,
                std::ops::Bound::Unbounded,
                Some(5),
            )
            .await
            .expect("scan");
        assert_eq!(res_scan.len(), 5);
        assert_eq!(res_scan[0].0, b"k:00");
        assert_eq!(res_scan[4].0, b"k:04");

        let (res_prefix, next_cursor) = storage
            .scan_prefix_bounded(b"k:", 5, None)
            .await
            .expect("scan_prefix_bounded");
        assert_eq!(res_prefix.len(), 5);
        assert_eq!(res_prefix[0].0, b"k:00");
        assert_eq!(res_prefix[4].0, b"k:04");
        assert_eq!(next_cursor, Some(b"k:04".to_vec()));
    }

    #[tokio::test]
    async fn test_mvcc_snapshot_isolation() {
        let (storage, _tmp) = test_storage().await;

        let tx1 = TxId::new(1);
        storage.put(tx1, b"key", b"val_t1").await.expect("put t1");
        storage.commit(tx1).await.expect("commit t1");
        let seq_t1 = storage.last_seq_no().await.expect("seq t1");

        let tx2 = TxId::new(2);
        storage.put(tx2, b"key", b"val_t2").await.expect("put t2");
        storage.commit(tx2).await.expect("commit t2");

        let val_at_t1 = storage.get_at_seq(b"key", seq_t1).await.expect("get at t1");
        assert_eq!(val_at_t1, Some(b"val_t1".to_vec()));

        let val_current = storage.get(b"key").await.expect("get current");
        assert_eq!(val_current, Some(b"val_t2".to_vec()));
    }

    #[tokio::test]
    async fn test_scan_prefix_at_uncommitted_isolation() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig {
                min_sstables_per_tier: 2,
                size_ratio: 4.0,
                check_interval: Duration::from_secs(30),
                yield_threshold: 1000,
                max_memory_bytes: Some(1024 * 1024),
                ..Default::default()
            },
            encryption_passphrase: None,
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");

        let tx1 = TxId::new(1);
        storage.put(tx1, b"prefix:doc1", b"val1").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"prefix:doc2", b"val2").await.unwrap();

        let seq = storage.last_seq_no().await.unwrap();
        let scanned = storage.scan_prefix_at(b"prefix:", seq).await.unwrap();

        assert_eq!(scanned.len(), 1);
        assert_eq!(scanned[0].0, b"prefix:doc1");
    }

    #[tokio::test]
    async fn test_get_at_seq_mvcc_sequence_correctness() {
        let (storage, _tmp) = test_storage().await;
        let key = b"mvcc_key";

        let tx1 = TxId::new(1);
        storage.put(tx1, key, b"a").await.unwrap();
        storage.commit(tx1).await.unwrap();
        let seq1 = storage.last_seq_no().await.unwrap();
        assert_eq!(seq1, 1);

        let tx2 = TxId::new(2);
        storage.delete(tx2, key).await.unwrap();
        storage.commit(tx2).await.unwrap();
        let seq2 = storage.last_seq_no().await.unwrap();
        assert_eq!(seq2, 2);

        let tx3 = TxId::new(3);
        storage.put(tx3, key, b"b").await.unwrap();
        storage.commit(tx3).await.unwrap();
        let seq3 = storage.last_seq_no().await.unwrap();
        assert_eq!(seq3, 3);

        let val_seq0 = storage.get_at_seq(key, 0).await.unwrap();
        assert_eq!(val_seq0, None, "seq 0 should be before any write");

        let val_seq1 = storage.get_at_seq(key, 1).await.unwrap();
        assert_eq!(val_seq1, Some(b"a".to_vec()));

        let val_seq2 = storage.get_at_seq(key, 2).await.unwrap();
        assert_eq!(val_seq2, None, "seq 2 should return None for tombstone");

        let val_seq3 = storage.get_at_seq(key, 3).await.unwrap();
        assert_eq!(val_seq3, Some(b"b".to_vec()));
    }

    #[tokio::test]
    async fn test_scan_bounded_respects_accumulator_ceiling_with_wide_range() {
        let (storage, _tmp) = test_storage().await;

        let total = memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR + 5;
        let batch_size = 5000;
        for (idx, chunk) in (0..total)
            .collect::<Vec<_>>()
            .chunks(batch_size)
            .enumerate()
        {
            let tx = TxId::new((idx + 1) as u64);
            let entries: Vec<(Vec<u8>, Vec<u8>)> = chunk
                .iter()
                .map(|i| {
                    (
                        format!("k:{:06}", i).into_bytes(),
                        format!("v:{:06}", i).into_bytes(),
                    )
                })
                .collect();
            storage.put_batch(tx, &entries).await.unwrap();
            storage.commit(tx).await.unwrap();
        }

        use std::ops::Bound;
        let res = storage
            .scan_bounded(Bound::Unbounded, Bound::Unbounded, 10, None)
            .await;

        assert!(matches!(
            res,
            Err(MemFuseError::LimitExceeded { limit, .. }) if limit == memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR
        ));
    }

    #[tokio::test]
    async fn test_scan_bounded_pagination_matches_full_scan() {
        let (storage, _tmp) = test_storage().await;
        let tx = TxId::new(1);

        let entries: Vec<(Vec<u8>, Vec<u8>)> = (0..50)
            .map(|i| {
                (
                    format!("k:{:02}", i).into_bytes(),
                    format!("v:{:02}", i).into_bytes(),
                )
            })
            .collect();

        storage.put_batch(tx, &entries).await.unwrap();
        storage.commit(tx).await.unwrap();

        use std::ops::Bound;
        let full_scan = storage
            .scan(Bound::Unbounded, Bound::Unbounded, None)
            .await
            .unwrap();

        let mut paginated = Vec::new();
        let mut cursor: Option<Vec<u8>> = None;

        loop {
            let (batch, next_cursor) = storage
                .scan_bounded(Bound::Unbounded, Bound::Unbounded, 7, cursor.as_deref())
                .await
                .unwrap();

            if batch.is_empty() {
                break;
            }

            paginated.extend(batch);

            if let Some(next) = next_cursor {
                cursor = Some(next);
            } else {
                break;
            }
        }

        assert_eq!(paginated, full_scan);
    }

    #[tokio::test]
    async fn test_scan_prefix_bounded_pagination() {
        let (storage, _tmp) = test_storage().await;
        let tx = TxId::new(1);

        for i in 0..25 {
            let key = format!("pfx:{:02}", i);
            let val = format!("val:{:02}", i);
            storage
                .put(tx, key.as_bytes(), val.as_bytes())
                .await
                .unwrap();
        }
        storage.commit(tx).await.unwrap();

        let (p1, cur1) = storage
            .scan_prefix_bounded(b"pfx:", 10, None)
            .await
            .unwrap();
        assert_eq!(p1.len(), 10);
        assert_eq!(p1[0].0, b"pfx:00");
        assert_eq!(p1[9].0, b"pfx:09");
        assert!(cur1.is_some());
        let cur1_val = cur1.unwrap();
        assert_eq!(cur1_val, b"pfx:09");

        let (p2, cur2) = storage
            .scan_prefix_bounded(b"pfx:", 10, Some(&cur1_val))
            .await
            .unwrap();
        assert_eq!(p2.len(), 10);
        assert_eq!(p2[0].0, b"pfx:10");
        assert_eq!(p2[9].0, b"pfx:19");
        assert!(cur2.is_some());
        let cur2_val = cur2.unwrap();
        assert_eq!(cur2_val, b"pfx:19");

        let (p3, cur3) = storage
            .scan_prefix_bounded(b"pfx:", 10, Some(&cur2_val))
            .await
            .unwrap();
        assert_eq!(p3.len(), 5);
        assert_eq!(p3[0].0, b"pfx:20");
        assert_eq!(p3[4].0, b"pfx:24");
        assert!(cur3.is_none());
    }

    #[tokio::test]
    async fn test_scan_bounded_respects_limit_and_cursor() {
        use std::ops::Bound;
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
        use std::ops::Bound;
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
}
