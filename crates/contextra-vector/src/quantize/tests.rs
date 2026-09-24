use super::*;

#[test]
fn test_percentile_clipping_in_try_train() {
    let dim = 2;
    let mut batch = Vec::new();
    // 200 normal vectors in range [0.0, 1.0]
    for i in 0..200 {
        batch.push(vec![i as f32 / 200.0, 0.5]);
    }
    // Add extreme outliers
    batch[0] = vec![-100.0, 0.5];
    batch[199] = vec![1000.0, 0.5];

    let batch_refs: Vec<&[f32]> = batch.iter().map(|v| v.as_slice()).collect();
    let q = ScalarQuantizer::try_train(&batch_refs, dim).expect("train with clipping");

    // Without percentile clipping, mins[0] would be -100.0 and maxes[0] would be 1000.0.
    // With 0.5% / 99.5% percentile clipping on 200 vectors:
    // mins[0] should be clipped close to 0.0 and maxes[0] close to 1.0.
    assert!(
        q.mins()[0] > -10.0,
        "Lower outlier -100.0 must be clipped, got min: {}",
        q.mins()[0]
    );
    assert!(
        q.maxes()[0] < 10.0,
        "Upper outlier 1000.0 must be clipped, got max: {}",
        q.maxes()[0]
    );

    // Outlier vectors should be flagged by check_drift
    let outlier_vec = vec![500.0, 0.5];
    let drift = q.check_drift(&outlier_vec);
    assert_eq!(drift, 0.5, "Outlier dimension 0 should register drift");
}

#[test]
fn test_serde_roundtrip() {
    let v1 = vec![0.0, 1.0, 2.0];
    let v2 = vec![10.0, 11.0, 12.0];
    let original = ScalarQuantizer::train(&[v1.as_slice(), v2.as_slice()], 3);

    let serialized = bincode::serialize(&original).expect("serialize"); // expect
    let deserialized: ScalarQuantizer = bincode::deserialize(&serialized).expect("deserialize"); // expect

    assert_eq!(original.mins, deserialized.mins);
    assert_eq!(original.maxes, deserialized.maxes);
    assert_eq!(original.scales, deserialized.scales);
    assert_eq!(original.inv_scales, deserialized.inv_scales);
    assert_eq!(original.dimension, deserialized.dimension);
}

#[test]
fn test_corrupted_quantizer_returns_err_no_panic() {
    let q = ScalarQuantizer {
        mins: vec![0.0], // truncated mins
        maxes: vec![1.0, 1.0, 1.0, 1.0],
        scales: vec![255.0, 255.0, 255.0, 255.0],
        inv_scales: vec![1.0, 1.0, 1.0, 1.0],
        dimension: 4,
        total_queries: AtomicU64::new(0),
        out_of_range_queries: AtomicU64::new(0),
    };

    let vec_4d = vec![0.5, 0.5, 0.5, 0.5];
    let res_quant = q.quantize(&vec_4d);
    assert!(matches!(
        res_quant,
        Err(contextra_core::ContextraError::InvalidInput(_))
    ));

    let q_corrupt_dequant = ScalarQuantizer {
        mins: vec![0.0, 0.0, 0.0, 0.0],
        maxes: vec![1.0, 1.0, 1.0, 1.0],
        scales: vec![255.0, 255.0, 255.0, 255.0],
        inv_scales: vec![1.0], // truncated inv_scales
        dimension: 4,
        total_queries: AtomicU64::new(0),
        out_of_range_queries: AtomicU64::new(0),
    };
    let u8_4d = vec![127u8, 127, 127, 127];

    let res_dequant = q_corrupt_dequant.dequantize(&u8_4d);
    assert!(matches!(
        res_dequant,
        Err(contextra_core::ContextraError::InvalidInput(_))
    ));
}

