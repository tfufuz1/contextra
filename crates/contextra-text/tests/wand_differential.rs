// FILE-CONTEXT: WAND Differential Test Suite against Exhaustive BM25 Reference.
// ZWECK: Verifiziert Block-Max WAND Optimierung gegen naive, vollstaendige BM25-Berechnung ueber alle Dokumente.
// INVARIANTEN: Zero-Panic Doctrine, deterministische Seeds (Regel 3: keine Systemzeit/Systemzufall), identische Rangfolge.

use contextra_ports::{BoxFuture, StorageEngine, TextIndex};
use contextra_text::bm25::score_term;
use contextra_text::inverted::InvertedIndex;
use contextra_text::tokenizer::{DefaultTokenizer, Tokenizer};
use contextra_types::{DocId, Result, TxId};
use proptest::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Default)]
struct TestRamStorage {
    store: parking_lot::RwLock<HashMap<Vec<u8>, Vec<(Vec<u8>, u64)>>>,
    next_seq: AtomicU64,
}

impl StorageEngine for TestRamStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
    }

    fn put<'a>(&'a self, _tx: TxId, key: &'a [u8], val: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
            self.store
                .write()
                .entry(key.to_vec())
                .or_default()
                .push((val.to_vec(), seq));
            Ok(())
        })
    }

    fn delete<'a>(&'a self, _tx: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
            self.store
                .write()
                .entry(key.to_vec())
                .or_default()
                .push((Vec::new(), seq | contextra_types::TOMBSTONE_BIT));
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

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(self.next_seq.load(Ordering::SeqCst)) })
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
        _: std::ops::Bound<&'a [u8]>,
        _: std::ops::Bound<&'a [u8]>,
        _: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.scan_prefix_at(prefix, u64::MAX)
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let store = self.store.read();
            let mut res = Vec::new();
            for (k, versions) in store.iter() {
                if k.starts_with(prefix) {
                    for (val, v_seq) in versions.iter().rev() {
                        let raw_seq = v_seq & !contextra_types::TOMBSTONE_BIT;
                        if raw_seq <= seq {
                            if (v_seq & contextra_types::TOMBSTONE_BIT) == 0 {
                                res.push((k.clone(), val.clone()));
                            }
                            break;
                        }
                    }
                }
            }
            Ok(res)
        })
    }
}

/// Naive reference evaluator:
/// Computes exact BM25 scores for all documents across query tokens,
/// applying the same tokenizer and BM25 formula as InvertedIndex.
/// Enforces documented tie-breaking: score descending, DocId ascending.
fn naive_bm25_reference(
    docs: &BTreeMap<DocId, Vec<String>>,
    query: &str,
    k: usize,
) -> Vec<(DocId, f32)> {
    if query.is_empty() || k == 0 || docs.is_empty() {
        return Vec::new();
    }

    let query_tokens = DefaultTokenizer.tokenize(query);
    if query_tokens.is_empty() {
        return Vec::new();
    }

    let total_docs = docs.len() as u32;
    let total_doc_tokens: usize = docs.values().map(|toks| toks.len()).sum();
    // InvertedIndex stores avg_doc_len as fixed-point x1000 in AtomicU64 (FIND-TXT-004)
    let avg_doc_len = if total_docs > 0 {
        let avg_x1000 = (total_doc_tokens as f64 / total_docs as f64 * 1000.0) as u64;
        avg_x1000 as f32 / 1000.0
    } else {
        0.0
    };

    // Precompute document frequencies (df) for unique query tokens across the corpus
    let mut df_map: HashMap<String, u32> = HashMap::new();
    for token in &query_tokens {
        if !df_map.contains_key(token) {
            let df = docs
                .values()
                .filter(|toks| toks.iter().any(|t| t == token))
                .count() as u32;
            df_map.insert(token.clone(), df);
        }
    }

    let mut scored: Vec<(DocId, f32)> = Vec::new();

    for (&doc_id, toks) in docs {
        let doc_len = toks.len() as u32;
        let mut doc_score = 0.0f32;
        let mut matched = false;

        // Note: For each query token (including duplicates), evaluate score_term
        for token in &query_tokens {
            let tf = toks.iter().filter(|&t| t == token).count() as u32;
            if tf > 0 {
                let df = df_map.get(token).copied().unwrap_or(0);
                if df > 0 {
                    let score = score_term(tf, doc_len, avg_doc_len, df, total_docs);
                    doc_score += score;
                    matched = true;
                }
            }
        }

        if matched && doc_score > 0.0 {
            scored.push((doc_id, doc_score));
        }
    }

    // Deterministic tie-breaking: score descending, DocId ascending
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    scored.truncate(k);
    scored
}

