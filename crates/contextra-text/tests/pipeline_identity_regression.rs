//! Regression test for Tokenizer Pipeline Identity (P1), Snapshot Isolation (P4), and Tx Rollback (P5).

use contextra_ports::{BoxFuture, StorageEngine, TextIndex};
use contextra_text::inverted::{InvertedIndex, Language};
use contextra_text::tokenizer::{GermanMorphTokenizer, Tokenizer};
use contextra_types::{DocId, Result, TxId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

type MockStoreMap = RwLock<HashMap<Vec<u8>, Vec<(Vec<u8>, u64)>>>;

struct TestMVCCStorage {
    store: MockStoreMap,
    staged: RwLock<HashMap<TxId, Vec<Vec<u8>>>>,
    next_seq: std::sync::atomic::AtomicU64,
}

impl TestMVCCStorage {
    fn new() -> Self {
        Self {
            store: MockStoreMap::new(HashMap::new()),
            staged: RwLock::new(HashMap::new()),
            next_seq: std::sync::atomic::AtomicU64::new(1),
        }
    }
}

impl StorageEngine for TestMVCCStorage {
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

/// P1: Pipeline Identity Regression Test
///
/// Indexes a document containing German umlauts, compounds, and protected special characters
/// (URL / email). For EVERY generated token, performs a point search to verify 100% recall.
#[tokio::test]
async fn test_pipeline_identity_point_search_recall() -> Result<()> {
    let storage = Arc::new(TestMVCCStorage::new());
    let index = InvertedIndex::new_with_language(storage.clone(), "p1_ns", Language::German);

    let doc_id = DocId::new(42);
    let tx = TxId::new(1);
    let sample_text = "Die Softwarearchitektur des Bundesverfassungsgerichts wurde in Bär-München unter support@contextra.ai und https://contextra.ai/v1 revidiert.";

    index.insert(tx, doc_id, sample_text).await?;
    index.commit(tx).await?;

    let tokenizer = GermanMorphTokenizer::new();
    let tokens = tokenizer.tokenize(sample_text);

    assert!(!tokens.is_empty(), "Tokenization must yield tokens");

    for token in &tokens {
        let results = index.search(token, 10).await?;
        assert!(
            results.iter().any(|r| r.doc_id == doc_id),
            "Point search for token '{}' failed to find document {}",
            token,
            doc_id
        );
    }

    Ok(())
}

/// P4: Snapshot Isolation (search_at) Test
///
/// Doc A at seq=1, Doc A' (same ID, different text) at seq=2.
/// search_at(query_A, seq=1) MUST find A, not A'.
#[tokio::test]
async fn test_snapshot_isolation_same_doc_id_update() -> Result<()> {
    let storage = Arc::new(TestMVCCStorage::new());
    let index = InvertedIndex::new_with_language(storage.clone(), "p4_ns", Language::German);

    let doc_id = DocId::new(100);

    // Step 1: Insert Doc A at tx1
    let tx1 = TxId::new(1);
    index.insert(tx1, doc_id, "Softwarearchitektur Alphatest").await?;
    index.commit(tx1).await?;
    let seq_a = storage.last_seq_no().await?;

    // Step 2: Update same doc_id to Doc A' at tx2
    let tx2 = TxId::new(2);
    index.insert(tx2, doc_id, "Datenbankverbindung Betatest").await?;
    index.commit(tx2).await?;
    let seq_a_prime = storage.last_seq_no().await?;

    // Query at historical seq_a for "Softwarearchitektur" MUST find doc_id
    let res_seq_a = index.search_at("Softwarearchitektur", 10, seq_a).await?;
    assert_eq!(res_seq_a.len(), 1);
    assert_eq!(res_seq_a[0].doc_id, doc_id);

    // Query at historical seq_a for "Datenbankverbindung" MUST NOT find doc_id
    let res_beta_at_seq_a = index.search_at("Datenbankverbindung", 10, seq_a).await?;
    assert!(res_beta_at_seq_a.is_empty(), "Datenbankverbindung was inserted after seq_a");

    // Query at latest seq_a_prime for "Softwarearchitektur" should be masked/updated by tombstone if replaced, or found if term preserved
    let res_beta_at_latest = index.search_at("Datenbankverbindung", 10, seq_a_prime).await?;
    assert_eq!(res_beta_at_latest.len(), 1);
    assert_eq!(res_beta_at_latest[0].doc_id, doc_id);

    Ok(())
}

/// P5: Transaction Rollback Test
///
/// Mutation under TxId T, Rollback of T -> text index entry is removed.
#[tokio::test]
async fn test_transaction_rollback_cleans_entry() -> Result<()> {
    let storage = Arc::new(TestMVCCStorage::new());
    let index = InvertedIndex::new_with_language(storage.clone(), "p5_ns", Language::German);

    let doc_id = DocId::new(999);
    let tx = TxId::new(55);

    index.insert(tx, doc_id, "Sicherheitskonzept und Datenschutz").await?;

    // Perform rollback
    index.rollback(tx).await?;

    // Search should return nothing
    let results = index.search("Sicherheitskonzept", 10).await?;
    assert!(results.is_empty(), "Rollback must remove staged text index entries");

    Ok(())
}
