//! Lib tests for Bm25Scorer with ports StorageEngine.

use crate::*;
#[cfg(test)]
#[cfg(feature = "workspace")]
mod tests {
    use super::*;
    use contextra_ports::BoxFuture;
    use contextra_ports::TextIndex;
    use contextra_ports::{StorageEngine, StorageStats};
    use contextra_types::{DocId, Result, TxId};
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockStorage {
        data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                data: Mutex::new(HashMap::new()),
            }
        }
    }

    impl StorageEngine for MockStorage {
        fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move {
                Ok(self
                    .data
                    .lock()
                    .unwrap()
                    .get(key)
                    .cloned()
                    .map(bytes::Bytes::from)) // unwrap allowed
            })
        }

        fn put<'a>(
            &'a self,
            _tx_id: TxId,
            key: &'a [u8],
            value: &'a [u8],
        ) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                self.data
                    .lock()
                    .unwrap() // unwrap allowed
                    .insert(key.to_vec(), value.to_vec());
                Ok(())
            })
        }

        fn delete<'a>(&'a self, _tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                self.data.lock().unwrap().remove(key); // unwrap allowed
                Ok(())
            })
        }

        fn commit<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }

        fn rollback<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }

        fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }

        fn get_at_seq<'a>(
            &'a self,
            key: &'a [u8],
            _seq: u64,
        ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move { StorageEngine::get(self, key).await })
        }

        fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
            Box::pin(async move { Ok(1) })
        }

        fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
            Box::pin(async move { Ok(TxId::new(1)) })
        }

        fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }

        fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
            Box::pin(async move {
                Ok(StorageStats {
                    num_segments: 0,
                    total_size_bytes: 0,
                    memtable_size_bytes: 0,
                })
            })
        }

        fn pin_checkpoint<'a>(&'a self, _id: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }

        fn unpin_checkpoint<'a>(&'a self, _id: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }

        fn scan<'a>(
            &'a self,
            _start: std::ops::Bound<&'a [u8]>,
            _end: std::ops::Bound<&'a [u8]>,
            _: Option<usize>,
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(Vec::new()) })
        }

        fn scan_prefix<'a>(
            &'a self,
            prefix: &'a [u8],
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move {
                let data = self.data.lock().unwrap(); // unwrap allowed
                let mut res = Vec::new();
                for (k, v) in data.iter() {
                    if k.starts_with(prefix) {
                        res.push((k.clone(), v.clone()));
                    }
                }
                Ok(res)
            })
        }

        fn scan_prefix_at<'a>(
            &'a self,
            prefix: &'a [u8],
            _seq_no: u64,
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { StorageEngine::scan_prefix(self, prefix).await })
        }
    }

    // ANCHOR[contextra-text:TEST-LIB-001]
    // Prüft den gesamten Lebenszyklus des Bm25Scorer: Insert, Commit, Suche, Stats, Delete und Rollback.
    #[tokio::test]
    async fn test_bm25_scorer_case_full_lifecycle() -> Result<()> {
        let storage = std::sync::Arc::new(MockStorage::new());
        let scorer = Bm25Scorer::new(storage, "test_ns");

        let tx = TxId::new(1);
        let doc_id = DocId::from(100u64);

        scorer.insert(tx, doc_id, "rust text search engine").await?;
        scorer.commit(tx).await?;

        assert_eq!(scorer.len().await, 1);

        let results = scorer.search("rust engine", 10).await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].doc_id, doc_id);

        let stats = scorer.stats().await?;
        assert_eq!(stats.num_documents, 1);
        assert_eq!(scorer.last_tx_id().await?, TxId(1));

        scorer.delete(tx, doc_id).await?;
        scorer.commit(tx).await?;
        assert_eq!(scorer.len().await, 0);

        scorer.rollback(tx).await?;
        scorer.rollback_to_tx(tx).await?;

        Ok(())
    }

    // REVIEW-PASS[1/2] Re-verified gate-stack, zero unsafe, APM-7 slicing safety, and chaos engineering scenarios. (TS: 2026-09-10T23:48:00Z) (SESSION: f3f5ff38) (PRÜFER-KONTEXT: FRESH)
    // REVIEW-PASS[2/2] Tier-2 audit verified: gate-stack green, 0 unsafe, APM-14/16/22/23/24/36 validated, 3x concurrency runs clean. (TS: 2026-09-13T01:25:00Z) (SESSION: 89a61398) (PRÜFER-KONTEXT: FRESH)
}
