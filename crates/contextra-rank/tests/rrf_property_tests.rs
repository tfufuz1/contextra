//! Property tests for Reciprocal Rank Fusion (RRF) using proptest (T-RANK-1).

#![allow(deprecated)]

use contextra_rank::fusion::{reciprocal_rank_fusion, rrf_fuse, Hit, SearchResult};
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
    // T-RANK-1: Leere Listen liefern leere Fusionsergebnisse
    #[test]
    fn prop_rrf_empty_lists_return_empty(max_results in 0usize..100) {
        let empty_sets: Vec<Vec<SearchResult>> = vec![vec![], vec![]];
        let fused = reciprocal_rank_fusion(empty_sets, max_results);
        prop_assert!(fused.is_empty());

        let empty_hits: Vec<Vec<Hit>> = vec![vec![], vec![]];
        let fused_hits = rrf_fuse(&empty_hits, 60, max_results);
        prop_assert!(fused_hits.is_empty());
    }

    // T-RANK-1: Monotonie (Scores fallen monoton ab)
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

        let hit_list: Vec<Hit> = (0..num_items)
            .map(|i| Hit::new(format!("doc_{i}"), (100 - i) as f32))
            .collect();

        let fused_hits = rrf_fuse(&[hit_list], 60, max_results);
        prop_assert_eq!(fused_hits.len(), expected_len);
        for window in fused_hits.windows(2) {
            if let [a, b] = window {
                prop_assert!(a.score >= b.score, "Scores must be monotonically non-increasing in rrf_fuse");
            }
        }
    }

    // T-RANK-1: Strikte Permutationsinvarianz bezüglich der Reihenfolge der Eingabelisten
    #[test]
    fn prop_rrf_fuse_permutation_invariance(
        perm in prop_oneof![
            Just(vec![0, 1, 2]),
            Just(vec![0, 2, 1]),
            Just(vec![1, 0, 2]),
            Just(vec![1, 2, 0]),
            Just(vec![2, 0, 1]),
            Just(vec![2, 1, 0]),
        ]
    ) {
        let list1: Vec<Hit> = (0..15)
            .map(|i| Hit::new(format!("doc_{i}"), (20 - i) as f32))
            .collect();
        let list2: Vec<Hit> = (5..20)
            .map(|i| Hit::new(format!("doc_{i}"), (25 - i) as f32))
            .collect();
        let list3: Vec<Hit> = (0..10)
            .map(|i| Hit::new(format!("doc_{i}"), (15 - i) as f32))
            .collect();

        let all_lists = vec![list1, list2, list3];
        let fused_baseline = rrf_fuse(&all_lists, 60, 15);

        if let (Some(&i0), Some(&i1), Some(&i2)) = (perm.first(), perm.get(1), perm.get(2)) {
            if let (Some(l0), Some(l1), Some(l2)) = (all_lists.get(i0), all_lists.get(i1), all_lists.get(i2)) {
                let permuted_lists = vec![l0.clone(), l1.clone(), l2.clone()];
                let fused_permuted = rrf_fuse(&permuted_lists, 60, 15);

                prop_assert_eq!(fused_baseline.len(), fused_permuted.len());

                for (item_base, item_perm) in fused_baseline.iter().zip(fused_permuted.iter()) {
                    prop_assert_eq!(&item_base.id, &item_perm.id);
                    prop_assert!((item_base.score - item_perm.score).abs() < 1e-6);
                }
            }
        }
    }

    // T-RANK-1: Randwerte für Glättungsfaktor k
    #[test]
    fn prop_rrf_fuse_k_boundaries(
        k_val in prop_oneof![Just(0u32), Just(1u32), Just(60u32), Just(1_000_000u32)],
        num_items in 1usize..20,
    ) {
        let set1: Vec<Hit> = (0..num_items)
            .map(|i| Hit::new(format!("doc_{i}"), (num_items - i) as f32))
            .collect();

        let fused = rrf_fuse(&[set1], k_val, 100);
        prop_assert_eq!(fused.len(), num_items);
        for item in &fused {
            prop_assert!(item.score.is_finite(), "Score must be finite for k={k_val}");
            prop_assert!(item.score > 0.0, "Score must be strictly positive for k={k_val}");
        }
    }

    // T-RANK-1: Duplikat-Deduplizierung innerhalb und über Listen
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

        let mut hit_list = Vec::new();
        for _ in 0..num_dups {
            hit_list.push(Hit::new("dup_doc", 0.9));
        }
        let fused_hits = rrf_fuse(&[hit_list], 60, 10);
        prop_assert_eq!(fused_hits.len(), 1);
        prop_assert_eq!(fused_hits.first().map(|f| f.id.as_str()), Some("dup_doc"));
    }
}

// T-RANK-1: Deterministischer Tie-Breaker (nach doc_tag aufsteigend bzw. id aufsteigend)
#[test]
fn test_prop_rrf_tie_breaker_stability_deterministic() {
    let tag_a = [0x01; 16];
    let tag_b = [0x02; 16];

    let list1 = vec![
        Hit::new("doc_z", 0.9),
        Hit::new("doc_b", 0.5).with_doc_tag(tag_b),
        Hit::new("doc_a", 0.5).with_doc_tag(tag_a),
    ];
    let list2 = vec![
        Hit::new("doc_z", 0.9),
        Hit::new("doc_a", 0.5).with_doc_tag(tag_a),
        Hit::new("doc_b", 0.5).with_doc_tag(tag_b),
    ];

    let fused = rrf_fuse(&[list1, list2], 60, 10);

    assert_eq!(fused.len(), 3);
    assert_eq!(fused.first().map(|h| h.id.as_str()), Some("doc_z"));
    // doc_a und doc_b haben identischen RRF-Score (1/62 + 1/63)
    // tag_a < tag_b -> doc_a steht strikt vor doc_b
    assert_eq!(fused.get(1).map(|h| h.id.as_str()), Some("doc_a"));
    assert_eq!(fused.get(2).map(|h| h.id.as_str()), Some("doc_b"));
}
