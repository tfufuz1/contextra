//! Test suite verifying zero-panic and deterministic behavior for NaN and Infinity float inputs.

#![allow(deprecated)]

use contextra_rank::fusion::{
    apply_resonance_bonus, g, modulate_and_renormalize_weights, reciprocal_rank_fusion, rrf_fuse,
    Hit, ProvenanceBuilder, ResonanceConfig, SearchResult,
};

// ANCHOR[contextra-rank:SAFE-TEST-001]
#[test]
fn test_g_function_nan_inf_safety() {
    assert_eq!(g(f32::NAN), 0.0);
    assert_eq!(g(f32::INFINITY), 0.0);
    assert_eq!(g(f32::NEG_INFINITY), 0.0);
    assert_eq!(g(0.0), 0.0);
}

// Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_provenance_builder_nan_inf_safety() {
    let builder = ProvenanceBuilder::new(f32::NAN)
        .vector(f32::NAN, 1, None)
        .bm25(f32::INFINITY, 1, None)
        .graph(f32::NEG_INFINITY, 1, None)
        .rerank_score(f32::NAN);

    let record = builder.build();
    assert_eq!(record.vector_distance, None);
    assert_eq!(record.bm25_score, None);
    assert_eq!(record.graph_score, None);
    assert_eq!(record.rerank_score, None);
}

// Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_resonance_config_nan_inf_safety() {
    let bad_cfg = ResonanceConfig {
        beta: f32::NAN,
        gamma: 0.3,
    };
    let docs = vec![SearchResult {
        id: "doc1".to_string(),
        score: 0.8,
        metadata: None,
        matched_signals: vec!["vector".to_string()],
        provenance: None,
    }];

    let res = apply_resonance_bonus(docs.clone(), 1, &bad_cfg);
    assert_eq!(res.first().map(|d| d.score), Some(0.8));
}

// Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_modulate_weights_nan_beta() {
    let sets = vec![("vec".to_string(), Vec::new(), 1.0)];
    let weights = modulate_and_renormalize_weights(&sets, None, f32::NAN);
    assert_eq!(weights, vec![1.0]);
}

// Testet das erwartete Verhalten und mathematische Invarianten.
#[test]
fn test_fusion_nan_doc_scores() {
    let docs = vec![
        SearchResult {
            id: "nan_doc".to_string(),
            score: f32::NAN,
            metadata: None,
            matched_signals: vec!["vec".to_string()],
            provenance: None,
        },
        SearchResult {
            id: "valid_doc".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["vec".to_string()],
            provenance: None,
        },
    ];

    let fused = reciprocal_rank_fusion(vec![docs], 10);
    assert_eq!(fused.len(), 2);
    assert!(fused.iter().all(|r| r.score.is_finite()));
}

// Testet NaN/Inf-Sicherheit für rrf_fuse
#[test]
fn test_rrf_fuse_nan_inf_safety() {
    let hits = vec![
        Hit::new("nan_doc", f32::NAN),
        Hit::new("inf_doc", f32::INFINITY),
        Hit::new("neg_inf_doc", f32::NEG_INFINITY),
        Hit::new("valid_doc", 0.5),
    ];

    let fused = rrf_fuse(&[hits], 60, 10);
    assert_eq!(fused.len(), 4);
    assert!(fused.iter().all(|h| h.score.is_finite()));
    assert!(fused.iter().all(|h| h.score > 0.0));
}
