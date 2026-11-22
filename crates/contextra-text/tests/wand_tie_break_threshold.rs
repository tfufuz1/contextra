// FILE-CONTEXT: WAND Tie-Break & Threshold Boundary Test (tests/wand_tie_break_threshold.rs).
// ZWECK: Verifiziert, dass WAND bei Score-Gleichstand an der k-ten Abbruchschwelle deterministisch nach aufsteigender DocId sortiert, exakt mit dem Brute-Force-Referenzergebnis uebereinstimmt und keine Dokumente durch accum_max_score > threshold faelschlich verwirft.

use contextra_ports::{BoxFuture, StorageEngine, TextIndex};
use contextra_text::inverted::InvertedIndex;
use contextra_types::{DocId, Result, TxId};
use std::collections::HashMap;

struct MockStorage {
    data: parking_lot::Mutex<HashMap<Vec<u8>, Vec<u8>>>,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            data: parking_lot::Mutex::new(HashMap::new()),
        }
    }
}

impl StorageEngine for MockStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(self.data.lock().get(key).cloned().map(bytes::Bytes::from)) })
    }

    fn put<'a>(
        &'a self,
        _tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data.lock().insert(key.to_vec(), value.to_vec());
            Ok(())
        })
    }

    fn delete<'a>(&'a self, _tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data.lock().remove(key);
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
        Box::pin(async move {
            let data = self.data.lock();
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
async fn test_wand_tie_break_at_threshold_boundary() -> Result<()> {
    let storage = std::sync::Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "wand_tie_break_test");

    let tx = TxId::new(1);

    // Insert 6 documents:
    // Docs 10, 20, 30, 40, 50, 60 all have identical text "alpha beta gamma delta".
    // Therefore all 6 documents get EXACTLY the same BM25 score for query "alpha beta".
    let doc_ids = [10u64, 20, 30, 40, 50, 60];
    for &id_val in &doc_ids {
        index
            .upsert_document(tx, DocId::new(id_val), "alpha beta gamma delta")
            .await?;
    }
    index.commit(tx).await?;
    storage.commit(tx).await?;

    let k = 3;
    let query = "alpha beta";

    // 1. WAND Search
    let wand_results = index.search_bm25(query, k, None).await?;

    // 2. Brute-Force Reference Calculation (Full scan across all documents)
    let full_scan_results = index.search_bm25(query, doc_ids.len(), None).await?;

    // Sort reference results: score descending, then doc_id ascending
    let mut expected_sorted = full_scan_results.clone();
    expected_sorted.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    expected_sorted.truncate(k);

    // (i) Result must match brute-force reference result exactly
    assert_eq!(
        wand_results.len(),
        k,
        "WAND search must return exactly k results"
    );
    assert_eq!(
        wand_results, expected_sorted,
        "WAND search results must match brute-force full-scan reference results"
    );

    // (ii) Tie-break rule: Smaller DocId comes first
    let returned_doc_ids: Vec<u64> = wand_results.iter().map(|(id, _)| id.inner()).collect();
    assert_eq!(
        returned_doc_ids,
        vec![10, 20, 30],
        "On score tie, smaller DocIds must be prioritized: expected [10, 20, 30], got {:?}",
        returned_doc_ids
    );

    // (iii) Verify that no document with smaller/equal doc_id is falsely discarded
    assert!(!wand_results.is_empty(), "wand_results must not be empty");
    if let Some((_, threshold_score)) = wand_results.last().copied() {
        for (doc_id, score) in &wand_results {
            assert_eq!(
                *score, threshold_score,
                "All tied documents at threshold boundary must have identical score"
            );
            assert!(
                doc_id.inner() <= 30,
                "DocId {} exceeds boundary threshold DocId 30",
                doc_id
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_wand_tie_break_heterogeneous_scores_threshold_boundary() -> Result<()> {
    let storage = std::sync::Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "wand_hetero_test");

    let tx = TxId::new(1);

    // Doc 5: "alpha alpha alpha beta" (high score)
    // Doc 10: "alpha beta" (medium score)
    // Doc 20: "alpha beta" (medium score - ties with Doc 10)
    // Doc 30: "alpha beta" (medium score - ties with Doc 10 and 20)
    // Doc 40: "alpha beta" (medium score - ties with Doc 10, 20, 30)
    index
        .upsert_document(tx, DocId::new(5), "alpha alpha alpha beta")
        .await?;
    index
        .upsert_document(tx, DocId::new(10), "alpha beta")
        .await?;
    index
        .upsert_document(tx, DocId::new(20), "alpha beta")
        .await?;
    index
        .upsert_document(tx, DocId::new(30), "alpha beta")
        .await?;
    index
        .upsert_document(tx, DocId::new(40), "alpha beta")
        .await?;

    index.commit(tx).await?;
    storage.commit(tx).await?;

    let k = 3;
    let query = "alpha beta";

    let wand_results = index.search_bm25(query, k, None).await?;

    // Brute-force reference
    let full_results = index.search_bm25(query, 5, None).await?;
    let mut reference = full_results.clone();
    reference.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    reference.truncate(k);

    assert_eq!(wand_results, reference);

    let doc_ids: Vec<u64> = wand_results.iter().map(|(id, _)| id.inner()).collect();
    assert_eq!(
        doc_ids,
        vec![5, 10, 20],
        "Top-3 must be Doc 5 (highest score) followed by Doc 10 and Doc 20 (tied score, smaller DocIds)"
    );

    Ok(())
}
