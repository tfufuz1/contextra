// FILE-CONTEXT
// STAND: 2026-09-15
// ZWECK: Property-based Tests für Retrieval- & Search-Invarianten in contextra-db (RRF, BM25, End-to-End).
// INVARIANTEN:
//   PROP-1: RRF-Ergebnisse <= k
//   PROP-2: E2E Suche (Collection / Contextra) <= k
//   PROP-3: BM25-Score monoton in Term-Frequenz (tf) bei sonst gleichen Parametern
//   PROP-4: RRF-Scores absteigend sortiert (fused[i].score >= fused[i+1].score)
//   PROP-5: Leere Eingaben & k=0 erzeugen niemals Panics und liefern leere Vektoren

use contextra_db::fusion::{reciprocal_rank_fusion, weighted_reciprocal_rank_fusion};
use contextra_db::{DistanceMetric, Contextra, ContextraConfig, SearchResult};
use contextra_text::bm25::score_term;
use proptest::prelude::*;
use tempfile::TempDir;
use tokio::runtime::Runtime;

// Strategy for generating arbitrary SearchResult objects with realistic or edge-case scores
fn search_result_strategy() -> impl Strategy<Value = SearchResult> {
    (0u64..50000, prop::option::of(prop::num::f32::ANY)).prop_map(|(id_num, score_opt)| {
        let score = score_opt.unwrap_or(0.0);
        SearchResult {
            id: format!("doc_{id_num}"),
            score,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        }
    })
}

