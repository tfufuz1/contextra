#![forbid(unsafe_code)]

use contextra_ports::StorageEngine;
use contextra_text::inverted::InvertedIndex;
use contextra_types::{DocId, Result, TxId};
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
    ) -> contextra_ports::BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
    }
    fn put<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> contextra_ports::BoxFuture<'a, Result<()>> {
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
    ) -> contextra_ports::BoxFuture<'a, Result<()>> {
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
    fn commit<'a>(&'a self, tx_id: TxId) -> contextra_ports::BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.staged.write().remove(&tx_id);
            Ok(())
        })
    }
    fn rollback<'a>(&'a self, tx_id: TxId) -> contextra_ports::BoxFuture<'a, Result<()>> {
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
    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> contextra_ports::BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> contextra_ports::BoxFuture<'a, Result<Option<bytes::Bytes>>> {
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
    fn last_seq_no<'a>(&'a self) -> contextra_ports::BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            Ok(self
                .next_seq
                .load(std::sync::atomic::Ordering::SeqCst)
                .saturating_sub(1))
        })
    }
    fn last_tx_id<'a>(&'a self) -> contextra_ports::BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId::try_new(0).unwrap()) })
    }
    fn flush<'a>(&'a self) -> contextra_ports::BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn stats<'a>(
        &'a self,
    ) -> contextra_ports::BoxFuture<'a, Result<contextra_ports::StorageStats>> {
        Box::pin(async move {
            Ok(contextra_ports::StorageStats {
                num_segments: 0,
                total_size_bytes: 0,
                memtable_size_bytes: 0,
            })
        })
    }
    fn pin_checkpoint<'a>(&'a self, _id: u64) -> contextra_ports::BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn unpin_checkpoint<'a>(&'a self, _id: u64) -> contextra_ports::BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn scan<'a>(
        &'a self,
        _start: std::ops::Bound<&'a [u8]>,
        _end: std::ops::Bound<&'a [u8]>,
        _: Option<usize>,
    ) -> contextra_ports::BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> contextra_ports::BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { self.scan_prefix_at(prefix, u64::MAX).await })
    }
    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> contextra_ports::BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
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
async fn test_wand_unbatched_posting_snapshot_isolation() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "wand_snap_test");

    // Transaction 1: Add document 1 with term "snapshot_test_term"
    let tx1 = TxId::try_new(1).unwrap();
    index
        .upsert_document(tx1, DocId::new(1), "snapshot_test_term")
        .await?;
    storage.commit(tx1).await?;

    let snap1_seq = storage.last_seq_no().await?;

    // Transaction 2: Add document 2 with term "snapshot_test_term"
    let tx2 = TxId::try_new(2).unwrap();
    index
        .upsert_document(tx2, DocId::new(2), "snapshot_test_term")
        .await?;
    storage.commit(tx2).await?;

    // Create a new InvertedIndex instance pointing to the same storage to bypass resident_index cache
    let fresh_index = InvertedIndex::new(storage.clone(), "wand_snap_test");

    let results = fresh_index
        .search_bm25_at("snapshot_test_term", 10, Some(snap1_seq))
        .await?;

    // doc_id 2 was added in tx2 after snap1_seq.
    // It must NOT appear in search results for snap1_seq.
    let found_doc_ids: Vec<u64> = results.iter().map(|(id, _)| id.inner() as u64).collect();

    assert!(
        found_doc_ids.contains(&1),
        "doc 1 should be present at snapshot sequence {}",
        snap1_seq
    );
    assert!(
        !found_doc_ids.contains(&2),
        "doc 2 committed after snapshot sequence {} must not be visible in WAND search! Found: {:?}",
        snap1_seq,
        found_doc_ids
    );

    Ok(())
}