#[test]
fn test_dist_dimension_mismatch_returns_error() {
    let v1 = vec![0.0, 1.0];
    let v2 = vec![2.0, 3.0];
    let q = ScalarQuantizer::train(&[v1.as_slice(), v2.as_slice()], 2);

    let query_3d = vec![1.0, 2.0, 3.0];
    let quant_2d = q.quantize(&v1).expect("quantize");

    // quantize dimension mismatch
    let res_quant = q.quantize(&query_3d);
    assert!(matches!(
        res_quant,
        Err(contextra_core::ContextraError::InvalidInput(_))
    ));

    // dequantize dimension mismatch
    let quant_3d = vec![1u8, 2, 3];
    let res_dequant = q.dequantize(&quant_3d);
    assert!(matches!(
        res_dequant,
        Err(contextra_core::ContextraError::InvalidInput(_))
    ));

    // asymmetric_dist length mismatch
    let res_asym = q.asymmetric_dist(&query_3d, &quant_2d, DistanceMetric::Cosine);
    assert!(matches!(
        res_asym,
        Err(contextra_core::ContextraError::InvalidInput(_))
    ));

    // symmetric_dist length mismatch
    let res_sym = q.symmetric_dist(&quant_2d, &quant_3d, DistanceMetric::Euclidean);
    assert!(matches!(
        res_sym,
        Err(contextra_core::ContextraError::InvalidInput(_))
    ));
}

#[test]
fn test_check_drift() {
    let v1 = vec![0.0, 0.0, 0.0, 0.0];
    let v2 = vec![10.0, 10.0, 10.0, 10.0];
    let q = ScalarQuantizer::train(&[v1.as_slice(), v2.as_slice()], 4);

    // Vector within range [0, 10] -> 0 drift
    let in_range = vec![0.5, 5.0, 2.0, 9.9];
    assert_eq!(q.check_drift(&in_range), 0.0);

    // 2 out of 4 dimensions out of range -> 0.5 drift
    let out_range = vec![-1.0, 5.0, 15.0, 3.0];
    assert_eq!(q.check_drift(&out_range), 0.5);
}

#[test]
fn test_quantize_dequantize_roundtrip() {
    let v1 = vec![0.1, -0.5, 0.8, 1.2];
    let v2 = vec![-1.0, 0.0, 0.5, 2.0];

    let q = ScalarQuantizer::train(&[v1.as_slice(), v2.as_slice()], 4);

    let quant = q.quantize(&v1).expect("quantize");
    let dequant = q.dequantize(&quant).expect("dequantize");

    let mut max_err = 0.0_f32;

    for i in 0..4 {
        let range = q.maxes[i] - q.mins[i];
        let err = (v1[i] - dequant[i]).abs();
        if err > max_err {
            max_err = err;
        }
        // Error should be strictly less than 1% of the per-dim range
        assert!(err < 0.01 * range);
    }
}

#[test]
fn test_quantized_search_no_panic() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut vectors = Vec::new();
    for _ in 0..100 {
        let v: Vec<f32> = (0..128).map(|_| rng.gen_range(-1.0..1.0)).collect();
        vectors.push(v);
    }

    let refs: Vec<&[f32]> = vectors.iter().map(|v| v.as_slice()).collect();
    let q = ScalarQuantizer::train(&refs, 128);

    let q_vecs: Vec<Vec<u8>> = vectors.iter().map(|v| q.quantize(v).unwrap()).collect();

    // Random queries
    for _ in 0..100 {
        let qv: Vec<f32> = (0..128).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let qq = q.quantize(&qv).unwrap();

        let mut top = 0;
        let mut top_dist = f32::MAX;
        for (i, v) in q_vecs.iter().enumerate() {
            let d = q
                .symmetric_dist(&qq, v, DistanceMetric::Cosine)
                .expect("dist"); // unwrap allowed in tests
            if d < top_dist {
                top_dist = d;
                top = i;
            }
        }
        assert!(top < 100);
    }
}

