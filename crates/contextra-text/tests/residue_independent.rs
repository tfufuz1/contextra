use contextra_ports::{BoxFuture, StorageEngine, StorageStats, TextIndex};
use contextra_text::{InvertedIndex, TextSweepDepth};
use contextra_types::{DocId, Result, TxId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct MemoryStorage {
    data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
}

impl MemoryStorage {
    fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }
}

impl StorageEngine for MemoryStorage {
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
        Box::pin(async move { Ok(TxId::try_new(1).unwrap()) })
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
async fn test_caller_provides_wrong_terms_independent_scan_finds_residue() -> Result<()> {
    let storage = Arc::new(MemoryStorage::new());
    let index = InvertedIndex::new(storage, "ns_wrong_terms");

    let tx1 = TxId::try_new(1).unwrap();
    let doc1 = DocId::new(100);
    let text = "apple banana cherry";

    index.insert(tx1, doc1, text).await?;
    index.commit(tx1).await?;

    // Caller provides wrong/empty terms
    let wrong_terms = vec![b"orange".to_vec(), b"mango".to_vec()];
    let caller_ev = index.scan_residue(doc1, &wrong_terms).await?;
    assert_eq!(caller_ev.postings_found, 0); // Caller scan falsely indicates no residue!

    // Independent scan finds the actual residue before deletion
    let independent_ev_before = index
        .scan_residue_independent(doc1, TextSweepDepth::FullSweep)
        .await?;
    assert_eq!(independent_ev_before.postings_found, 3);
    assert_eq!(independent_ev_before.sweep_depth, TextSweepDepth::FullSweep);

    // Delete doc1
    let tx2 = TxId::try_new(2).unwrap();
    index.delete(tx2, doc1).await?;
    index.commit(tx2).await?;

    // Independent scan after deletion confirms clean state
    let independent_ev_after = index
        .scan_residue_independent(doc1, TextSweepDepth::FullSweep)
        .await?;
    assert_eq!(independent_ev_after.postings_found, 0);
    assert_eq!(independent_ev_after.positions_found, 0);
    assert_eq!(independent_ev_after.dict_entries_remaining_df0, 0);
    assert!(independent_ev_after.doc_count_before_after_consistent);
    assert!(independent_ev_after.derived_stats_recomputed);

    Ok(())
}

#[tokio::test]
async fn test_rare_canary_term_df1_purged_on_deletion() -> Result<()> {
    let storage = Arc::new(MemoryStorage::new());
    let index = InvertedIndex::new(storage, "ns_canary");

    let tx1 = TxId::try_new(1).unwrap();
    let doc1 = DocId::new(200);
    let text = "canarysecretterm commonword";

    index.insert(tx1, doc1, text).await?;
    index.commit(tx1).await?;

    // Verify search works before deletion
    let search_res = index.search("canarysecretterm", 10).await?;
    assert_eq!(search_res.len(), 1);

    // Delete doc1
    let tx2 = TxId::try_new(2).unwrap();
    index.delete(tx2, doc1).await?;
    index.commit(tx2).await?;

    // Verify independent scan reports no df==0 remaining dictionary entries
    let ev = index
        .scan_residue_independent(doc1, TextSweepDepth::FullSweep)
        .await?;
    assert_eq!(ev.postings_found, 0);
    assert_eq!(ev.dict_entries_remaining_df0, 0);

    // Verify canary search returns nothing
    let search_res_after = index.search("canarysecretterm", 10).await?;
    assert!(search_res_after.is_empty());

    // Verify resident_index does not hold canary term
    assert!(index.resident_index().get("canarysecretterm").is_none());

    Ok(())
}

#[tokio::test]
async fn test_statistical_consistency_anti_mirroring_reference() -> Result<()> {
    let storage1 = Arc::new(MemoryStorage::new());
    let index1 = InvertedIndex::new(storage1, "ns_stats");

    let tx1 = TxId::try_new(1).unwrap();
    let doc1 = DocId::new(10);
    let doc2 = DocId::new(20);

    index1.insert(tx1, doc1, "alpha beta gamma").await?;
    index1.insert(tx1, doc2, "delta epsilon").await?;
    index1.commit(tx1).await?;

    // Delete doc1 from index1
    let tx2 = TxId::try_new(2).unwrap();
    index1.delete(tx2, doc1).await?;
    index1.commit(tx2).await?;

    // Create fresh reference index without doc1
    let storage_ref = Arc::new(MemoryStorage::new());
    let index_ref = InvertedIndex::new(storage_ref, "ns_stats_ref");

    let tx_ref = TxId::try_new(1).unwrap();
    index_ref.insert(tx_ref, doc2, "delta epsilon").await?;
    index_ref.commit(tx_ref).await?;

    // Compare statistics
    let stats1 = index1.stats().await?;
    let stats_ref = index_ref.stats().await?;

    assert_eq!(stats1.num_documents, stats_ref.num_documents);
    assert_eq!(stats1.num_tokens, stats_ref.num_tokens);
    assert_eq!(index1.len().await, index_ref.len().await);

    // Verify residue scan reports consistent statistics
    let ev = index1
        .scan_residue_independent(doc1, TextSweepDepth::FullSweep)
        .await?;
    assert!(ev.doc_count_before_after_consistent);
    assert!(ev.derived_stats_recomputed);

    Ok(())
}

#[tokio::test]
async fn test_multiple_documents_sharing_terms() -> Result<()> {
    let storage = Arc::new(MemoryStorage::new());
    let index = InvertedIndex::new(storage, "ns_shared");

    let tx1 = TxId::try_new(1).unwrap();
    let doc1 = DocId::new(1);
    let doc2 = DocId::new(2);

    index
        .insert(tx1, doc1, "shared_keyword_rust fast_search")
        .await?;
    index
        .insert(tx1, doc2, "shared_keyword_rust memory_safe")
        .await?;
    index.commit(tx1).await?;

    // Delete doc1
    let tx2 = TxId::try_new(2).unwrap();
    index.delete(tx2, doc1).await?;
    index.commit(tx2).await?;

    // Scan doc1 -> no postings found for doc1
    let ev1 = index
        .scan_residue_independent(doc1, TextSweepDepth::FullSweep)
        .await?;
    assert_eq!(ev1.postings_found, 0);

    // Search for shared keyword -> doc2 is still found
    let results = index.search("shared_keyword_rust", 10).await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].doc_id, doc2);

    // Scan doc2 -> doc2 postings remain intact
    let ev2 = index
        .scan_residue_independent(doc2, TextSweepDepth::FullSweep)
        .await?;
    assert!(ev2.postings_found > 0);

    Ok(())
}
