// Testpflicht AK-12 (docs/specs/CONTEXTRA_SPEC_v2.md / Spec §15.2)

use contextra_core::DistanceMetric;
use contextra_vector::hnsw::Sq8Bias;
use contextra_vector::quantize::ScalarQuantizer;

#[test]
fn test_sq8_bias_calibration_audit_euclidean() {
    let dim = 128;
    let mut rng_state = 12345u64;

    // Helper PRNG
    let mut next_f32 = || {
        rng_state = rng_state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = rng_state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        (z ^ (z >> 31)) as f32 / u64::MAX as f32
    };

    // Build training dataset of 100 vectors
    let mut training_data = Vec::with_capacity(100);
    for _ in 0..100 {
        let vec: Vec<f32> = (0..dim).map(|_| next_f32() * 10.0 - 5.0).collect();
        training_data.push(vec);
    }

    let quantizer = ScalarQuantizer::train(&training_data, DistanceMetric::Euclidean)
        .expect("ScalarQuantizer training");

    let num_pairs = 1000;
    let mut raw_errors = Vec::with_capacity(num_pairs);
    let mut calibrated_errors = Vec::with_capacity(num_pairs);

    for _ in 0..num_pairs {
        let v1: Vec<f32> = (0..dim).map(|_| next_f32() * 10.0 - 5.0).collect();
        let v2: Vec<f32> = (0..dim).map(|_| next_f32() * 10.0 - 5.0).collect();

        let exact_dist_sq = v1
            .iter()
            .zip(v2.iter())
            .map(|(&a, &b)| (a - b) * (a - b))
            .sum::<f32>();

        let q1 = quantizer.encode(&v1);
        let q2 = quantizer.encode(&v2);

        let sq8_raw = quantizer.distance_u8(&q1, &q2);
        let sq8_calibrated = Sq8Bias::calibrate_distance(sq8_raw, DistanceMetric::Euclidean, &quantizer);

        raw_errors.push((sq8_raw - exact_dist_sq).abs());
        calibrated_errors.push((sq8_calibrated - exact_dist_sq).abs());
    }

    let mean_raw_error: f32 = raw_errors.iter().sum::<f32>() / (num_pairs as f32);
    let mean_calibrated_error: f32 = calibrated_errors.iter().sum::<f32>() / (num_pairs as f32);

    println!("Mean Raw SQ8 Distance Error:        {mean_raw_error:.4}");
    println!("Mean Calibrated SQ8 Distance Error: {mean_calibrated_error:.4}");

    assert!(
        mean_calibrated_error <= mean_raw_error * 1.05,
        "Calibrated distance error ({mean_calibrated_error:.4}) should be comparable or better than raw SQ8 error ({mean_raw_error:.4})"
    );
}