#[test]
fn test_train_empty_batch() {
    let q = ScalarQuantizer::train(&[], 128);
    assert_eq!(q.mins[0], 0.0);
    assert_eq!(q.maxes[0], 1.0);
    assert_eq!(q.dimension, 128);

    let v = vec![0.5; 128];
    let quantized = q.quantize(&v).expect("quantize");
    assert_eq!(quantized.len(), 128);
    for &val in &quantized {
        assert!(val > 120 && val < 135); // Close to 127
    }
}

/// Proves per-dimension SQ8 yields strictly better (or equal) reconstruction accuracy
/// than a global min/max quantizer when dimensions have heterogeneous value ranges.
///
/// # Anti-Mirroring: Reference values computed independently
/// The global-quantizer MSE is computed using a *separately implemented* quantization
/// formula (inline, without using `ScalarQuantizer`), making this a true regression check.
///
/// # Invariant (FIND-IND-002)
/// Per-dim MSE <= Global MSE, with strict inequality for heterogeneous distributions.
#[test]
fn test_per_dim_better_recall_than_global() {
    // Dimension 0: range [0.0, 1.0]
    // Dimension 1: range [0.0, 1000.0]
    // The wide range on dim 1 would dominate a global quantizer, degrading dim 0 recall.
    let vectors: Vec<Vec<f32>> = vec![
        vec![0.0, 0.0],
        vec![1.0, 1000.0],
        vec![0.5, 500.0],
        vec![0.1, 100.0],
        vec![0.9, 900.0],
    ];
    let refs: Vec<&[f32]> = vectors.iter().map(|v| v.as_slice()).collect();
    let per_dim_q = ScalarQuantizer::train(&refs, 2);

    // Probe vector: dim 0 has fine-grained variation (0.05), dim 1 is coarse.
    let probe = vec![0.05_f32, 50.0_f32];
    let per_dim_quant = per_dim_q.quantize(&probe).expect("quantize");
    let per_dim_dequant = per_dim_q.dequantize(&per_dim_quant).expect("dequantize");

    // Compute per-dim MSE
    let per_dim_mse: f32 = probe
        .iter()
        .zip(per_dim_dequant.iter())
        .map(|(orig, recon)| (orig - recon).powi(2))
        .sum::<f32>()
        / probe.len() as f32;

    // Independently compute global quantizer MSE:
    // Global min = 0.0, global max = 1000.0 (derived from the training data above, not from ScalarQuantizer)
    let global_min = 0.0_f32;
    let global_max = 1000.0_f32;
    let global_range = global_max - global_min;
    let global_scale = 255.0 / global_range;
    let global_inv_scale = global_range / 255.0;

    let global_quant: Vec<u8> = probe
        .iter()
        .map(|&v| {
            ((v.clamp(global_min, global_max) - global_min) * global_scale)
                .round()
                .clamp(0.0, 255.0) as u8
        })
        .collect();

    let global_dequant: Vec<f32> = global_quant
        .iter()
        .map(|&q| f32::from(q) * global_inv_scale + global_min)
        .collect();

    let global_mse: f32 = probe
        .iter()
        .zip(global_dequant.iter())
        .map(|(orig, recon)| (orig - recon).powi(2))
        .sum::<f32>()
        / probe.len() as f32;

    // Per-dim MSE must be strictly better than global MSE for heterogeneous data.
    // For dim 0 (range [0,1]): per-dim uses full 256 steps over 1.0 range (step ≈ 0.004)
    //                           global uses 256 steps over 1000.0 range (step ≈ 3.9) — terrible
    // This makes global_mse >> per_dim_mse by approximately (3.9/0.004)^2 ≈ 1M×
    assert!(
        per_dim_mse < global_mse,
        "Per-dim MSE ({}) should be < global MSE ({}) for heterogeneous dimensions",
        per_dim_mse,
        global_mse
    );

    // Concrete bound: per-dim reconstruction error on dim 0 must be < 1% of its range (0..1)
    let dim0_err = (probe[0] - per_dim_dequant[0]).abs();
    assert!(
        dim0_err < 0.01,
        "Per-dim quantizer should reconstruct dim0 (range 0..1) within 1%, got error: {}",
        dim0_err
    );
}

