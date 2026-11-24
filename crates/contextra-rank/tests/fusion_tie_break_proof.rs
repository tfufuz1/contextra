//! Umfassende Beweis-Suite für Tie-Breaking und Determinismus bei allen Fusionspfaden (WP-4).
//!
//! Kernversprechen (AGENTS.md §1.1.4):
//! Gleiche Eingabe ⇒ gleiche Rangfolge; Gleichstand ⇒ ID aufsteigend.

#![allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use contextra_rank::fusion::{
    reciprocal_rank_fusion, rrf_fuse, score_normalized_fusion_with_options,
    weighted_reciprocal_rank_fusion, MetadataMergePriority, ScoredItem,
    SearchResult,
};
use proptest::prelude::*;

fn make_res(id: &str, score: f32) -> SearchResult {
    SearchResult {
        id: id.to_string(),
        score,
        metadata: None,
        matched_signals: vec!["sig".to_string()],
        provenance: None,
    }
}

#[test]
fn test_rrf_fuse_tie_break_strictly_ascending() {
    let list1 = vec![
        ScoredItem::new("doc_z", 10.0),
        ScoredItem::new("doc_a", 5.0),
    ];
    let list2 = vec![
        ScoredItem::new("doc_a", 10.0),
        ScoredItem::new("doc_z", 5.0),
    ];

    let fused = rrf_fuse(&[list1, list2], 60).expect("rrf_fuse should succeed");
    assert_eq!(fused.len(), 2);
    assert_eq!(fused[0].id, "doc_a");
    assert_eq!(fused[1].id, "doc_z");
}

#[test]
fn test_reciprocal_rank_fusion_tie_break_strictly_ascending() {
    let list1 = vec![
        make_res("item_z", 10.0),
        make_res("item_a", 5.0),
    ];
    let list2 = vec![
        make_res("item_a", 10.0),
        make_res("item_z", 5.0),
    ];

    let fused = reciprocal_rank_fusion(vec![list1, list2], 10);
    assert_eq!(fused.len(), 2);
    assert_eq!(fused[0].id, "item_a");
    assert_eq!(fused[1].id, "item_z");
    assert!((fused[0].score - fused[1].score).abs() < 1e-6);
}

#[test]
fn test_weighted_rrf_tie_break_strictly_ascending() {
    let list1 = vec![
        make_res("doc_300", 1.0),
        make_res("doc_100", 0.5),
        make_res("doc_200", 0.5),
    ];
    let list2 = vec![
        make_res("doc_300", 1.0),
        make_res("doc_200", 0.5),
        make_res("doc_100", 0.5),
    ];

    let sets = vec![
        ("s1".to_string(), list1, 1.0),
        ("s2".to_string(), list2, 1.0),
    ];

    let fused = weighted_reciprocal_rank_fusion(sets, 10);
    assert_eq!(fused.len(), 3);
    assert_eq!(fused[0].id, "doc_300");
    assert_eq!(fused[1].id, "doc_100");
    assert_eq!(fused[2].id, "doc_200");
}

#[test]
fn test_score_normalized_tie_break_strictly_ascending() {
    let list1 = vec![
        make_res("doc_y", 10.0),
        make_res("doc_x", 0.0),
    ];
    let list2 = vec![
        make_res("doc_x", 10.0),
        make_res("doc_y", 0.0),
    ];

    let sets = vec![
        ("s1".to_string(), list1, 1.0_f32),
        ("s2".to_string(), list2, 1.0_f32),
    ];

    let fused = score_normalized_fusion_with_options(
        sets,
        10,
        MetadataMergePriority::VectorFirst,
        false,
        None,
    );

    assert_eq!(fused.len(), 2);
    assert_eq!(fused[0].id, "doc_x");
    assert_eq!(fused[1].id, "doc_y");
    assert!((fused[0].score - fused[1].score).abs() < 1e-6);
}

proptest! {
    #[test]
    fn prop_fusion_tie_break_alphabetical(
        doc_count in 2usize..8,
        prefix in "[a-z]{3}",
    ) {
        let mut ids: Vec<String> = (0..doc_count)
            .map(|i| format!("{prefix}_{i:03}"))
            .collect();
        ids.sort();

        let list1: Vec<ScoredItem> = ids.iter().map(|id| ScoredItem::new(id, 1.0)).collect();
        let mut rev_ids = ids.clone();
        rev_ids.reverse();
        let list2: Vec<ScoredItem> = rev_ids.iter().map(|id| ScoredItem::new(id, 1.0)).collect();

        let fused = rrf_fuse(&[list1, list2], 60).unwrap();
        prop_assert_eq!(fused.len(), doc_count);
        for i in 0..(doc_count - 1) {
            if (fused[i].score - fused[i + 1].score).abs() < 1e-6 {
                prop_assert!(fused[i].id < fused[i + 1].id, "Gleichstand muss alphabetisch nach ID aufsteigend aufgelöst werden");
            }
        }
    }
}
