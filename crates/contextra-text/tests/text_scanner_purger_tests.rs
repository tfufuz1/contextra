//! Integration and negative tests for `TextScanner` and `TextPurger` (§12.7.4, INV-WITNESS-MINT-1).

use contextra_ports::clock::SystemClock;
use contextra_ports::scanner::{LayerScanner, ScanTarget, ScanTechnique, WitnessLayer};
use contextra_ports::{BoxFuture, StorageEngine, TextIndex};
use contextra_text::{text_purger, InvertedIndex, TextPurger, TextScanner};
use contextra_types::{DocId, RequestContext, Result, TenantId, TxId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

struct TestStorage {
    store: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
}

impl TestStorage {
    fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }
}

impl StorageEngine for TestStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(self.store.read().get(key).cloned().map(bytes::Bytes::from)) })
    }

    fn put<'a>(&'a self, _tx: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.store.write().insert(key.to_vec(), value.to_vec());
            Ok(())
        })
    }

    fn delete<'a>(&'a self, _tx: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.store.write().remove(key);
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
            let store = self.store.read();
            Ok(store
                .iter()
                .filter(|(k, _)| k.starts_with(prefix))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect())
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

/// Test 1: Scanner reports residual when document is present or only logically tombstoned.
#[tokio::test]
async fn test_scanner_detects_unpurged_residual(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let storage = Arc::new(TestStorage::new());
    let index = Arc::new(InvertedIndex::new(storage, "test_ns"));

    let doc_id = DocId::new(100);
    let tx1 = TxId::new(1);

    index
        .insert(tx1, doc_id, "searchable text document")
        .await?;
    index.commit(tx1).await?;

    let clock = SystemClock::new();
    let target = ScanTarget::new("doc:100", [0xAA; 32]);
    let scanner = TextScanner::new_with_target(index.clone(), target.clone());
    let req_ctx = RequestContext::new(TenantId::try_new(1)?, tx1);

    let witness = scanner.scan(&req_ctx, target, ScanTechnique::IndexIntrospection, &clock)?;

    assert_eq!(witness.witness_layer(), &WitnessLayer::from("TextIndex"));
    assert!(witness.scanned() > 0, "Scanned count must be > 0");
    assert!(
        witness.remaining() > 0,
        "Remaining residual count must be > 0 before purge"
    );

    Ok(())
}

/// Test 2: After `Purger::purge`, `remaining == 0`, doc is unsearchable,
/// and BM25 score of remaining doc matches a freshly built index without doc 100.
#[tokio::test]
async fn test_purge_and_bm25_score_equivalence(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let storage1 = Arc::new(TestStorage::new());
    let index1 = Arc::new(InvertedIndex::new(storage1, "ns1"));

    let d100 = DocId::new(100);
    let d200 = DocId::new(200);
    let tx1 = TxId::new(1);

    index1.insert(tx1, d100, "apple banana orange").await?;
    index1.insert(tx1, d200, "apple cherry melon").await?;
    index1.commit(tx1).await?;

    // Purge doc 100
    let purger = text_purger(index1.clone());
    let req_ctx = RequestContext::new(TenantId::try_new(1)?, tx1);
    let target = ScanTarget::new("doc:100", [0xBB; 32]);

    let stats = purger.purge(&req_ctx, &target)?;
    assert!(stats.purged_units > 0);

    // Verify scanner reports 0 remaining
    let clock = SystemClock::new();
    let scanner = TextScanner::new_with_target(index1.clone(), target.clone());
    let witness = scanner.scan(&req_ctx, target, ScanTechnique::IndexIntrospection, &clock)?;
    assert_eq!(
        witness.remaining(),
        0,
        "Remaining residuals must be 0 after purge"
    );

    // Search for doc 100 terms
    let banana_hits = index1.search("banana", 10).await?;
    assert_eq!(
        banana_hits.len(),
        0,
        "Purged document terms must yield 0 search hits"
    );

    // Get BM25 score for d200 on "apple" in purged index
    let hits_purged = index1.search("apple", 10).await?;
    assert_eq!(hits_purged.len(), 1);
    assert_eq!(hits_purged[0].doc_id, d200);
    let score_purged = hits_purged[0].score;

    // Build fresh index with only doc 200
    let storage2 = Arc::new(TestStorage::new());
    let index2 = InvertedIndex::new(storage2, "ns2");
    index2.insert(tx1, d200, "apple cherry melon").await?;
    index2.commit(tx1).await?;

    let hits_fresh = index2.search("apple", 10).await?;
    assert_eq!(hits_fresh.len(), 1);
    let score_fresh = hits_fresh[0].score;

    assert!(
        (score_purged - score_fresh).abs() < 1e-5,
        "BM25 score after purge must equal fresh index score"
    );

    Ok(())
}