proptest::proptest! {
    /// Proptest: for any batch of vectors with dimension 2 where the ranges differ by
    /// at least 10×, per-dim SQ8 produces lower reconstruction error than global SQ8.
    ///
    /// # Anti-Mirroring: global quantizer reference is independently computed inline.
    /// # FIND-IND-002
    #[test]
    fn prop_per_dim_mse_le_global_mse(
        // dim0 values in [0, 1], dim1 values in [100, 1000] — heterogeneous ranges guaranteed
        dim0_vals in proptest::collection::vec(0.0f32..1.0f32, 2..10),
        dim1_vals in proptest::collection::vec(100.0f32..1000.0f32, 2..10),
    ) {
        let n = dim0_vals.len().min(dim1_vals.len());
        let vectors: Vec<Vec<f32>> = (0..n)
            .map(|i| vec![dim0_vals[i], dim1_vals[i]])
            .collect();
        let refs: Vec<&[f32]> = vectors.iter().map(|v| v.as_slice()).collect();
        let per_dim_q = ScalarQuantizer::train(&refs, 2);

        // Global quantizer: independently compute using min/max of ALL values across ALL dims
        let all_vals: Vec<f32> = vectors.iter().flat_map(|v| v.iter().copied()).collect();
        let g_min = all_vals.iter().cloned().fold(f32::MAX, f32::min);
        let g_max = all_vals.iter().cloned().fold(f32::MIN, f32::max);
        let g_range = if (g_max - g_min).abs() < f32::EPSILON { 1e-6 } else { g_max - g_min };
        let g_scale = 255.0 / g_range;
        let g_inv = g_range / 255.0;

        // Invariant 1: Step sizes (inv_scales) for per-dimension must be <= global step size (g_inv)
        for i in 0..2 {
            proptest::prop_assert!(
                per_dim_q.inv_scales[i] <= g_inv + 1e-5,
                "Dimension {} inv_scale {} should be <= global step size {}",
                i, per_dim_q.inv_scales[i], g_inv
            );
        }

        // Invariant 2: Average MSE over 500 uniformly distributed random probe points
        // must be <= global average MSE (within a small statistical tolerance).
        // We use a simple deterministic LCG to avoid grid-alignment artifacts.
        let mut total_per_dim_mse = 0.0f32;
        let mut total_global_mse = 0.0f32;

        let mut seed = 42u32;
        let mut next_random = || {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            (seed & 0x7FFFFFFF) as f32 / 0x7FFFFFFF as f32
        };

        let num_probes = 500;
        let d0_min = vectors.iter().map(|v| v[0]).fold(f32::MAX, f32::min);
        let d0_max = vectors.iter().map(|v| v[0]).fold(f32::MIN, f32::max);
        let d0_range = if (d0_max - d0_min).abs() < f32::EPSILON { 0.0 } else { d0_max - d0_min };

        let d1_min = vectors.iter().map(|v| v[1]).fold(f32::MAX, f32::min);
        let d1_max = vectors.iter().map(|v| v[1]).fold(f32::MIN, f32::max);
        let d1_range = if (d1_max - d1_min).abs() < f32::EPSILON { 0.0 } else { d1_max - d1_min };

        for _ in 0..num_probes {
            let p0 = d0_min + next_random() * d0_range;
            let p1 = d1_min + next_random() * d1_range;
            let probe = vec![p0, p1];

            // Per-dim error
            let per_dim_quant = per_dim_q.quantize(&probe).expect("quantize");
            let per_dim_dequant = per_dim_q.dequantize(&per_dim_quant).expect("dequantize");
            let per_dim_mse: f32 = probe.iter().zip(per_dim_dequant.iter())
                .map(|(o, r)| (o - r).powi(2)).sum::<f32>() / 2.0;

            // Global error
            let global_mse: f32 = probe.iter().map(|&v| {
                let q = ((v.clamp(g_min, g_max) - g_min) * g_scale).round().clamp(0.0, 255.0) as u8;
                let r = f32::from(q) * g_inv + g_min;
                (v - r).powi(2)
            }).sum::<f32>() / 2.0;

            total_per_dim_mse += per_dim_mse;
            total_global_mse += global_mse;
        }

        let avg_per_dim_mse = total_per_dim_mse / num_probes as f32;
        let avg_global_mse = total_global_mse / num_probes as f32;

        // With 500 uniform points, the statistical variance is extremely small,
        // but we allow a tiny 1e-3 tolerance for edge cases.
        proptest::prop_assert!(
            avg_per_dim_mse <= avg_global_mse + 1e-3,
            "Average per_dim_mse={} should be <= global_mse={}",
            avg_per_dim_mse, avg_global_mse
        );
    }
}

