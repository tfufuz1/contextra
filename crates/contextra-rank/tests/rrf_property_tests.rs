//! Property tests for Reciprocal Rank Fusion (RRF) using proptest.

use contextra_rank::fusion::{
    reciprocal_rank_fusion, weighted_reciprocal_rank_fusion,
    weighted_reciprocal_rank_fusion_with_options, MetadataMergePriority, SearchResult,
};
use proptest::prelude::*;

fn make_search_result(id: &str, score: f32) -> SearchResult {
    SearchResult {
        id: id.to_string(),
        score,
        metadata: None,
        matched_signals: vec!["signal".to_string()],
        provenance: None,
    }
}

// ANCHOR[contextra-rank:RRF-TEST-001]
proptest! {
    // Testet das erwartete Verhalten und mathematische Invarianten.
    #[test]
    fn prop_rrf_empty_lists_return_empty(max_results in 0usize..100) {
        let empty_sets: Vec<Vec<SearchResult>> = vec![vec![], vec![]];
        let fused = reciprocal_rank_fusion(empty_sets, max_results);
        prop_assert!(fused.is_empty());
    }

    // Testet das erwartete Verhalten und mathematische Invarianten.
    #[test]
    fn prop_rrf_monotonicity_single_list(
        num_items in 1usize..50,
        max_results in 1usize..100,
    ) {
        let list: Vec<SearchResult> = (0..num_items)
            .map(|i| make_search_result(&format!("doc_{i}"), (100 - i) as f32))
            .collect();

        let fused = reciprocal_rank_fusion(vec![list], max_results);
        let expected_len = num_items.min(max_results);
        prop_assert_eq!(fused.len(), expected_len);

        for window in fused.windows(2) {
            if let [a, b] = window {
                prop_assert!(a.score >= b.score, "Scores must be monotonically non-increasing");
            }
        }
    }

    // Testet das erwartete Verhalten und mathematische Invarianten.
    #[test]
    fn prop_rrf_permutation_invariance(
        perm in prop_oneof![
            Just(vec![0, 1, 2]),
            Just(vec![0, 2, 1]),
            Just(vec![1, 0, 2]),
            Just(vec![1, 2, 0]),
            Just(vec![2, 0, 1]),
            Just(vec![2, 1, 0]),
        ]
    ) {
        let list1: Vec<SearchResult> = (0..15)
            .map(|i| make_search_result(&format!("doc_{i}"), (20 - i) as f32))
            .collect();
        let list2: Vec<SearchResult> = (5..20)
            .map(|i| make_search_result(&format!("doc_{i}"), (25 - i) as f32))
            .collect();
        let list3: Vec<SearchResult> = (0..10)
            .map(|i| make_search_result(&format!("doc_{i}"), (15 - i) as f32))
            .collect();

        let set_tuples = vec![
            ("signal_a".to_string(), list1, 1.0),
            ("signal_b".to_string(), list2, 1.0),
            ("signal_c".to_string(), list3, 1.0),
        ];

        let fused_baseline = weighted_reciprocal_rank_fusion(set_tuples.clone(), 15);

        if let (Some(&i0), Some(&i1), Some(&i2)) = (perm.first(), perm.get(1), perm.get(2)) {
            if let (Some(s0), Some(s1), Some(s2)) = (set_tuples.get(i0), set_tuples.get(i1), set_tuples.get(i2)) {
                let permuted_tuples = vec![s0.clone(), s1.clone(), s2.clone()];
                let fused_permuted = weighted_reciprocal_rank_fusion(permuted_tuples, 15);

                prop_assert_eq!(fused_baseline.len(), fused_permuted.len());

                for (item_base, item_perm) in fused_baseline.iter().zip(fused_permuted.iter()) {
                    prop_assert_eq!(&item_base.id, &item_perm.id);
                    prop_assert!((item_base.score - item_perm.score).abs() < 1e-6);
                }
            }
        }
    }

    // Testet das erwartete Verhalten und mathematische Invarianten.
    #[test]
    fn prop_rrf_k_boundary_values(
        k_val in prop_oneof![Just(0.0), Just(1.0), Just(60.0), Just(1_000_000.0)],
        num_items in 1usize..20,
    ) {
        let set1: Vec<SearchResult> = (0..num_items)
            .map(|i| make_search_result(&format!("doc_{i}"), (num_items - i) as f32))
            .collect();

        let sets = vec![("sig1".to_string(), set1, 1.0)];
        let fused = weighted_reciprocal_rank_fusion_with_options(
            sets,
            100,
            MetadataMergePriority::default(),
            true,
            None,
        );

        prop_assert_eq!(fused.len(), num_items);
        for item in &fused {
            prop_assert!(item.score.is_finite(), "Score must be finite for k={k_val}");
            prop_assert!(item.score >= 0.0, "Score must be non-negative for k={k_val}");
        }
    }

    // Testet das erwartete Verhalten und mathematische Invarianten.
    #[test]
    fn prop_rrf_duplicate_ids_in_single_list(
        num_dups in 2usize..10,
    ) {
        let mut list = Vec::new();
        for _ in 0..num_dups {
            list.push(make_search_result("dup_doc", 0.9));
        }

        let fused = reciprocal_rank_fusion(vec![list], 10);
        prop_assert_eq!(fused.len(), 1, "Duplicate IDs in single list must be deduplicated into a single entry");
        prop_assert_eq!(fused.first().map(|f| f.id.as_str()), Some("dup_doc"));
    }
}

    // Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_prop_rrf_tie_breaker_stability_deterministic() {
    // Multiple documents with identical fusion scores must be tie-broken deterministically by DocId ascending.
    // doc_b (rank 1 in s1, rank 2 in s2) => score 1/61 + 1/62
    // doc_a (rank 2 in s1, rank 1 in s2) => score 1/62 + 1/61
    let list1 = vec![
        make_search_result("doc_z", 0.9),
        make_search_result("doc_b", 0.5),
        make_search_result("doc_a", 0.5),
    ];
    let list2 = vec![
        make_search_result("doc_z", 0.9),
        make_search_result("doc_a", 0.5),
        make_search_result("doc_b", 0.5),
    ];

    let fused = weighted_reciprocal_rank_fusion(
        vec![
            ("s1".to_string(), list1, 1.0),
            ("s2".to_string(), list2, 1.0),
        ],
        10,
    );

    assert_eq!(fused.len(), 3);
    assert_eq!(fused.first().map(|r| r.id.as_str()), Some("doc_z"));
    // doc_a and doc_b have exact identical RRF score sum (1/62 + 1/63) -> sorted alphabetically: doc_a then doc_b
    assert_eq!(fused.get(1).map(|r| r.id.as_str()), Some("doc_a"));
    assert_eq!(fused.get(2).map(|r| r.id.as_str()), Some("doc_b"));
}
