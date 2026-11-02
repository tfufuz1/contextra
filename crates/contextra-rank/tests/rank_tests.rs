//! # Integrationstest: Basis-Ranking-Tests für RRF
//!
//! **Crate:** `contextra-rank`
//!
//! Prüft die Korrektheit, Stabilität und Invarianten der RRF-Komponenten.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

#[cfg(test)]
mod tests {
    use contextra_rank::fusion::{
        g, reciprocal_rank_fusion, rrf_fuse, BoundedTopK, Hit, ProvenanceBuilder, SearchResult,
    };

    // ANCHOR[contextra-rank:RNK-TEST-001]
    #[test]
    fn test_rrf_k_zero_boundary() {
        let prov = ProvenanceBuilder::new(0.0)
            .vector(0.9, 1, 1.0)
            .source_collection("col")
            .index_type("hnsw")
            .expected_total(1.0)
            .build();
        let contrib = prov
            .signal_contributions
            .get("vector")
            .expect("vector contribution present");
        assert_eq!(contrib.rrf_contribution, 1.0);
    }

    // Testet das erwartete Verhalten und mathematische Invarianten.
    #[test]
    fn test_rrf_dual_signal_higher_than_single_signal() {
        let set1 = vec![SearchResult {
            id: "doc_both".to_string(),
            score: 0.99,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        }];
        let set2 = vec![
            SearchResult {
                id: "doc_both".to_string(),
                score: 0.95,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
            SearchResult {
                id: "doc_single".to_string(),
                score: 0.99,
                metadata: None,
                matched_signals: vec![],
                provenance: None,
            },
        ];

        let fused = reciprocal_rank_fusion(vec![set1, set2], 10);
        assert_eq!(fused.len(), 2);
        if let (Some(f0), Some(f1)) = (fused.first(), fused.get(1)) {
            assert_eq!(f0.id, "doc_both");
            assert_eq!(f1.id, "doc_single");
            assert!(f0.score > f1.score);
        }
    }

    // Testet kanonische rrf_fuse Funktion mit Hits
    #[test]
    fn test_rrf_fuse_dual_signal_higher_than_single_signal() {
        let set1 = vec![Hit::new("doc_both", 0.99)];
        let set2 = vec![Hit::new("doc_both", 0.95), Hit::new("doc_single", 0.99)];

        let fused = rrf_fuse(&[set1, set2], 60, 10);
        assert_eq!(fused.len(), 2);
        if let (Some(f0), Some(f1)) = (fused.first(), fused.get(1)) {
            assert_eq!(f0.id, "doc_both");
            assert_eq!(f1.id, "doc_single");
            assert!(f0.score > f1.score);
        }
    }

    // Testet das erwartete Verhalten und mathematische Invarianten.
    #[test]
    fn test_bounded_top_k() {
        let mut top_k = BoundedTopK::new(2);
        top_k.push(10i32);
        top_k.push(20i32);
        top_k.push(5i32);
        let sorted = top_k.into_sorted_vec();
        assert_eq!(sorted, vec![5, 10]);
    }

    // Testet das erwartete Verhalten und mathematische Invarianten.
    #[test]
    fn test_g_function_standalone_properties() {
        // (c) g(0) is exactly 0
        assert_eq!(g(0.0), 0.0);
        assert_eq!(g(-0.0), 0.0);

        // g(x) bounded towards 1 for x -> infinity
        assert!((g(1000.0) - 1.0).abs() < 0.01);
        assert!((g(1_000_000.0) - 1.0).abs() < 0.0001);

        // g(x) bounded towards -1 for x -> -infinity
        assert!((g(-1000.0) - (-1.0)).abs() < 0.01);

        // Non-finite float safety
        assert_eq!(g(f32::NAN), 0.0);
        assert_eq!(g(f32::INFINITY), 0.0);
        assert_eq!(g(f32::NEG_INFINITY), 0.0);
    }
}