#[test]
fn test_quantizer_try_train_guards() {
    // Zero dimension guard
    let res_zero = ScalarQuantizer::try_train(&[], 0);
    assert!(res_zero.is_err());
    assert!(res_zero
        .unwrap_err()
        .to_string()
        .contains("dimension must be greater than 0"));

    // Mismatched vector length guard
    let v1 = vec![1.0, 2.0, 3.0];
    let v2 = vec![1.0, 2.0];
    let batch: Vec<&[f32]> = vec![&v1, &v2];
    let res_mismatch = ScalarQuantizer::try_train(&batch, 3);
    assert!(res_mismatch.is_err());
    assert!(res_mismatch.unwrap_err().to_string().contains("expected 3"));

    // Invalid percentile bounds guard
    let res_inv1 = ScalarQuantizer::try_train_with_percentiles(&batch[..1], 3, -0.1, 0.9);
    assert!(res_inv1.is_err());

    let res_inv2 = ScalarQuantizer::try_train_with_percentiles(&batch[..1], 3, 0.9, 0.8);
    assert!(res_inv2.is_err());

    let res_inv3 = ScalarQuantizer::try_train_with_percentiles(&batch[..1], 3, 0.1, 1.1);
    assert!(res_inv3.is_err());

    let res_inv4 = ScalarQuantizer::try_train_with_percentiles(&batch[..1], 3, f32::NAN, 0.95);
    assert!(res_inv4.is_err());
}

#[test]
fn test_percentile_clipping_outlier_recall_improvement() {
    // Create 400 synthetic vectors in [0.0, 1.0] for dimension 0
    let mut vectors: Vec<Vec<f32>> = (0..398).map(|i| vec![(i as f32 / 397.0), 0.5]).collect();
    // Add 2 extreme artificial outliers
    vectors.push(vec![-500.0, 0.5]);
    vectors.push(vec![500.0, 0.5]);

    let refs: Vec<&[f32]> = vectors.iter().map(|v| v.as_slice()).collect();

    // 1. Raw min/max quantizer (0.0% / 100.0%)
    let raw_q = ScalarQuantizer::train_with_percentiles(&refs, 2, 0.0, 1.0);
    // 2. Percentile clipped quantizer (0.5% / 99.5%)
    let clipped_q = ScalarQuantizer::train_with_percentiles(&refs, 2, 0.005, 0.995);

    // Check range for dim 0
    assert_eq!(raw_q.mins()[0], -500.0);
    assert_eq!(raw_q.maxes()[0], 500.0);

    assert!(clipped_q.mins()[0] >= 0.0);
    assert!(clipped_q.maxes()[0] <= 1.0);

    // Evaluate reconstruction MSE on non-outlier vectors (first 398)
    let mut raw_mse_sum = 0.0_f32;
    let mut clipped_mse_sum = 0.0_f32;

    for v in &vectors[..398] {
        let q_raw = raw_q.quantize(v).expect("quantize raw");
        let deq_raw = raw_q.dequantize(&q_raw).expect("dequantize raw");
        raw_mse_sum += (v[0] - deq_raw[0]).powi(2);

        let q_clipped = clipped_q.quantize(v).expect("quantize clipped");
        let deq_clipped = clipped_q
            .dequantize(&q_clipped)
            .expect("dequantize clipped");
        clipped_mse_sum += (v[0] - deq_clipped[0]).powi(2);
    }

    let raw_mse = raw_mse_sum / 398.0;
    let clipped_mse = clipped_mse_sum / 398.0;

    // Percentile clipped MSE must be significantly lower than raw min/max MSE
    assert!(
        clipped_mse < raw_mse / 100.0,
        "Clipped MSE ({clipped_mse}) should be at least 100x lower than raw MSE ({raw_mse})"
    );
}

