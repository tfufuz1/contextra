use contextra_ports::{BoxFuture, StorageEngine, StorageStats, TextIndex};
use contextra_text::{InvertedIndex, Posting, PostingList};
use contextra_types::{DocId, Result, TxId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct TestStorage {
    data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
}

impl TestStorage {
    fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }
}

impl StorageEngine for TestStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move {
            Ok(self
                .data
                .lock()
                .unwrap()
                .get(key)
                .cloned()
                .map(bytes::Bytes::from))
        })
    }

    fn put<'a>(&'a self, _tx: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data
                .lock()
                .unwrap()
                .insert(key.to_vec(), value.to_vec());
            Ok(())
        })
    }

    fn delete<'a>(&'a self, _tx: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data.lock().unwrap().remove(key);
            Ok(())
        })
    }

    fn commit<'a>(&'a self, _tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback<'a>(&'a self, _tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_to_tx<'a>(&'a self, _tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        _seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.get(key).await })
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

    fn scan_prefix<'a>(&'a self, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let data = self.data.lock().unwrap();
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
        Box::pin(async move { self.scan_prefix(prefix).await })
    }
}

#[tokio::test]
async fn test_injected_posting_remains_detected() -> Result<()> {
    let storage = Arc::new(TestStorage::new());
    let index = InvertedIndex::new(storage, "test_ns");

    let tx1 = TxId::new(1);
    let doc1 = DocId::new(10);

    index.insert(tx1, doc1, "alpha beta").await?;
    index.commit(tx1).await?;

    let tx2 = TxId::new(2);
    index.delete(tx2, doc1).await?;
    index.commit(tx2).await?;

    let terms = vec![b"alpha".to_vec(), b"beta".to_vec()];

    // Clean check
    let clean = index.scan_residue(doc1, &terms).await?;
    assert_eq!(clean.postings_found, 0);

    // Fault injection: Inject residual posting for doc1 into resident index
    index
        .resident_index()
        .upsert_posting("alpha", Posting::new(doc1, 1, 2));

    let evidence = index.scan_residue(doc1, &terms).await?;
    assert!(
        evidence.postings_found > 0,
        "Scanner MUST detect residual posting entry"
    );

    Ok(())
}

#[tokio::test]
async fn test_injected_df0_dict_entry_detected() -> Result<()> {
    let storage = Arc::new(TestStorage::new());
    let index = InvertedIndex::new(storage, "test_ns");

    let doc1 = DocId::new(10);
    let terms = vec![b"orphan_term".to_vec()];

    // Fault injection: Insert empty posting list (df == 0) for orphan_term into resident index
    index
        .resident_index()
        .insert_list("orphan_term".to_string(), PostingList::empty());

    let evidence = index.scan_residue(doc1, &terms).await?;
    assert!(
        evidence.dict_entries_remaining_df0 > 0,
        "Scanner MUST detect dictionary entry with df == 0"
    );

    Ok(())
}
