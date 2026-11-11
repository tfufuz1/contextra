//! Unit tests for Block-Max WAND with ports StorageEngine.

use crate::wand::*;
#[cfg(test)]
#[cfg(feature = "workspace")]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use crate::types::DocId;
    use crate::inverted::InvertedIndex;
    use contextra_ports::BoxFuture;
    use contextra_ports::StorageEngine;
    use parking_lot::RwLock;

    type MockStoreMap = RwLock<BTreeMap<Vec<u8>, Vec<(Vec<u8>, u64)>>>;

    struct MockStorage {
        store: MockStoreMap,
        staged: RwLock<BTreeMap<contextra_types::TxId, Vec<Vec<u8>>>>,
        next_seq: std::sync::atomic::AtomicU64,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                store: MockStoreMap::new(BTreeMap::new()),
                staged: RwLock::new(BTreeMap::new()),
                next_seq: std::sync::atomic::AtomicU64::new(1),
            }
        }
    }

    impl StorageEngine for MockStorage {
        fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, contextra_types::Result<Option<bytes::Bytes>>> {
            Box::pin(async move { StorageEngine::get_at_seq(self, key, u64::MAX).await })
        }
        fn put<'a>(
            &'a self,
            tx_id: contextra_types::TxId,
            key: &'a [u8],
            value: &'a [u8],
        ) -> BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move {
                let seq = self
                    .next_seq
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                self.store
                    .write()
                    .entry(key.to_vec())
                    .or_default()
                    .push((value.to_vec(), seq));
                self.staged
                    .write()
                    .entry(tx_id)
                    .or_default()
                    .push(key.to_vec());
                Ok(())
            })
        }
        fn delete<'a>(
            &'a self,
            tx_id: contextra_types::TxId,
            key: &'a [u8],
        ) -> BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move {
                let seq = self
                    .next_seq
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let mut w = self.store.write();
                if let Some(versions) = w.get_mut(key) {
                    versions.push((Vec::new(), seq | contextra_types::TOMBSTONE_BIT));
                } else {
                    w.insert(
                        key.to_vec(),
                        vec![(Vec::new(), seq | contextra_types::TOMBSTONE_BIT)],
                    );
                }
                self.staged
                    .write()
                    .entry(tx_id)
                    .or_default()
                    .push(key.to_vec());
                Ok(())
            })
        }
        fn commit<'a>(&'a self, tx_id: contextra_types::TxId) -> BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move {
                self.staged.write().remove(&tx_id);
                Ok(())
            })
        }
        fn rollback<'a>(&'a self, tx_id: contextra_types::TxId) -> BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move {
                let keys = self.staged.write().remove(&tx_id).unwrap_or_default();
                let mut store = self.store.write();
                for k in keys {
                    if let Some(versions) = store.get_mut(&k) {
                        versions.pop();
                        if versions.is_empty() {
                            store.remove(&k);
                        }
                    }
                }
                Ok(())
            })
        }
        fn rollback_to_tx<'a>(
            &'a self,
            _tx_id: contextra_types::TxId,
        ) -> BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn get_at_seq<'a>(
            &'a self,
            key: &'a [u8],
            seq: u64,
        ) -> BoxFuture<'a, contextra_types::Result<Option<bytes::Bytes>>> {
            Box::pin(async move {
                let store = self.store.read();
                if let Some(versions) = store.get(key) {
                    for (val, v_seq) in versions.iter().rev() {
                        let raw_seq = v_seq & !contextra_types::TOMBSTONE_BIT;
                        if raw_seq <= seq {
                            if (v_seq & contextra_types::TOMBSTONE_BIT) != 0 {
                                return Ok(None);
                            }
                            return Ok(Some(bytes::Bytes::from(val.clone())));
                        }
                    }
                }
                Ok(None)
            })
        }
        fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, contextra_types::Result<u64>> {
            Box::pin(async move {
                Ok(self
                    .next_seq
                    .load(std::sync::atomic::Ordering::SeqCst)
                    .saturating_sub(1))
            })
        }
        fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, contextra_types::Result<contextra_types::TxId>> {
            Box::pin(async move { Ok(contextra_types::TxId::new(0)) })
        }
        fn flush<'a>(&'a self) -> BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn stats<'a>(&'a self) -> BoxFuture<'a, contextra_types::Result<contextra_ports::StorageStats>> {
            Box::pin(async move {
                Ok(contextra_ports::StorageStats {
                    num_segments: 0,
                    total_size_bytes: 0,
                    memtable_size_bytes: 0,
                })
            })
        }
        fn pin_checkpoint<'a>(&'a self, _id: u64) -> BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn unpin_checkpoint<'a>(&'a self, _id: u64) -> BoxFuture<'a, contextra_types::Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn scan<'a>(
            &'a self,
            _start: std::ops::Bound<&'a [u8]>,
            _end: std::ops::Bound<&'a [u8]>,
            _: Option<usize>,
        ) -> BoxFuture<'a, contextra_types::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(Vec::new()) })
        }
        fn scan_prefix<'a>(
            &'a self,
            prefix: &'a [u8],
        ) -> BoxFuture<'a, contextra_types::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { StorageEngine::scan_prefix_at(self, prefix, u64::MAX).await })
        }
        fn scan_prefix_at<'a>(
            &'a self,
            prefix: &'a [u8],
            seq_no: u64,
        ) -> BoxFuture<'a, contextra_types::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move {
                let store = self.store.read();
                let mut results = Vec::new();
                for (k, versions) in store.iter() {
                    if k.starts_with(prefix) {
                        for (val, v_seq) in versions.iter().rev() {
                            let raw_seq = v_seq & !contextra_types::TOMBSTONE_BIT;
                            if raw_seq <= seq_no {
                                if (v_seq & contextra_types::TOMBSTONE_BIT) == 0 {
                                    results.push((k.clone(), val.clone()));
                                }
                                break;
                            }
                        }
                    }
                }
                Ok(results)
            })
        }
    }

    // ANCHOR[contextra-text:TEST-WAND-001]
    // Prüft Ergebnis-Äquivalenz und Auswertungs-Pruning von Block-Max WAND im Vergleich zum Vollscan.
    #[tokio::test]
    async fn test_block_max_wand_equivalence_and_pruning_performance() -> contextra_types::Result<()> {
        let storage = Arc::new(MockStorage::new());
        let index = InvertedIndex::new(storage.clone(), "wand_test");
        let tx = contextra_types::TxId::new(1);

        // Index 500 documents:
        // "common" occurs in all 500 documents.
        // "rare" occurs in only 5 documents (DocId 100, 200, 300, 400, 500) with high frequency.
        for i in 1u64..=500u64 {
            let doc_id = DocId::new(i);
            let text = if i % 100 == 0 {
                format!("common rare rare rare rare rare doc {i}")
            } else {
                format!("common doc {i}")
            };
            index.upsert_document(tx.into(), doc_id, &text).await?;
        }
        index.commit_stats(tx.into()).await?;
        StorageEngine::commit(storage.as_ref(), tx).await?;

        // 1. Equivalence Test: WAND search results vs full scan expectations
        let k = 5;
        let query_results = index.search_bm25("common rare", k, None).await?;
        assert_eq!(query_results.len(), k);

        // The top 5 documents should be doc_ids 100, 200, 300, 400, 500 because they match "rare" 5x
        let top_doc_ids: Vec<u64> = query_results
            .iter()
            .map(|(id, _)| id.inner() as u64)
            .collect();
        assert!(top_doc_ids.contains(&100));
        assert!(top_doc_ids.contains(&200));
        assert!(top_doc_ids.contains(&300));
        assert!(top_doc_ids.contains(&400));
        assert!(top_doc_ids.contains(&500));

        // 2. Pruning Performance Test: Verify evaluated_docs_count is significantly less than 500
        let n = index.total_docs.load(std::sync::atomic::Ordering::Acquire);
        let avg_len_x1000 = index
            .avg_doc_len_x1000
            .load(std::sync::atomic::Ordering::Acquire);
        let avg_doc_len = avg_len_x1000 as f32 / 1000.0;
        let terms = vec!["common".to_string(), "rare".to_string()];

        let wand_metrics = block_max_wand_search(
            storage.as_ref(),
            &terms,
            &index.resident_index,
            |term| index.key_term_prefix(term),
            |doc_id, term| index.key_tombstone(doc_id, term),
            |doc_id_inner| index.key_with_id("dl:", doc_id_inner),
            k,
            None,
            n,
            avg_doc_len,
        )
        .await?;

        assert_eq!(wand_metrics.results, query_results);

        // Without WAND, full scan evaluates all 500 documents.
        // With Block-Max WAND, dynamic thresholding skips low-scoring blocks of "common".
        assert!(
            wand_metrics.evaluated_docs_count < 500,
            "Block-Max WAND evaluated {} docs, which should be significantly less than total 500",
            wand_metrics.evaluated_docs_count
        );

        Ok(())
    }
}
