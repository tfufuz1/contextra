// FILE-CONTEXT
// ZWECK: Recall- und Kalibrierungstests für RaBitQ (randomisierte binäre Quantisierung).

use contextra_vector::quantize_rabitq::RaBitQQuantizer;

#[test]
fn test_rabitq_seeded_calibration_recall() {
    let dim = 32;
    let seed = 987654321u64;

    let v1: Vec<f32> = (0..dim).map(|i| (i as f32 * 0.1).sin()).collect();
    let v2: Vec<f32> = (0..dim).map(|i| (i as f32 * 0.1).cos()).collect();

    let quantizer = RaBitQQuantizer::try_train_with_seed(&[v1.clone(), v2.clone()], dim, seed).unwrap();

    let code1 = quantizer.quantize(&v1).unwrap();
    let code2 = quantizer.quantize(&v2).unwrap();

    let estimated_dist = quantizer.asymmetric_distance(&v1, &code2).unwrap();
    let exact_dist: f32 = v1.iter().zip(v2.iter()).map(|(&a, &b)| (a - b) * (a - b)).sum::<f32>().sqrt();

    assert!((estimated_dist - exact_dist).abs() < 1.0, "Estimated distance {estimated_dist} should be near exact {exact_dist}");
}
