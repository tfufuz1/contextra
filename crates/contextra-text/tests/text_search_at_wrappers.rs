//! Integration tests for Bm25Scorer and BM25MorphIndex search_at delegation (ADR-024 Snapshot Isolation).

use contextra_types::{DocId, Result, TxId};
use contextra_ports::{BoxFuture, StorageEngine, TextIndex};
use contextra_text::morphology::GermanCompoundSplitter;
use contextra_text::{Bm25Scorer, BM25MorphIndex};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

type MockStoreMap = RwLock<HashMap<Vec<u8>, Vec<(Vec<u8>, u64)>>>;

struct MVCCMockStorage {
    store: MockStoreMap,
    staged: RwLock<HashMap<TxId, Vec<Vec<u8>>>>,
    next_seq: std::sync::atomic::AtomicU64,
}

impl MVCCMockStorage {
    fn new() -> Self {
        Self {
            store: MockStoreMap::new(HashMap::new()),
            staged: RwLock::new(HashMap::new()),
            next_seq: std::sync::atomic::AtomicU64::new(1),
        }
    }
}

impl StorageEngine for MVCCMockStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
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

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
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

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
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

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.staged.write().remove(&tx_id);
            Ok(())
        })
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
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

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            Ok(self
                .next_seq
                .load(std::sync::atomic::Ordering::SeqCst)
                .saturating_sub(1))
        })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId::new(0)) })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<contextra_ports::StorageStats>> {
        Box::pin(async move {
            Ok(contextra_ports::StorageStats {
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
        Box::pin(async move { self.scan_prefix_at(prefix, u64::MAX).await })
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
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

#[tokio::test]
async fn test_bm25_scorer_search_at_snapshot_isolation() -> Result<()> {
    let storage = Arc::new(MVCCMockStorage::new());
    let scorer = Bm25Scorer::new(storage.clone(), "scorer_ns");

    let doc_a = DocId::new(1);
    let doc_b = DocId::new(2);

    // 1. Commit Document A
    let tx1 = TxId::new(1);
    scorer.insert(tx1, doc_a, "hello world").await?;
    scorer.commit(tx1).await?;
    let seq_a = storage.last_seq_no().await?;

    // 2. Commit Document B
    let tx2 = TxId::new(2);
    scorer.insert(tx2, doc_b, "hello rust").await?;
    scorer.commit(tx2).await?;

    // 3. Explicit check that search_at does NOT return CapabilityUnsupported
    let res_at_a_raw = scorer.search_at("hello", 10, seq_a).await;
    assert!(
        !matches!(
            res_at_a_raw,
            Err(contextra_types::ContextraError::CapabilityUnsupported { .. })
        ),
        "search_at must not return CapabilityUnsupported on Bm25Scorer"
    );

    // 4. search_at seq_a must return Document A, but NOT Document B
    let res_at_a = res_at_a_raw?;
    assert_eq!(res_at_a.len(), 1, "search_at(seq_a) should return 1 doc");
    assert_eq!(
        res_at_a[0].doc_id, doc_a,
        "search_at(seq_a) should return Document A"
    );

    // 5. search without snapshot returns both documents
    let res_latest = scorer.search("hello", 10).await?;
    assert_eq!(res_latest.len(), 2, "search() should return 2 docs");

    Ok(())
}

#[tokio::test]
async fn test_bm25_morph_index_search_at_snapshot_isolation() -> Result<()> {
    let storage = Arc::new(MVCCMockStorage::new());
    let splitter = Arc::new(GermanCompoundSplitter::new());
    let morph_index = BM25MorphIndex::new(storage.clone(), "morph_ns", splitter);

    let doc_a = DocId::new(10);
    let doc_b = DocId::new(20);

    // 1. Commit Document A
    let tx1 = TxId::new(1);
    morph_index
        .insert(tx1, doc_a, "bundesverfassungsgericht urteil")
        .await?;
    morph_index.commit(tx1).await?;
    let seq_a = storage.last_seq_no().await?;

    // 2. Commit Document B
    let tx2 = TxId::new(2);
    morph_index
        .insert(tx2, doc_b, "bundesverfassungsgericht beschluss")
        .await?;
    morph_index.commit(tx2).await?;

    // 3. Explicit check that search_at does NOT return CapabilityUnsupported
    let res_at_a_raw = morph_index
        .search_at("bundesverfassungsgericht", 10, seq_a)
        .await;
    assert!(
        !matches!(
            res_at_a_raw,
            Err(contextra_types::ContextraError::CapabilityUnsupported { .. })
        ),
        "search_at must not return CapabilityUnsupported on BM25MorphIndex"
    );

    // 4. search_at seq_a must return Document A, but NOT Document B
    let res_at_a = res_at_a_raw?;
    assert_eq!(res_at_a.len(), 1, "search_at(seq_a) should return 1 doc");
    assert_eq!(
        res_at_a[0].doc_id, doc_a,
        "search_at(seq_a) should return Document A"
    );

    // 5. search without snapshot returns both documents
    let res_latest = morph_index
        .search("bundesverfassungsgericht", 10)
        .await?;
    assert_eq!(res_latest.len(), 2, "search() should return 2 docs");

    Ok(())
}

#[tokio::test]
async fn test_wrappers_search_at_edge_cases() -> Result<()> {
    let storage = Arc::new(MVCCMockStorage::new());
    let scorer = Bm25Scorer::new(storage.clone(), "edge_scorer");
    let splitter = Arc::new(GermanCompoundSplitter::new());
    let morph_index = BM25MorphIndex::new(storage.clone(), "edge_morph", splitter);

    let tx = TxId::new(1);
    let doc_id = DocId::new(100);

    scorer.insert(tx, doc_id, "test data processing").await?;
    scorer.commit(tx).await?;
    let seq = storage.last_seq_no().await?;

    morph_index
        .insert(tx, doc_id, "test data processing")
        .await?;
    morph_index.commit(tx).await?;

    // Empty query -> Ok(vec![])
    let empty_scorer = scorer.search_at("", 10, seq).await?;
    assert!(empty_scorer.is_empty());

    let empty_morph = morph_index.search_at("", 10, seq).await?;
    assert!(empty_morph.is_empty());

    // k == 0 -> Ok(vec![])
    let k0_scorer = scorer.search_at("test", 0, seq).await?;
    assert!(k0_scorer.is_empty());

    let k0_morph = morph_index.search_at("test", 0, seq).await?;
    assert!(k0_morph.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_bm25_morph_index_compound_term_search_at() -> Result<()> {
    let storage = Arc::new(MVCCMockStorage::new());
    let splitter = Arc::new(GermanCompoundSplitter::new());
    let morph_index = BM25MorphIndex::new(storage.clone(), "compound_ns", splitter);

    let doc_a = DocId::new(100);
    let doc_b = DocId::new(200);

    // 1. Commit Document A with compound term "bundesverfassungsgericht"
    let tx1 = TxId::new(1);
    morph_index
        .insert(tx1, doc_a, "das bundesverfassungsgericht hat entschieden")
        .await?;
    morph_index.commit(tx1).await?;
    let seq_a = storage.last_seq_no().await?;

    // 2. Commit Document B with another compound term "datenbankverbindung"
    let tx2 = TxId::new(2);
    morph_index
        .insert(tx2, doc_b, "die datenbankverbindung wurde unterbrochen")
        .await?;
    morph_index.commit(tx2).await?;

    // 3. search_at seq_a for "bundesverfassungsgericht" finds Document A
    let res_a = morph_index
        .search_at("bundesverfassungsgericht", 10, seq_a)
        .await?;
    assert_eq!(res_a.len(), 1);
    assert_eq!(res_a[0].doc_id, doc_a);

    // 4. search_at seq_a for "datenbankverbindung" returns empty (Doc B came later)
    let res_b_at_seq_a = morph_index
        .search_at("datenbankverbindung", 10, seq_a)
        .await?;
    assert!(
        res_b_at_seq_a.is_empty(),
        "Doc B inserted after seq_a must not be returned"
    );

    Ok(())
}