// Strategy for generating lists of SearchResult sets
fn result_sets_strategy() -> impl Strategy<Value = Vec<Vec<SearchResult>>> {
    prop::collection::vec(
        prop::collection::vec(search_result_strategy(), 0..100),
        0..10,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    // =========================================================================
    // PROP-1: RRF-Ergebnisse <= k
    // =========================================================================
    #[test]
    fn prop_rrf_bounded_by_k(
        sets in result_sets_strategy(),
        k in 0usize..200,
    ) {
        let fused = reciprocal_rank_fusion(sets, k);
        prop_assert!(
            fused.len() <= k,
            "RRF output length {} exceeded requested max_results k={}",
            fused.len(),
            k
        );
    }

    // =========================================================================
    // PROP-2: Suche gibt niemals mehr als k Ergebnisse (End-to-End)
    // =========================================================================
    #[test]
    fn prop_e2e_search_bounded_by_k(
        doc_count in 1usize..100,
        k in 0usize..50,
        query_text in "[a-z]{3,10}",
    ) {
        // APM-1: Sync block_on wrapping a Tokio Runtime (KEIN #[tokio::test] in proptest!)
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async move {
            let tmp_dir = TempDir::new().expect("Failed to create tempdir");
            let config = ContextraConfig {
                dimension: 4,
                distance_metric: DistanceMetric::Cosine,
                ..Default::default()
            };

            let db = Contextra::open_with_config(tmp_dir.path(), config)
                .await
                .expect("Failed to open Contextra DB");

            let col = db.collection("prop_col").await.expect("Failed to get collection");

            // APM-3: Dummy vector embeddings without external embedder calls
            for i in 0..doc_count {
                let id = format!("doc_{i}");
                let dummy_embedding = vec![0.1 * (i as f32 + 1.0), 0.2, 0.3, 0.4];
                let doc_text = format!("sample document content {} {query_text}", i % 5);
                col.insert(&id, &dummy_embedding, Some(serde_json::json!({"text": doc_text})))
                    .await
                    .expect("Failed to insert document");
            }

            // Execute vector search
            let dummy_query_vector = vec![0.1, 0.2, 0.3, 0.4];
            let vec_results = col
                .query()
                .embedding(&dummy_query_vector)
                .k(k)
                .execute()
                .await
                .expect("Vector search failed");

            prop_assert!(
                vec_results.len() <= k,
                "End-to-end vector search returned {} results, exceeding k={}",
                vec_results.len(),
                k
            );

            // Execute hybrid search (vector + text query)
            let hybrid_results = col
                .query()
                .text(&query_text)
                .vector(&dummy_query_vector)
                .k(k)
                .execute()
                .await
                .expect("Hybrid search failed");

            prop_assert!(
                hybrid_results.len() <= k,
                "End-to-end hybrid search returned {} results, exceeding k={}",
                hybrid_results.len(),
                k
            );

            Ok::<(), TestCaseError>(())
        }).expect("Async block execution failed");
    }

    // =========================================================================
    // PROP-3: BM25-Score monoton in Term-Frequenz
    // =========================================================================
    #[test]
    fn prop_bm25_score_monotonic_in_tf(
        tf_a in 1u32..1000,
        tf_delta in 0u32..1000,
        doc_len in 1u32..10000,
        avg_doc_len in 1.0f32..10000.0f32,
        df in 1u32..10000,
        n_delta in 0u32..10000,
    ) {
        let tf_b = tf_a + tf_delta;
        let n = df + n_delta;

        let score_a = score_term(tf_a, doc_len, avg_doc_len, df, n);
        let score_b = score_term(tf_b, doc_len, avg_doc_len, df, n);

        prop_assert!(score_a.is_finite(), "Score A must be finite");
        prop_assert!(score_b.is_finite(), "Score B must be finite");
        prop_assert!(
            score_b >= score_a,
            "BM25 monotonicity failed: score(tf_b={}) = {} < score(tf_a={}) = {} for doc_len={}, avg_doc_len={}, df={}, n={}",
            tf_b,
            score_b,
            tf_a,
            score_a,
            doc_len,
            avg_doc_len,
            df,
            n
        );
    }

    // =========================================================================
    // PROP-4: RRF-Scores absteigend sortiert
    // =========================================================================
    #[test]
    fn prop_rrf_scores_sorted_descending(
        sets in result_sets_strategy(),
        k in 1usize..100,
    ) {
        let fused = reciprocal_rank_fusion(sets, k);
        for i in 0..fused.len().saturating_sub(1) {
            let score_curr = fused[i].score;
            let score_next = fused[i + 1].score;

            // Finite scores must be strictly descending or equal
            if score_curr.is_finite() && score_next.is_finite() {
                prop_assert!(
                    score_curr >= score_next,
                    "RRF scores not sorted descending: index {} (score {}) < index {} (score {})",
                    i,
                    score_curr,
                    i + 1,
                    score_next
                );
            }
        }
    }

    // =========================================================================
    // PROP-5: Leere Eingabe nie Panic
    // =========================================================================
    #[test]
    fn prop_rrf_empty_inputs_never_panic(
        k in 0usize..(usize::MAX / 2),
        weights in prop::collection::vec(-10.0f32..10.0f32, 0..5),
    ) {
        // Case 1: Empty outer vector
        let res1 = reciprocal_rank_fusion(vec![], k);
        prop_assert!(res1.is_empty(), "Empty input sets must yield empty results");

        // Case 2: Outer vector with empty inner sets
        let res2 = reciprocal_rank_fusion(vec![vec![], vec![]], k);
        prop_assert!(res2.is_empty(), "Sets of empty inner vectors must yield empty results");

        // Case 3: k = 0 with populated or empty sets
        let dummy_set = vec![SearchResult {
            id: "doc_1".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        }];
        let res3 = reciprocal_rank_fusion(vec![dummy_set.clone()], 0);
        prop_assert!(res3.is_empty(), "max_results k=0 must return empty results");

        // Case 4: Weighted RRF with empty sets or k=0
        let weighted_sets: Vec<(String, Vec<SearchResult>, f32)> = weights
            .into_iter()
            .enumerate()
            .map(|(i, w)| (format!("signal_{i}"), if i % 2 == 0 { vec![] } else { dummy_set.clone() }, w))
            .collect();
        let res4 = weighted_reciprocal_rank_fusion(weighted_sets, 0);
        prop_assert!(res4.is_empty(), "Weighted RRF with k=0 must return empty results");
    }
}
