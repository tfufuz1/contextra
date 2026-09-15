//! Integration tests for B-5: Tombstone read evaluation & MVCC snapshot isolation in inverted index search.

use memfuse_core::{BoxFuture, DocId, Result, StorageEngine, TextIndex, TxId};
use memfuse_text::InvertedIndex;
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
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
        Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
    }

    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Vec<u8>>>> {
        Box::pin(async move {
            let store = self.store.read();
            if let Some(versions) = store.get(key) {
                for (val, v_seq) in versions.iter().rev() {
                    let raw_seq = v_seq & !memfuse_core::TOMBSTONE_BIT;
                    if raw_seq <= seq {
                        if (v_seq & memfuse_core::TOMBSTONE_BIT) != 0 {
                            return Ok(None);
                        }
                        return Ok(Some(val.clone()));
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
                versions.push((Vec::new(), seq | memfuse_core::TOMBSTONE_BIT));
            } else {
                w.insert(
                    key.to_vec(),
                    vec![(Vec::new(), seq | memfuse_core::TOMBSTONE_BIT)],
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

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<memfuse_core::StorageStats>> {
        Box::pin(async move {
            Ok(memfuse_core::StorageStats {
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
                        let raw_seq = v_seq & !memfuse_core::TOMBSTONE_BIT;
                        if raw_seq <= seq_no {
                            if (v_seq & memfuse_core::TOMBSTONE_BIT) == 0 {
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

/// Invariant Check: A document that was deleted or updated (leaving tombstones)
/// MUST NOT appear in BM25 search results before `resolve_tombstones()` is called.
#[tokio::test]
async fn proof_deleted_doc_not_in_search_results() -> Result<()> {
    let storage = Arc::new(MVCCMockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "default");

    let doc_id = DocId::new(42);

    // 1. Insert document: "rust programming language"
    let tx1 = TxId::new(1);
    index
        .insert(tx1, doc_id, "rust programming language")
        .await?;
    index.commit(tx1).await?;

    // Search matches doc 42
    let res = index.search("rust", 10).await?;
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].doc_id, doc_id);

    // 2. Update document to "python data science" (writes tombstones for "rust", "programming", "language")
    let tx2 = TxId::new(2);
    index.insert(tx2, doc_id, "python data science").await?;
    index.commit(tx2).await?;

    // WITHOUT calling resolve_tombstones(), searching for "rust" MUST NOT return doc 42
    let res_updated = index.search("rust", 10).await?;
    assert!(
        res_updated.is_empty(),
        "Updated document with tombstone for 'rust' must not appear in search results for 'rust'"
    );

    // Searching for "python" MUST return doc 42
    let res_python = index.search("python", 10).await?;
    assert_eq!(res_python.len(), 1);
    assert_eq!(res_python[0].doc_id, doc_id);

    // 3. Delete document completely
    let tx3 = TxId::new(3);
    index.delete(tx3, doc_id).await?;
    index.commit(tx3).await?;

    // Searching for "python" MUST NOT return doc 42
    let res_deleted = index.search("python", 10).await?;
    assert!(
        res_deleted.is_empty(),
        "Deleted document must not appear in search results for 'python'"
    );

    Ok(())
}

/// Invariant Check: Tombstone evaluation MUST respect MVCC snapshot isolation (`at_seq`).
/// A snapshot taken BEFORE the update/delete must still see the old document,
/// while a snapshot taken AFTER the update/delete must NOT see the old terms.
#[tokio::test]
async fn proof_tombstone_snapshot_isolation() -> Result<()> {
    let storage = Arc::new(MVCCMockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "isolation");

    let doc_id = DocId::new(100);

    // Tx1: Insert doc 100 with "architecture design"
    let tx1 = TxId::new(1);
    index.insert(tx1, doc_id, "architecture design").await?;
    index.commit(tx1).await?;
    let seq_after_insert = storage.last_seq_no().await?;

    // Tx2: Update doc 100 to "performance optimization"
    let tx2 = TxId::new(2);
    index
        .insert(tx2, doc_id, "performance optimization")
        .await?;
    index.commit(tx2).await?;
    let seq_after_update = storage.last_seq_no().await?;

    // At seq_after_insert (BEFORE update): searching "architecture" MUST find doc 100
    let res_before = index
        .search_at("architecture", 10, seq_after_insert)
        .await?;
    assert_eq!(
        res_before.len(),
        1,
        "Snapshot before update must still return the document for 'architecture'"
    );
    assert_eq!(res_before[0].doc_id, doc_id);

    // At seq_after_update (AFTER update): searching "architecture" MUST NOT find doc 100
    let res_after = index
        .search_at("architecture", 10, seq_after_update)
        .await?;
    assert!(
        res_after.is_empty(),
        "Snapshot after update must NOT return the document for tombstoned term 'architecture'"
    );

    // At seq_after_update (AFTER update): searching "performance" MUST find doc 100
    let res_perf = index.search_at("performance", 10, seq_after_update).await?;
    assert_eq!(res_perf.len(), 1);
    assert_eq!(res_perf[0].doc_id, doc_id);

    Ok(())
}