/// Asserts that WAND search results match the naive reference.
/// Tolerates small float rounding differences (< 1e-4) in score summation,
/// but strictly enforces exact ranking order and DocId matching.
fn assert_wand_matches_reference(
    wand_results: &[(DocId, f32)],
    reference_results: &[(DocId, f32)],
    scenario: &str,
) {
    assert_eq!(
        wand_results.len(),
        reference_results.len(),
        "Result count mismatch in {}: WAND count {} != Ref count {}",
        scenario,
        wand_results.len(),
        reference_results.len()
    );

    for (rank, (wand_item, ref_item)) in wand_results
        .iter()
        .zip(reference_results.iter())
        .enumerate()
    {
        assert_eq!(
            wand_item.0, ref_item.0,
            "Rank {} DocId mismatch in {}: WAND {:?} vs Ref {:?}",
            rank, scenario, wand_item, ref_item
        );
        let score_diff = (wand_item.1 - ref_item.1).abs();
        assert!(
            score_diff < 1e-4,
            "Rank {} Score difference {:.6} exceeds tolerance in {}: WAND {} vs Ref {} for doc {:?}",
            rank, score_diff, scenario, wand_item.1, ref_item.1, wand_item.0
        );
    }
}

#[tokio::test]
async fn test_wand_edge_cases_deterministic() -> Result<()> {
    // 1. Empty Index Test
    {
        let storage = Arc::new(TestRamStorage::default());
        let index = InvertedIndex::new(storage, "empty_idx");
        let results = index.search_bm25("any query terms", 10, None).await?;
        assert!(results.is_empty(), "Empty index must return 0 results");
    }

    // 2. Query with no matches (terms without hits)
    {
        let storage = Arc::new(TestRamStorage::default());
        let index = InvertedIndex::new(storage, "no_hits_idx");
        let tx = TxId::new(1);
        index
            .insert(tx, DocId::new(1), "apple banana cherry")
            .await?;
        index.commit(tx).await?;

        let results = index
            .search_bm25("nonexistent unknown xyz", 10, None)
            .await?;
        assert!(results.is_empty(), "Unmatched query must return 0 results");
    }

    // 3. Query duplicates ("apple apple banana banana apple")
    {
        let storage = Arc::new(TestRamStorage::default());
        let index = InvertedIndex::new(storage, "dup_query_idx");
        let tx = TxId::new(1);
        let mut docs = BTreeMap::new();

        let doc1_text = "apple banana";
        let doc2_text = "apple apple apple";
        let doc3_text = "banana banana";

        docs.insert(DocId::new(1), DefaultTokenizer.tokenize(doc1_text));
        docs.insert(DocId::new(2), DefaultTokenizer.tokenize(doc2_text));
        docs.insert(DocId::new(3), DefaultTokenizer.tokenize(doc3_text));

        index.insert(tx, DocId::new(1), doc1_text).await?;
        index.insert(tx, DocId::new(2), doc2_text).await?;
        index.insert(tx, DocId::new(3), doc3_text).await?;
        index.commit(tx).await?;

        let query = "apple apple banana";
        for k in [1, 2, 3, 10] {
            let wand_res = index.search_bm25(query, k, None).await?;
            let ref_res = naive_bm25_reference(&docs, query, k);
            assert_wand_matches_reference(&wand_res, &ref_res, &format!("duplicate query k={}", k));
        }
    }

    // 4. Identical score tie-breaking across multiple documents
    {
        let storage = Arc::new(TestRamStorage::default());
        let index = InvertedIndex::new(storage, "tie_break_idx");
        let tx = TxId::new(1);
        let mut docs = BTreeMap::new();

        // 8 documents with identical contents
        let ids = [100u64, 42, 10, 88, 5, 23, 77, 12];
        for &id in &ids {
            let text = "common term across identical documents";
            docs.insert(DocId::new(id), DefaultTokenizer.tokenize(text));
            index.insert(tx, DocId::new(id), text).await?;
        }
        index.commit(tx).await?;

        let query = "common identical";
        for k in [1, 2, 4, 8, 20] {
            let wand_res = index.search_bm25(query, k, None).await?;
            let ref_res = naive_bm25_reference(&docs, query, k);
            assert_wand_matches_reference(&wand_res, &ref_res, &format!("tie-break k={}", k));

            // Verify strictly increasing DocId in tie-broken output
            for window in wand_res.windows(2) {
                if let [a, b] = window {
                    assert!(
                        a.0 < b.0,
                        "Tied documents must be sorted by DocId ascending: {:?} vs {:?}",
                        a.0,
                        b.0
                    );
                }
            }
        }
    }

    // 5. Skewed posting list lengths (1 frequent term, 7 rare terms)
    {
        let storage = Arc::new(TestRamStorage::default());
        let index = InvertedIndex::new(storage, "skewed_idx");
        let tx = TxId::new(1);
        let mut docs = BTreeMap::new();

        // 60 documents: "omnipresent" is in all 60.
        // "rare1" in doc 5, "rare2" in doc 15, "rare3" in doc 25, "rare4" in doc 35...
        for i in 1..=60 {
            let doc_id = DocId::new(i);
            let mut text = "omnipresent regular content ".to_string();
            if i == 5 {
                text.push_str("rare1 rare1 rare1 ");
            }
            if i == 15 {
                text.push_str("rare2 rare2 ");
            }
            if i == 25 {
                text.push_str("rare3 rare3 rare3 rare3 ");
            }
            if i == 35 {
                text.push_str("rare4 ");
            }
            if i == 45 {
                text.push_str("rare5 rare5 ");
            }
            if i == 55 {
                text.push_str("rare6 ");
            }

            docs.insert(doc_id, DefaultTokenizer.tokenize(&text));
            index.insert(tx, doc_id, &text).await?;
        }
        index.commit(tx).await?;

        // 8 query terms (omnipresent + rare1..rare6 + missing_rare7)
        let query = "omnipresent rare1 rare2 rare3 rare4 rare5 rare6 missing_rare7";
        for k in [1, 2, 5, 10, 100] {
            let wand_res = index.search_bm25(query, k, None).await?;
            let ref_res = naive_bm25_reference(&docs, query, k);
            assert_wand_matches_reference(&wand_res, &ref_res, &format!("skewed lengths k={}", k));
        }
    }

    Ok(())
}

