//! Property-based tests for BM25 score invariants and tombstone isolation.

use memfuse_core::{BoxFuture, DocId, Result, StorageEngine, TextIndex, TxId};
use memfuse_text::bm25::score_term;
use memfuse_text::tokenizer::{DefaultTokenizer, Tokenizer};
use memfuse_text::InvertedIndex;
use parking_lot::RwLock;
use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

type MockStoreMap = RwLock<HashMap<Vec<u8>, Vec<(Vec<u8>, u64)>>>;

/// In-memory MVCC MockStorage for testing InvertedIndex snapshot isolation and tombstone behavior.
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

proptest! {
    // PROP-1: BM25-Score strikt positiv für vorhandene Terme
    // Für tf ≥ 1, doc_len ≥ 1, avg_doc_len > 0, df ≥ 1, n ≥ df:
    // score_term(tf, doc_len, avg_doc_len, df, n) > 0.0.
    #[test]
    fn prop_1_bm25_score_strictly_positive_for_present_terms(
        tf in 1..10_000u32,
        doc_len in 1..100_000u32,
        avg_doc_len in 0.001f32..10_000.0f32,
        df in 1..10_000u32,
        n_delta in 0..100_000u32,
    ) {
        let n = df + n_delta; // Guaranteed n >= df
        let score = score_term(tf, doc_len, avg_doc_len, df, n);
        prop_assert!(
            score > 0.0,
            "Expected strictly positive score for tf={}, doc_len={}, avg_doc_len={}, df={}, n={}, got {}",
            tf, doc_len, avg_doc_len, df, n, score
        );
    }

    // PROP-2: BM25-Score monoton in tf (bei fixem doc_len)
    // tf1 < tf2 -> score(tf1, ...) < score(tf2, ...).
    #[test]
    fn prop_2_bm25_score_monotone_increasing_in_tf(
        tf1 in 1..5_000u32,
        tf_delta in 1..5_000u32,
        doc_len in 1..100_000u32,
        avg_doc_len in 0.001f32..10_000.0f32,
        df in 1..10_000u32,
        n_delta in 0..100_000u32,
    ) {
        let tf2 = tf1 + tf_delta;
        let n = df + n_delta;
        let score1 = score_term(tf1, doc_len, avg_doc_len, df, n);
        let score2 = score_term(tf2, doc_len, avg_doc_len, df, n);
        prop_assert!(
            score1 < score2,
            "Expected score(tf1={}) < score(tf2={}), got score1={}, score2={}",
            tf1, tf2, score1, score2
        );
    }

    // PROP-3: BM25-Score monoton abfallend in doc_len (bei fixem tf)
    // doc_len1 < doc_len2 -> score(tf, doc_len1, ...) > score(tf, doc_len2, ...).
    //
    // MATHEMATISCHE BEGRÜNDUNG:
    // Der BM25 Term-Score ist definiert als:
    //   Score = IDF * [ tf * (k1 + 1) / (tf + k1 * (1 - b + b * (doc_len / avg_doc_len))) ]
    //
    // Da tf >= 1, k1 = 1.5 > 0, b = 0.75 > 0, und avg_doc_len > 0 ist, hängt der Nenner
    // D(doc_len) = tf + k1*(1 - b) + k1*b*(doc_len / avg_doc_len) streng monoton steigend
    // von doc_len ab, da dD/d(doc_len) = (k1 * b) / avg_doc_len > 0.
    //
    // Der Zähler N = tf * (k1 + 1) > 0 und die IDF = ln(1 + (N - df + 0.5)/(df + 0.5)) > 0 (für df <= n)
    // sind invariant bezüglich doc_len.
    // Dementsprechend gilt für d(Score)/d(doc_len) = -N * IDF * (k1 * b / avg_doc_len) / (D(doc_len))^2 < 0.
    // Somit ist die Score-Funktion bezüglich doc_len streng monoton fallend:
    // doc_len1 < doc_len2 ==> score(tf, doc_len1, ...) > score(tf, doc_len2, ...).
    #[test]
    fn prop_3_bm25_score_monotone_decreasing_in_doc_len(
        tf in 1..10_000u32,
        doc_len1 in 1..50_000u32,
        doc_len_delta in 1..50_000u32,
        avg_doc_len in 0.001f32..10_000.0f32,
        df in 1..10_000u32,
        n_delta in 0..100_000u32,
    ) {
        let doc_len2 = doc_len1 + doc_len_delta;
        let n = df + n_delta;
        let score1 = score_term(tf, doc_len1, avg_doc_len, df, n);
        let score2 = score_term(tf, doc_len2, avg_doc_len, df, n);
        prop_assert!(
            score1 > score2,
            "Expected score(doc_len1={}) > score(doc_len2={}), got score1={}, score2={}",
            doc_len1, doc_len2, score1, score2
        );
    }

    // PROP-4: Keine NaN-Ausgabe der Score-Funktion
    // Für beliebige valide Inputs (tf ∈ [1..1000], doc_len ∈ [1..10000], avg_doc_len ∈ [1.0..1000.0], df ∈ [1..10000], n ∈ [df..100000]):
    // !score_term(...).is_nan().
    #[test]
    fn prop_4_no_nan_output_for_valid_inputs(
        tf in 1..1000u32,
        doc_len in 1..10_000u32,
        avg_doc_len in 1.0f32..1000.0f32,
        df in 1..10_000u32,
        n_delta in 0..90_000u32,
    ) {
        let n = (df + n_delta).min(100_000);
        let score = score_term(tf, doc_len, avg_doc_len, df, n);
        prop_assert!(
            !score.is_nan(),
            "Score must not be NaN for tf={}, doc_len={}, avg_doc_len={}, df={}, n={}",
            tf, doc_len, avg_doc_len, df, n
        );
        prop_assert!(
            score.is_finite(),
            "Score must be finite for tf={}, doc_len={}, avg_doc_len={}, df={}, n={}",
            tf, doc_len, avg_doc_len, df, n
        );
    }

    // PROP-5: Tombstone-Isolation (Snapshot-Semantik)
    // Async Proptest: Insert-Dokument bei seq=S1, Delete bei seq=S2 > S1.
    // - search_bm25_at(q, k, Some(S1)) enthält DocId.
    // - search_bm25_at(q, k, Some(S2)) enthält DocId NICHT.
    // Invariante gilt für beliebige Queries [a-z]{2..8} und k ∈ [1..20].
    #[test]
    fn prop_5_tombstone_isolation_snapshot_semantics(
        q in "[a-z]{2,8}",
        k in 1..20usize,
        doc_id_raw in 1..1_000_000u64,
    ) {
        prop_assume!(!DefaultTokenizer.tokenize(&q).is_empty());

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let storage = Arc::new(MVCCMockStorage::new());
            let index = InvertedIndex::new(storage.clone(), "prop5_ns");
            let doc_id = DocId::new(doc_id_raw);

            // Step 1: Insert document with the query text `q` at transaction tx1
            let tx1 = TxId::new(1);
            index.insert(tx1, doc_id, &q).await.unwrap();
            index.commit(tx1).await.unwrap();
            let s1 = storage.last_seq_no().await.unwrap();

            // Step 2: Delete document at transaction tx2
            let tx2 = TxId::new(2);
            index.delete(tx2, doc_id).await.unwrap();
            index.commit(tx2).await.unwrap();
            let s2 = storage.last_seq_no().await.unwrap();

            prop_assert!(s2 > s1, "Deletion sequence S2 ({}) must be greater than insertion S1 ({})", s2, s1);

            // Search at snapshot S1 MUST contain doc_id
            let results_s1 = index.search_bm25_at(&q, k, Some(s1)).await.unwrap();
            let contains_s1 = results_s1.iter().any(|(id, _)| *id == doc_id);
            prop_assert!(
                contains_s1,
                "Snapshot at S1 ({}) must contain doc_id {} for query '{}'",
                s1, doc_id_raw, q
            );

            // Search at snapshot S2 MUST NOT contain doc_id
            let results_s2 = index.search_bm25_at(&q, k, Some(s2)).await.unwrap();
            let contains_s2 = results_s2.iter().any(|(id, _)| *id == doc_id);
            prop_assert!(
                !contains_s2,
                "Snapshot at S2 ({}) must NOT contain deleted doc_id {} for query '{}'",
                s2, doc_id_raw, q
            );

            Ok(())
        })?;
    }
}
