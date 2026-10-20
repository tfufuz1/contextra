// Testpflicht AK-12 (docs/specs/CONTEXTRA_SPEC_v2.md / Spec §15.2)

use contextra_core::DistanceMetric;
use contextra_vector::hnsw::Sq8Bias;
use contextra_vector::quantize::ScalarQuantizer;

#[test]
fn test_sq8_bias_calibration_and_persistence() {
    let v1 = vec![0.0f32, 0.0, 0.0, 0.0];
    let v2 = vec![1.0f32, 1.0, 1.0, 1.0];
    let v3 = vec![0.5f32, 0.5, 0.5, 0.5];
    let batch = vec![v1.as_slice(), v2.as_slice(), v3.as_slice()];

    let quantizer = ScalarQuantizer::train(&batch, 4);
    let bias = Sq8Bias::calibrate(&batch, &quantizer, DistanceMetric::Euclidean);

    assert!(bias.sample_count > 0);
    assert!(bias.mean_bias >= 0.0);
    assert!(bias.variance_bias >= 0.0);

    // Test serialization / header reconstruction
    let serialized = bincode::serialize(&bias).expect("Serialization failed");
    let deserialized: Sq8Bias = bincode::deserialize(&serialized).expect("Deserialization failed");

    assert_eq!(bias, deserialized);
}

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

    let training_batch: Vec<&[f32]> = training_data.iter().map(|v| v.as_slice()).collect();
    let quantizer = ScalarQuantizer::train(&training_batch, dim);

    let sq8_bias = Sq8Bias::calibrate(&training_batch, &quantizer, DistanceMetric::Euclidean);

    assert!(sq8_bias.sample_count > 0);
    assert!(sq8_bias.mean_bias >= 0.0);
    assert!(sq8_bias.variance_bias >= 0.0);

    let num_pairs = 1000;
    let mut raw_errors = Vec::with_capacity(num_pairs);

    for _ in 0..num_pairs {
        let v1: Vec<f32> = (0..dim).map(|_| next_f32() * 10.0 - 5.0).collect();
        let v2: Vec<f32> = (0..dim).map(|_| next_f32() * 10.0 - 5.0).collect();

        let exact_dist = v1
            .iter()
            .zip(v2.iter())
            .map(|(&a, &b)| (a - b) * (a - b))
            .sum::<f32>()
            .sqrt();

        let q1 = quantizer.quantize(&v1).expect("quantize v1");

        let sq8_raw = quantizer
            .asymmetric_dist(&v2, &q1, DistanceMetric::Euclidean)
            .expect("asymmetric_dist");

        raw_errors.push((sq8_raw - exact_dist).abs());
    }

    let mean_raw_error: f32 = raw_errors.iter().sum::<f32>() / (num_pairs as f32);

    println!("Mean SQ8 Distance Error: {mean_raw_error:.4}");
    println!("Calibrated Mean Bias:    {:.4}", sq8_bias.mean_bias);

    // Calibrated mean bias from dataset sample should closely reflect mean quantization error
    assert!(
        (sq8_bias.mean_bias - mean_raw_error).abs() < 0.05,
        "Calibrated mean bias ({:.4}) should accurately reflect average quantization error ({mean_raw_error:.4})",
        sq8_bias.mean_bias
    );

    // Test serialization / header reconstruction
    let serialized = bincode::serialize(&sq8_bias).expect("Serialization failed");
    let deserialized: Sq8Bias = bincode::deserialize(&serialized).expect("Deserialization failed");

    assert_eq!(sq8_bias, deserialized);
}