#[test]
fn test_quantize_clamping_and_drift_counting() -> contextra_core::Result<()> {
    let v1 = vec![0.0, 0.0];
    let v2 = vec![1.0, 1.0];
    let q = ScalarQuantizer::train(&[&v1, &v2], 2);

    assert_eq!(q.drift_ratio(), 0.0);

    // Quantize in-range vector -> 0 drift
    let in_range = vec![0.5, 0.5];
    let res_in = q.quantize(&in_range)?;
    assert_eq!(res_in.len(), 2);
    assert_eq!(q.drift_ratio(), 0.0);

    // Quantize out-of-range vector -> values clamped, drift ratio increases
    let out_range = vec![-5.0, 10.0];
    let res_out = q.quantize(&out_range)?;
    assert_eq!(res_out, vec![0, 255]);
    assert!(q.drift_ratio() > 0.0);

    // Codebook mins/maxes must remain unchanged (Codebook Invariance)
    assert_eq!(q.mins(), &[0.0, 0.0]);
    assert_eq!(q.maxes(), &[1.0, 1.0]);
    Ok(())
}

#[test]
fn test_asymmetric_symmetric_distance_metrics() {
    // Build quantizer for 3D vectors
    // x in [0, 10], y in [0, 20], z in [0, 100]
    let v1 = vec![0.0, 0.0, 0.0];
    let v2 = vec![10.0, 20.0, 100.0];
    let q = ScalarQuantizer::train(&[&v1, &v2], 3);

    let query = vec![5.0, 10.0, 50.0];
    let target_vec = vec![0.0, 0.0, 0.0];
    let target_quant = q.quantize(&target_vec).expect("quantize");

    // Test asymmetric distances
    let cos_dist = q
        .asymmetric_dist(&query, &target_quant, DistanceMetric::Cosine)
        .unwrap(); // unwrap
    assert!(cos_dist >= 0.0);

    let euc_dist = q
        .asymmetric_dist(&query, &target_quant, DistanceMetric::Euclidean)
        .unwrap(); // unwrap
                   // Distance query [5, 10, 50] to target [0, 0, 0] is sqrt(25 + 100 + 2500) = sqrt(2625) ~ 51.2347
    assert!((euc_dist - 51.2347).abs() < 1.0);

    let dot_dist = q
        .asymmetric_dist(&query, &target_quant, DistanceMetric::DotProduct)
        .unwrap(); // unwrap
                   // Dot product distance returns negative dot product: -(0 + 0 + 0) = 0.0
    assert_eq!(dot_dist, 0.0);

    // Test symmetric distances
    let query_quant = q.quantize(&query).expect("quantize");
    let sym_cos = q
        .symmetric_dist(&query_quant, &target_quant, DistanceMetric::Cosine)
        .unwrap(); // unwrap
    assert!(sym_cos >= 0.0);

    let sym_euc = q
        .symmetric_dist(&query_quant, &target_quant, DistanceMetric::Euclidean)
        .unwrap(); // unwrap
    assert!((sym_euc - 51.2347).abs() < 2.0);

    let sym_dot = q
        .symmetric_dist(&query_quant, &target_quant, DistanceMetric::DotProduct)
        .unwrap(); // unwrap
    assert!(sym_dot <= 0.0);
}
