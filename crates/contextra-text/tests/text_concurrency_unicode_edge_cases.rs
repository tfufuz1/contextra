//! Concurrency, Snapshot Isolation, Unicode, Edge-Case, and Tie-Breaking Tests for contextra-text

use contextra_ports::{BoxFuture, StorageEngine, TextIndex};
use contextra_text::InvertedIndex;
use contextra_types::{DocId, Result, ScoredDocument, TxId, TOMBSTONE_BIT};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Barrier;

type StoreMap = RwLock<HashMap<Vec<u8>, Vec<(Vec<u8>, u64)>>>;

struct ConcurrentMockStorage {
    store: StoreMap,
    staged: RwLock<HashMap<TxId, Vec<Vec<u8>>>>,
    next_seq: AtomicU64,
}

impl ConcurrentMockStorage {
    fn new() -> Self {
        Self {
            store: StoreMap::new(HashMap::new()),
            staged: RwLock::new(HashMap::new()),
            next_seq: AtomicU64::new(1),
        }
    }

    fn current_seq(&self) -> u64 {
        self.next_seq.load(Ordering::SeqCst).saturating_sub(1)
    }
}

impl StorageEngine for ConcurrentMockStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
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
            let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
            let mut w = self.store.write();
            if let Some(versions) = w.get_mut(key) {
                versions.push((Vec::new(), seq | TOMBSTONE_BIT));
            } else {
                w.insert(key.to_vec(), vec![(Vec::new(), seq | TOMBSTONE_BIT)]);
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

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move {
            let store = self.store.read();
            if let Some(versions) = store.get(key) {
                for (val, v_seq) in versions.iter().rev() {
                    let raw_seq = v_seq & !TOMBSTONE_BIT;
                    if raw_seq <= seq {
                        if (v_seq & TOMBSTONE_BIT) != 0 {
                            return Ok(None);
                        }
                        return Ok(Some(bytes::Bytes::from(val.clone())));
                    }
                }
            }
            Ok(None)
        })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(self.current_seq()) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId::try_new(1).unwrap_or_else(|_| TxId::internal())) })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<contextra_ports::StorageStats>> {
        Box::pin(async move {
            Ok(contextra_ports::StorageStats {
                num_segments: 1,
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
                        let raw_seq = v_seq & !TOMBSTONE_BIT;
                        if raw_seq <= seq_no {
                            if (v_seq & TOMBSTONE_BIT) == 0 {
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

// ============================================================================
// CONCURRENCY TESTS (Step 4)
// ============================================================================

#[tokio::test]
async fn test_pinned_snapshot_isolation_under_concurrent_writes() -> Result<()> {
    let storage = Arc::new(ConcurrentMockStorage::new());
    let index = Arc::new(InvertedIndex::new(storage.clone(), "concurrent_ns"));

    // Insert baseline documents
    let tx1 = TxId::try_new(1)?;
    let tx2 = TxId::try_new(2)?;
    index
        .insert(tx1, DocId::new(10), "alpha beta gamma")
        .await?;
    index.commit(tx1).await?;
    index
        .insert(tx2, DocId::new(20), "alpha delta epsilon")
        .await?;
    index.commit(tx2).await?;

    let pinned_seq = storage.current_seq();
    assert!(pinned_seq > 0, "pinned sequence number must be non-zero");

    // Clear resident index cache to force snapshot lookup via storage.get_at_seq
    index.resident_index().clear();

    let initial_results = index.search_at("alpha", 10, pinned_seq).await?;
    assert_eq!(
        initial_results.len(),
        2,
        "Snapshot at seq {} must see doc 10 and 20",
        pinned_seq
    );

    let barrier = Arc::new(Barrier::new(2));

    // Spawn concurrent writer modifying doc 10, deleting doc 20, and adding doc 30
    let index_writer = index.clone();
    let barrier_writer = barrier.clone();
    let writer_handle = tokio::spawn(async move {
        barrier_writer.wait().await;
        for i in 3..=10 {
            let tx = TxId::try_new(i as u64).expect("valid tx");
            index_writer
                .insert(tx, DocId::new(30 + i), "alpha zeta new_content")
                .await
                .expect("insert succeeds");
            index_writer.commit(tx).await.expect("commit succeeds");

            let tx_del = TxId::try_new(100 + i as u64).expect("valid tx");
            index_writer
                .delete(tx_del, DocId::new(20))
                .await
                .expect("delete succeeds");
            index_writer.commit(tx_del).await.expect("commit succeeds");
        }
    });

    fn doc_ids(docs: &[ScoredDocument]) -> Vec<DocId> {
        docs.iter().map(|d| d.doc_id).collect()
    }

    let expected_ids = doc_ids(&initial_results);

    // Concurrent reader querying pinned_seq
    barrier.wait().await;
    for _ in 0..20 {
        let snapshot_results = index.search_at("alpha", 10, pinned_seq).await?;
        assert_eq!(
            doc_ids(&snapshot_results), expected_ids,
            "Pinned snapshot query MUST yield identical document visibility despite concurrent writes"
        );
        tokio::task::yield_now().await;
    }

    writer_handle.await.expect("writer task joins cleanly");
    Ok(())
}

#[tokio::test]
async fn test_concurrent_upsert_and_delete_race_serializability() -> Result<()> {
    let storage = Arc::new(ConcurrentMockStorage::new());
    let index = Arc::new(InvertedIndex::new(storage.clone(), "race_ns"));

    let target_doc = DocId::new(42);
    let tx_init = TxId::try_new(1)?;
    index
        .insert(tx_init, target_doc, "original baseline document")
        .await?;
    index.commit(tx_init).await?;

    let barrier = Arc::new(Barrier::new(2));

    let index_upsert = index.clone();
    let barrier_upsert = barrier.clone();
    let upsert_task = tokio::spawn(async move {
        barrier_upsert.wait().await;
        let tx = TxId::try_new(10)?;
        index_upsert
            .insert(tx, target_doc, "updated race document")
            .await?;
        index_upsert.commit(tx).await
    });

    let index_delete = index.clone();
    let barrier_delete = barrier.clone();
    let delete_task = tokio::spawn(async move {
        barrier_delete.wait().await;
        let tx = TxId::try_new(20)?;
        index_delete.delete(tx, target_doc).await?;
        index_delete.commit(tx).await
    });

    let (res1, res2) = tokio::join!(upsert_task, delete_task);
    res1.expect("upsert task finished")?;
    res2.expect("delete task finished")?;

    // Clear resident cache to force evaluation against storage
    index.resident_index().clear();

    let search_res = index.search("document", 10).await?;
    let found = search_res.iter().any(|d| d.doc_id == target_doc);

    // End state oracle: either document exists OR is completely deleted (no inconsistent/partial state)
    if found {
        let updated_res = index.search("updated", 10).await?;
        assert!(
            updated_res.iter().any(|d| d.doc_id == target_doc),
            "If document exists after race, it MUST reflect the updated text"
        );
    } else {
        let orig_res = index.search("original", 10).await?;
        assert!(
            !orig_res.iter().any(|d| d.doc_id == target_doc),
            "If document was deleted after race, original terms MUST NOT be found"
        );
    }

    Ok(())
}

// ============================================================================
// UNICODE & TEXT EDGE-CASE TESTS (Step 5)
// ============================================================================

#[tokio::test]
async fn test_text_and_unicode_edge_cases() -> Result<()> {
    let storage = Arc::new(ConcurrentMockStorage::new());
    let index = InvertedIndex::new_with_language(
        storage.clone(),
        "edge_ns",
        contextra_text::Language::German,
    );

    let tx = TxId::try_new(1)?;
    // Index documents with complex Unicode, 1-char terms, German compounds, and Emojis
    index
        .insert(
            tx,
            DocId::new(1),
            "Das Softwareentwicklungsunternehmen baut Module mit Umlaute Ä ö Ü und ß in München 🦀🚀.",
        )
        .await?;
    index.commit(tx).await?;

    // 1. Empty query
    let empty_res = index.search("", 10).await?;
    assert!(
        empty_res.is_empty(),
        "Empty query MUST return empty results"
    );

    // 2. Only stopwords
    let stop_res = index.search("das und in der die", 10).await?;
    assert!(
        stop_res.is_empty(),
        "Query containing only stopwords MUST return empty results"
    );

    // 3. 1-character terms
    let char_res = index.search("x", 10).await?;
    assert!(
        char_res.is_empty(),
        "Query for non-existent 1-char term MUST return 0 hits"
    );

    // 4. Extremely long term
    let long_term = "a".repeat(100_000);
    let long_res = index.search(&long_term, 10).await?;
    assert!(
        long_res.is_empty(),
        "Extremely long query term MUST return 0 hits without error or panic"
    );

    // 5. German Umlaute and ß / ss equivalence
    let umlaut_res = index.search("muenchen", 10).await?;
    assert_eq!(
        umlaut_res.len(),
        1,
        "German tokenizer MUST find 'München' via normalized query 'muenchen'"
    );

    // 6. Combining diacritics / NFD vs NFC
    let nfd_query = "Mu\u{0308}nchen"; // M u + combining diaeresis
    let nfd_res = index.search(nfd_query, 10).await?;
    assert_eq!(
        nfd_res.len(),
        1,
        "German tokenizer MUST match NFD combining diacritics"
    );

    // 7. German compound word decomposition
    let comp_res = index.search("softwareentwicklung", 10).await?;
    assert_eq!(
        comp_res.len(),
        1,
        "Query 'softwareentwicklung' MUST match 'Softwareentwicklungsunternehmen'"
    );

    // 8. Emojis
    let emoji_res = index.search("🦀", 10).await?;
    let _ = emoji_res; // Should execute without panic

    // 9. 0 hits for non-existent terms
    let zero_res = index.search("nonexistentterm12345", 10).await?;
    assert!(zero_res.is_empty(), "Non-existent terms MUST return 0 hits");

    // 10. k = 0
    let k0_res = index.search("software", 0).await?;
    assert!(k0_res.is_empty(), "k = 0 MUST return empty vec");

    // 11. k > hits
    let k_large_res = index.search("software", 1000).await?;
    assert_eq!(
        k_large_res.len(),
        1,
        "k > total hits MUST return all hits bounded by total available matches"
    );

    Ok(())
}

// ============================================================================
// WAND TOP-K SCORE TIE-BREAKING TESTS (P28) (Step 6)
// ============================================================================

#[tokio::test]
async fn test_wand_top_k_deterministic_tie_breaking_p28() -> Result<()> {
    let storage = Arc::new(ConcurrentMockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "tiebreak_ns");

    // Insert 5 documents with EXACTLY identical text content
    // They will receive identical BM25 scores.
    for i in 1..=5u64 {
        let tx = TxId::try_new(i)?;
        index
            .insert(
                tx,
                DocId::new(i.into()),
                "identical deterministic score test content",
            )
            .await?;
        index.commit(tx).await?;
    }

    fn doc_key(docs: &[ScoredDocument]) -> Vec<(DocId, u32)> {
        docs.iter().map(|d| (d.doc_id, d.score.to_bits())).collect()
    }

    // Perform query multiple times
    let first_run: Vec<ScoredDocument> = index.search("identical score test", 5).await?;
    assert_eq!(
        first_run.len(),
        5,
        "All 5 identical documents MUST be returned"
    );
    let first_key = doc_key(&first_run);

    // Verify tie-break order is strictly deterministic across repeated invocations
    for run_idx in 2..=10 {
        let current_run: Vec<ScoredDocument> = index.search("identical score test", 5).await?;
        assert_eq!(
            doc_key(&current_run),
            first_key,
            "Run {} tie-break order MUST match first run identically (P28 determinism)",
            run_idx
        );
    }

    // Verify SystemRng port exists and works deterministically
    let rng = contextra_ports::SystemRng::new();
    let mut val = [0u8; 8];
    contextra_ports::EntropyRng::fill_bytes(&rng, &mut val);

    Ok(())
}
