// FILE-CONTEXT
// ZWECK: Golden Values Testtabelle und Differentialtests für DistanceKernel / ScalarKernel (WP-VEC-3, INV-SIMD-DETERMINISM-1).

use contextra_types::DistanceMetric;
use contextra_vector::distance::{
    compute_distance, compute_distance_f32_bytes_trusted, compute_distance_trusted,
    validate_vector, DistanceKernel, ScalarKernel,
};

struct GoldenMetricEntry {
    dim: usize,
    cosine: f32,
    euclidean: f32,
    dot_product: f32,
}

const GOLDEN_TABLE: &[GoldenMetricEntry] = &[
    GoldenMetricEntry {
        dim: 1,
        cosine: 0.0000000,
        euclidean: 0.2393051,
        dot_product: -0.0691089,
    },
    GoldenMetricEntry {
        dim: 2,
        cosine: 0.2141170,
        euclidean: 0.2792108,
        dot_product: -0.1323516,
    },
    GoldenMetricEntry {
        dim: 7,
        cosine: 1.8002642,
        euclidean: 2.8332749,
        dot_product: 1.7415028,
    },
    GoldenMetricEntry {
        dim: 8,
        cosine: 1.8537676,
        euclidean: 3.4146806,
        dot_product: 2.6490323,
    },
    GoldenMetricEntry {
        dim: 16,
        cosine: 1.9321470,
        euclidean: 5.5833615,
        dot_product: 7.4855571,
    },
    GoldenMetricEntry {
        dim: 32,
        cosine: 1.7978861,
        euclidean: 7.5494783,
        dot_product: 12.6000476,
    },
    GoldenMetricEntry {
        dim: 64,
        cosine: 1.1147990,
        euclidean: 8.4003428,
        dot_product: 3.6329142,
    },
    GoldenMetricEntry {
        dim: 128,
        cosine: 1.1508599,
        euclidean: 12.0736925,
        dot_product: 9.5524986,
    },
];

#[test]
fn test_scalar_kernel_golden_table_values() {
    let kernel = ScalarKernel;

    for entry in GOLDEN_TABLE {
        let dim = entry.dim;
        let a: Vec<f32> = (0..dim).map(|i| ((i + 1) as f32 * 0.17).sin()).collect();
        let b: Vec<f32> = (0..dim).map(|i| ((i + 5) as f32 * 0.23).cos()).collect();

        // Cosine
        let cos = kernel.cosine_distance(&a, &b).unwrap();
        assert!(
            (cos - entry.cosine).abs() < 1e-5,
            "Golden cosine mismatch at dim {dim}: got {cos}, expected {}",
            entry.cosine
        );

        // Euclidean
        let euc = kernel.euclidean_distance(&a, &b).unwrap();
        assert!(
            (euc - entry.euclidean).abs() < 1e-5,
            "Golden euclidean mismatch at dim {dim}: got {euc}, expected {}",
            entry.euclidean
        );

        // Dot product
        let dot = kernel.dot_product_distance(&a, &b).unwrap();
        assert!(
            (dot - entry.dot_product).abs() < 1e-5,
            "Golden dot_product mismatch at dim {dim}: got {dot}, expected {}",
            entry.dot_product
        );
    }
}

#[test]
fn test_bytes_kernel_equivalence_golden_table() {
    let kernel = ScalarKernel;

    for entry in GOLDEN_TABLE {
        let dim = entry.dim;
        let a: Vec<f32> = (0..dim).map(|i| ((i + 1) as f32 * 0.17).sin()).collect();
        let b: Vec<f32> = (0..dim).map(|i| ((i + 5) as f32 * 0.23).cos()).collect();

        let mut b_bytes = Vec::with_capacity(dim * 4);
        for &val in &b {
            b_bytes.extend_from_slice(&val.to_le_bytes());
        }

        let cos_slice = kernel.cosine_distance(&a, &b).unwrap();
        let cos_bytes = kernel
            .distance_f32_bytes(DistanceMetric::Cosine, &a, &b_bytes)
            .unwrap();
        assert!(
            (cos_slice - cos_bytes).abs() < 1e-6,
            "Byte path cosine mismatch at dim {dim}"
        );

        let euc_slice = kernel.euclidean_distance(&a, &b).unwrap();
        let euc_bytes = kernel
            .distance_f32_bytes(DistanceMetric::Euclidean, &a, &b_bytes)
            .unwrap();
        assert!(
            (euc_slice - euc_bytes).abs() < 1e-6,
            "Byte path euclidean mismatch at dim {dim}"
        );

        let dot_slice = kernel.dot_product_distance(&a, &b).unwrap();
        let dot_bytes = kernel
            .distance_f32_bytes(DistanceMetric::DotProduct, &a, &b_bytes)
            .unwrap();
        assert!(
            (dot_slice - dot_bytes).abs() < 1e-6,
            "Byte path dot product mismatch at dim {dim}"
        );
    }
}

#[test]
fn test_fault_and_edge_cases() {
    // 1. Dimension mismatch
    let v2 = [1.0, 2.0];
    let v3 = [1.0, 2.0, 3.0];
    assert!(compute_distance(&v2, &v3, DistanceMetric::Euclidean).is_err());
    assert!(compute_distance_trusted(&v2, &v3, DistanceMetric::Cosine).is_err());

    // 2. Unaligned bytes (INV-SIMD-ALIGN-1)
    let bad_bytes = [1u8, 2, 3];
    assert!(
        compute_distance_f32_bytes_trusted(&v2, &bad_bytes, DistanceMetric::Euclidean).is_err()
    );

    // 3. Byte length dimension mismatch
    let v2_bytes = [0u8; 8];
    assert!(compute_distance_f32_bytes_trusted(&v3, &v2_bytes, DistanceMetric::Euclidean).is_err());

    // 4. Non-finite values (INV-SIMD-FINITE-1)
    let nan_vec = [1.0, f32::NAN];
    let inf_vec = [1.0, f32::INFINITY];
    let neg_inf_vec = [1.0, f32::NEG_INFINITY];

    assert!(validate_vector(&nan_vec).is_err());
    assert!(validate_vector(&inf_vec).is_err());
    assert!(validate_vector(&neg_inf_vec).is_err());

    assert!(compute_distance(&nan_vec, &v2, DistanceMetric::Cosine).is_err());
    assert!(compute_distance(&v2, &inf_vec, DistanceMetric::Euclidean).is_err());

    // 5. Zero vector norm safety in cosine
    let zero = [0.0, 0.0];
    let cos_zero = compute_distance(&zero, &zero, DistanceMetric::Cosine).unwrap();
    assert_eq!(cos_zero, 1.0);
}
