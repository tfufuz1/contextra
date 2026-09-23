// Testpflicht AK-12 (docs/specs/MEMFUSE_SPEC_v2.md / Spec §15.2)

use memfuse_core::DistanceMetric;
use memfuse_vector::hnsw::Sq8Bias;
use memfuse_vector::quantize::ScalarQuantizer;

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
