#![allow(dead_code, unused_imports)]

use contextra_ports::{StorageEngine, TextIndex};
use contextra_text::InvertedIndex;
use contextra_types::{DocId, TxId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

type MockStoreMap = RwLock<HashMap<Vec<u8>, Vec<(Vec<u8>, u64)>>>;

struct MockStorage {
    store: MockStoreMap,
    staged: RwLock<HashMap<TxId, Vec<Vec<u8>>>>,
    next_seq: std::sync::atomic::AtomicU64,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            store: MockStoreMap::new(HashMap::new()),
            staged: RwLock::new(HashMap::new()),
            next_seq: std::sync::atomic::AtomicU64::new(1),
        }
    }
}

impl StorageEngine for MockStorage {
    fn get<'a>(
        &'a self,
        key: &'a [u8],
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
    }
    fn put<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
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
        tx_id: TxId,
        key: &'a [u8],
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
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
    fn commit<'a>(
        &'a self,
        tx_id: TxId,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        Box::pin(async move {
            self.staged.write().remove(&tx_id);
            Ok(())
        })
    }
    fn rollback<'a>(
        &'a self,
        tx_id: TxId,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
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
        _tx_id: TxId,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Option<bytes::Bytes>>> {
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
    fn last_seq_no<'a>(&'a self) -> contextra_ports::BoxFuture<'a, contextra_types::Result<u64>> {
        Box::pin(async move {
            Ok(self
                .next_seq
                .load(std::sync::atomic::Ordering::SeqCst)
                .saturating_sub(1))
        })
    }
    fn last_tx_id<'a>(&'a self) -> contextra_ports::BoxFuture<'a, contextra_types::Result<TxId>> {
        Box::pin(async move { Ok(TxId::new(0)) })
    }
    fn flush<'a>(&'a self) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn stats<'a>(
        &'a self,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<contextra_ports::StorageStats>>
    {
        Box::pin(async move {
            Ok(contextra_ports::StorageStats {
                num_segments: 0,
                total_size_bytes: 0,
                memtable_size_bytes: 0,
            })
        })
    }
    fn pin_checkpoint<'a>(
        &'a self,
        _id: u64,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn unpin_checkpoint<'a>(
        &'a self,
        _id: u64,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn scan<'a>(
        &'a self,
        _start: std::ops::Bound<&'a [u8]>,
        _end: std::ops::Bound<&'a [u8]>,
        _: Option<usize>,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { self.scan_prefix_at(prefix, u64::MAX).await })
    }
    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
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

#[cfg(feature = "docid-128")]
#[tokio::test]
async fn test_docid_128_tombstone_resolution_exceeding_u64_max() -> contextra_types::Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "docid_128_test");

    // Construct a DocId > u64::MAX
    let large_doc_id = DocId::new((u64::MAX as u128) + 42);

    // 1. Upsert document with large DocId
    let tx1 = TxId::new(1);
    index
        .upsert_document(tx1, large_doc_id, "rust search engine tombstone test")
        .await?;
    index.commit(tx1).await?;

    // Verify document is searchable
    let results_before = index.search("tombstone", 10).await?;
    assert_eq!(results_before.len(), 1);
    assert_eq!(results_before[0].doc_id, large_doc_id);

    // 2. Delete document -> writes tombstone entries
    let tx2 = TxId::new(2);
    index.delete_document(tx2, large_doc_id).await?;
    index.commit(tx2).await?;

    // 3. Resolve tombstones
    let tx_resolve = TxId::new(3);
    let resolved_count = index.resolve_tombstones(tx_resolve).await?;
    index.commit(tx_resolve).await?;

    // Under docid-128, all tombstones for large_doc_id must be resolved and not silently skipped
    assert!(
        resolved_count > 0,
        "Expected resolved_count > 0 for u128 DocId tombstone resolution, got {}",
        resolved_count
    );

    Ok(())
}
