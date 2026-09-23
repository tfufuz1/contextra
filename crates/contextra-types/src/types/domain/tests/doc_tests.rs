#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use crate::error::ContextraError;

use crate::types::domain::*;





    #[test]
    fn test_core_distance_dimension_mismatch() {
        let a = [1.0f32; 128];
        let b = [1.0f32; 256];
        let res = DistanceMetric::Cosine.compute(&a, &b);
        assert!(res.is_err());
    }

    #[test]
    fn test_embedding_norm_and_normalize() {
        let emb = Embedding::new(vec![3.0, 4.0]);
        assert_eq!(emb.dim(), 2);
        assert_eq!(emb.l2_norm(), 5.0);

        let normalized = emb.normalize();
        assert_eq!(normalized.l2_norm(), 1.0);
        assert_eq!(normalized.as_slice(), &[0.6, 0.8]);

        // Zero norm handling
        let zero_emb = Embedding::new(vec![0.0, 0.0]);
        let normalized_zero = zero_emb.normalize();
        assert_eq!(normalized_zero.l2_norm(), 0.0);
    }

    #[test]
    fn test_distance_metrics_f32() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        // Cosine: 1 - (0 / 1) = 1.0
        assert_eq!(DistanceMetric::Cosine.compute(&a, &b).unwrap(), 1.0); // unwrap
                                                                          // Euclidean: sqrt(1^2 + 1^2) = sqrt(2)
        assert_eq!(
            DistanceMetric::Euclidean.compute(&a, &b).unwrap(), // unwrap
            2.0f32.sqrt()
        );
        // DotProduct: -(0) = 0.0
        assert_eq!(DistanceMetric::DotProduct.compute(&a, &b).unwrap(), 0.0); // unwrap
    }

    #[test]
    fn test_distance_metrics_u8() {
        let a = [10, 20];
        let b = [20, 30];
        // Euclidean: sqrt((10-20)^2 + (20-30)^2) = sqrt(200) ≈ 14.142 -> rounded 14
        assert_eq!(DistanceMetric::Euclidean.compute_u8(&a, &b).unwrap(), 14); // unwrap
                                                                               // DotProduct: dot = 10*20 + 20*30 = 800 -> inverted: u32::MAX - 800
        assert_eq!(
            DistanceMetric::DotProduct.compute_u8(&a, &b).unwrap(), // unwrap
            u32::MAX - 800
        ); // unwrap
           // Cosine: orthogonal vectors → distance 1.0 → 1_000_000
        let orth_a: [u8; 2] = [255, 0];
        let orth_b: [u8; 2] = [0, 255];
        assert_eq!(
            DistanceMetric::Cosine.compute_u8(&orth_a, &orth_b).unwrap(), // unwrap
            1_000_000
        );
        // Cosine: identical vectors → distance 0.0 → 0
        assert_eq!(DistanceMetric::Cosine.compute_u8(&a, &a).unwrap(), 0); // unwrap
    }

    /// Regressionstest für AGT-CORE-001: Beweist, dass compute_u8() bei Vektoren der Länge
    /// 100_000 mit allen Elementen = 255 in keinem der drei Zweige panikt oder überläuft.
    ///
    /// Worst-case-Analyse:
    /// - Euclidean: diff=0 (identische Vektoren) → sum=0. Maximaler Fall: diff=255, sum = 255²×100_000
    ///   = 6_502_500_000, sqrt = 80_638.1.
    /// - DotProduct: 255×255×100_000 = 6_502_500_000 > u32::MAX → dot saturiert bei u32::MAX -> u32::MAX - u32::MAX = 0.
    /// - Cosine: f64-Akkumulation, kein Ganzzahl-Overflow möglich.
    #[test]
    fn test_distance_metrics_u8_overflow() {
        // Identische Vektoren (diff=0): Euclidean=0, DotProduct=0 (höchste Ähnlichkeit), Cosine=0
        let max_vec: Vec<u8> = vec![255u8; 100_000];
        let same_vec: Vec<u8> = vec![255u8; 100_000];

        // Euclidean: identische Vektoren → Distanz 0
        let eucl_same = DistanceMetric::Euclidean
            .compute_u8(&max_vec, &same_vec)
            .unwrap(); // unwrap
        assert_eq!(
            eucl_same, 0,
            "Euclidean distance of identical vectors must be 0"
        );

        // DotProduct: dot = saturiert u32::MAX → u32::MAX - u32::MAX = 0
        let dot_same = DistanceMetric::DotProduct
            .compute_u8(&max_vec, &same_vec)
            .unwrap(); // unwrap
        assert_eq!(
            dot_same, 0,
            "DotProduct inverted distance must be 0 for identical high-value vectors"
        );

        // Cosine: identische Vektoren → Distanz 0 (cos_dist = 1 - 1 = 0)
        let cos_same = DistanceMetric::Cosine
            .compute_u8(&max_vec, &same_vec)
            .unwrap(); // unwrap
        assert_eq!(
            cos_same, 0,
            "Cosine distance of identical vectors must be 0"
        );

        // Worst-case Euclidean: maximale Differenz (255 vs. 0) → sum = 255²×100_000 = 6_502_500_000, sqrt ≈ 80_638.1
        let zero_vec: Vec<u8> = vec![0u8; 100_000];
        let eucl_max = DistanceMetric::Euclidean
            .compute_u8(&max_vec, &zero_vec)
            .unwrap(); // unwrap
        assert_eq!(
            eucl_max, 80638,
            "Euclidean must produce rounded sqrt of sum of squared diffs"
        );

        // Cosine: senkrechte Vektoren (255..255 vs. 0..0) → Sonderfall: Nullvektor → Distanz 1.0
        let cos_zero = DistanceMetric::Cosine
            .compute_u8(&max_vec, &zero_vec)
            .unwrap(); // unwrap
        assert_eq!(
            cos_zero, 1_000_000,
            "Cosine distance against zero vector must be 1.0 (scaled: 1_000_000)"
        );
    }

    #[test]
    fn test_u8_and_f32_distance_metrics_ranking_and_value_parity() {
        let metrics = [
            DistanceMetric::Cosine,
            DistanceMetric::Euclidean,
            DistanceMetric::DotProduct,
        ];

        let query = [100u8, 200, 50, 150];
        let close_vec = [110u8, 190, 60, 140]; // very close vector
        let far_vec = [10u8, 20, 250, 5]; // distant vector

        let q_f32: Vec<f32> = query.iter().map(|&x| x as f32).collect();
        let c_f32: Vec<f32> = close_vec.iter().map(|&x| x as f32).collect();
        let f_f32: Vec<f32> = far_vec.iter().map(|&x| x as f32).collect();

        for metric in metrics {
            let u8_close = metric.compute_u8(&query, &close_vec).unwrap(); // unwrap
            let u8_far = metric.compute_u8(&query, &far_vec).unwrap(); // unwrap

            let f32_close = metric.compute(&q_f32, &c_f32).unwrap(); // unwrap
            let f32_far = metric.compute(&q_f32, &f_f32).unwrap(); // unwrap

            // Ranking order parity: smaller distance MUST mean closer for BOTH f32 and u8
            assert!(
                u8_close < u8_far,
                "u8 ranking mismatch for {metric:?}: close={u8_close}, far={u8_far}"
            );
            assert!(
                f32_close < f32_far,
                "f32 ranking mismatch for {metric:?}: close={f32_close}, far={f32_far}"
            );

            // Value scale sanity check
            match metric {
                DistanceMetric::Euclidean => {
                    // f32 euclidean sqrt diff vs u8 rounded sqrt diff
                    let diff = (u8_close as f32 - f32_close).abs();
                    assert!(
                        diff < 1.0,
                        "Euclidean f32 ({f32_close}) vs u8 ({u8_close}) deviation too large: diff={diff}"
                    );
                }
                DistanceMetric::Cosine => {
                    // u8 cosine is scaled by 1_000_000
                    let expected_scaled = (f32_close as f64 * 1_000_000.0).round() as u32;
                    let diff = (u8_close as i64 - expected_scaled as i64).abs();
                    assert!(
                        diff <= 1,
                        "Cosine fixed point scaling mismatch for {metric:?}: u8={u8_close}, scaled_f32={expected_scaled}"
                    );
                }
                DistanceMetric::DotProduct => {
                    // f32 dot product is -dot; u8 dot product is u32::MAX - dot
                    let dot_f32 = -f32_close; // raw dot
                    let dot_u8 = u32::MAX - u8_close; // raw dot
                    assert_eq!(
                        dot_f32 as u32, dot_u8,
                        "DotProduct raw dot values should match between f32 and u8"
                    );
                }
            }
        }
    }

    #[test]
    fn test_distance_metrics_empty_and_single_element() {
        let empty_a: [f32; 0] = [];
        let empty_b: [f32; 0] = [];

        // 0-dim vectors
        assert_eq!(
            DistanceMetric::Cosine.compute(&empty_a, &empty_b).unwrap(), // unwrap
            1.0
        ); // unwrap
        assert_eq!(
            DistanceMetric::Euclidean
                .compute(&empty_a, &empty_b)
                .unwrap(), // unwrap
            0.0
        ); // unwrap
        assert_eq!(
            DistanceMetric::DotProduct
                .compute(&empty_a, &empty_b)
                .unwrap(), // unwrap
            0.0
        ); // unwrap

        // 1-dim vectors
        let a1 = [3.0f32];
        let b1 = [4.0f32];
        // Cosine: angle is 0 between positive 1D values -> distance 0.0
        assert!((DistanceMetric::Cosine.compute(&a1, &b1).unwrap() - 0.0).abs() < 1e-6); // unwrap
                                                                                         // Euclidean: |3 - 4| = 1.0
        assert_eq!(DistanceMetric::Euclidean.compute(&a1, &b1).unwrap(), 1.0); // unwrap
                                                                               // DotProduct: -(3 * 4) = -12.0
        assert_eq!(DistanceMetric::DotProduct.compute(&a1, &b1).unwrap(), -12.0);
        // unwrap
        // unwrap
    }

    #[test]
    fn test_distance_metric_dimension_mismatch() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0];

        let res_cos = DistanceMetric::Cosine.compute(&a, &b);
        assert!(matches!(
            res_cos,
            Err(ContextraError::InvalidInput(msg)) if msg.contains("Vector dimensions must match")
        ));

        let res_euc = DistanceMetric::Euclidean.compute(&a, &b);
        assert!(matches!(
            res_euc,
            Err(ContextraError::InvalidInput(msg)) if msg.contains("Vector dimensions must match")
        ));

        let res_dot = DistanceMetric::DotProduct.compute(&a, &b);
        assert!(matches!(
            res_dot,
            Err(ContextraError::InvalidInput(msg)) if msg.contains("Vector dimensions must match")
        ));
    }

    #[test]
    fn test_embedding_normalize_edge_cases() {
        let zero_emb = Embedding::new(vec![0.0, 0.0, 0.0]);
        let norm_zero = zero_emb.normalize();
        assert_eq!(norm_zero.as_slice(), &[0.0, 0.0, 0.0]);

        let single_emb = Embedding::new(vec![5.0]);
        let norm_single = single_emb.normalize();
        assert_eq!(norm_single.as_slice(), &[1.0]);

        // Subnormal / near-zero norm protection
        let subnormal_emb = Embedding::new(vec![1e-38, 1e-38, 1e-38]);
        let norm_sub = subnormal_emb.normalize();
        for &val in norm_sub.as_slice() {
            assert!(
                val.is_finite(),
                "Normalized value must be finite, got {val}"
            );
            assert!(!val.is_nan(), "Normalized value must not be NaN");
            assert!(!val.is_infinite(), "Normalized value must not be Inf");
        }
        assert_eq!(norm_sub.as_slice(), &[1e-38, 1e-38, 1e-38]);
    }