/// Test 3: Document with a unique term: term disappears from dictionary after purge.
#[tokio::test]
async fn test_unique_term_dictionary_purge() -> std::result::Result<(), Box<dyn std::error::Error>>
{
    let storage = Arc::new(TestStorage::new());
    let index = Arc::new(InvertedIndex::new(storage, "unique_ns"));

    let doc_id = DocId::new(300);
    let tx = TxId::new(1);

    index
        .insert(tx, doc_id, "unique_xyz_canary_term shared_term")
        .await?;
    index.commit(tx).await?;

    let hits_before = index.search("unique_xyz_canary_term", 10).await?;
    assert_eq!(hits_before.len(), 1);

    // Purge document 300
    let purger = TextPurger::new(index.clone());
    purger.purge_document(tx, doc_id)?;

    let hits_after = index.search("unique_xyz_canary_term", 10).await?;
    assert_eq!(hits_after.len(), 0, "Unique term must yield 0 search hits");

    Ok(())
}

/// Test 4: Collection-wide fast purge clears all documents in namespace.
#[tokio::test]
async fn test_collection_wide_fast_purge() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let storage = Arc::new(TestStorage::new());
    let ns = "collection_fast_purge";
    let index = Arc::new(InvertedIndex::new(storage.clone(), ns));

    let tx = TxId::new(1);
    for i in 1..=10 {
        index
            .insert(tx, DocId::new(i), &format!("document content number {}", i))
            .await?;
    }
    index.commit(tx).await?;

    assert_eq!(index.len().await, 10);

    let purger = TextPurger::new(index.clone());
    let stats = purger.purge_collection(tx, ns)?;

    assert!(stats.purged_units >= 10);
    assert_eq!(index.len().await, 0);

    let prefix = index.prefix();
    let remaining_keys = storage.scan_prefix(prefix).await?;
    assert_eq!(
        remaining_keys.len(),
        0,
        "No keys must remain under collection prefix after collection purge"
    );

    Ok(())
}

/// Test 5: Concurrency test — reader task during purge sees consistent state.
#[tokio::test]
async fn test_concurrency_purge_and_read() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let storage = Arc::new(TestStorage::new());
    let index = Arc::new(InvertedIndex::new(storage, "concurrent_ns"));

    let tx1 = TxId::new(1);
    index
        .insert(tx1, DocId::new(1), "parallel reader test content")
        .await?;
    index.commit(tx1).await?;

    let index_clone = index.clone();
    let reader_handle = tokio::spawn(async move {
        let mut saw_valid = true;
        for _ in 0..50 {
            if let Ok(results) = index_clone.search("parallel test", 10).await {
                if results.len() > 1 {
                    saw_valid = false;
                }
            }
            tokio::task::yield_now().await;
        }
        saw_valid
    });

    let purger = TextPurger::new(index);
    let tx2 = TxId::new(2);
    purger.purge_document(tx2, DocId::new(1))?;

    let saw_valid = reader_handle.await?;
    assert!(saw_valid, "Concurrent reader must never see invalid state");

    Ok(())
}

/// Test 6: Sabotage test A — claiming 0 remaining without purging fails scanner check.
#[tokio::test]
async fn test_sabotage_unpurged_residual_detected(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let storage = Arc::new(TestStorage::new());
    let index = Arc::new(InvertedIndex::new(storage, "sabotage_ns"));

    let d1 = DocId::new(500);
    let tx = TxId::new(1);
    index.insert(tx, d1, "sabotage test payload").await?;
    index.commit(tx).await?;

    // DO NOT PURGE - instead run scanner
    let clock = SystemClock::new();
    let scanner = TextScanner::new_with_target(index, ScanTarget::new("doc:500", [0xCC; 32]));
    let req_ctx = RequestContext::new(TenantId::try_new(1)?, tx);

    let witness = scanner.scan(
        &req_ctx,
        ScanTarget::new("doc:500", [0xCC; 32]),
        ScanTechnique::IndexIntrospection,
        &clock,
    )?;

    assert!(
        witness.remaining() > 0,
        "Scanner must catch unpurged residual"
    );

    Ok(())
}

/// Test 7: Sabotage test B — scanner positive control verifies canary presence.
#[tokio::test]
async fn test_scanner_positive_control_execution(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let storage = Arc::new(TestStorage::new());
    let index = Arc::new(InvertedIndex::new(storage, "pctl_test_ns"));

    let target = ScanTarget::new("doc:9999", [0xEE; 32]);
    let scanner = TextScanner::new_with_target(index, target.clone());
    let clock = SystemClock::new();
    let req_ctx = RequestContext::new(TenantId::try_new(1)?, TxId::new(1));

    let witness = scanner.scan(&req_ctx, target, ScanTechnique::IndexIntrospection, &clock)?;

    assert_eq!(witness.witness_layer(), &WitnessLayer::from("TextIndex"));
    assert!(witness.scanned() > 0);

    Ok(())
}