// Fixed seed deterministic proptest suite.
// Rule 3: No system time or system RNG. Use fixed deterministic runner configuration.
proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 80,
        rng_algorithm: proptest::test_runner::RngAlgorithm::ChaCha,
        failure_persistence: None,
        .. proptest::test_runner::Config::default()
    })]

    #[test]
    fn prop_wand_differential_randomized_corpus(
        num_docs in 5..35usize,
        k_val in prop_oneof![Just(1usize), Just(2usize), Just(10usize), 15..50usize],
        query_word_count in 1..=8usize,
        seed in 1u64..10_000u64,
    ) {
        let vocab = [
            "alpha", "beta", "gamma", "delta", "epsilon", "zeta",
            "eta", "theta", "iota", "kappa", "lambda", "mu",
        ];

        // Seeded deterministic pseudo-random corpus generation (LCG generator)
        let mut rng_state = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut next_u32 = || {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (rng_state >> 32) as u32
        };

        let mut docs: BTreeMap<DocId, Vec<String>> = BTreeMap::new();
        let mut doc_texts: Vec<(DocId, String)> = Vec::with_capacity(num_docs);

        for i in 1..=num_docs {
            let doc_id = DocId::new(i as u64);
            let doc_words_count = (next_u32() % 25) as usize + 2;
            let mut words = Vec::with_capacity(doc_words_count);
            for _ in 0..doc_words_count {
                let word_idx = (next_u32() as usize) % vocab.len();
                words.push(vocab[word_idx]);
            }
            let text = words.join(" ");
            docs.insert(doc_id, DefaultTokenizer.tokenize(&text));
            doc_texts.push((doc_id, text));
        }

        // Build query of 1 to 8 terms (which may include vocabulary terms, duplicates, and unknown terms)
        let mut query_words = Vec::with_capacity(query_word_count);
        for _ in 0..query_word_count {
            let choice = next_u32() % (vocab.len() + 2) as u32;
            if choice < vocab.len() as u32 {
                query_words.push(vocab[choice as usize]);
            } else {
                query_words.push("unseen_extra_term");
            }
        }
        let query = query_words.join(" ");

        // Execute asynchronous test using single-threaded futures executor block_on
        futures::executor::block_on(async {
            let storage = Arc::new(TestRamStorage::default());
            let index = InvertedIndex::new(storage, "prop_wand_test");
            let tx = TxId::new(1);

            for (doc_id, text) in &doc_texts {
                index.insert(tx, *doc_id, text).await.expect("insert doc");
            }
            index.commit(tx).await.expect("commit tx");

            let wand_results = index.search_bm25(&query, k_val, None).await.expect("wand search");
            let ref_results = naive_bm25_reference(&docs, &query, k_val);

            assert_wand_matches_reference(
                &wand_results,
                &ref_results,
                &format!("proptest num_docs={} k={} query={}", num_docs, k_val, query)
            );
        });
    }
}
