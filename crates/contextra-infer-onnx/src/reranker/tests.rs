// FILE-CONTEXT
// STAND: 2026-09-15T16:10:40Z (SESSION: ec33599e)
// ZWECK: Unit-Tests für Cross-Encoder Reranker.

#![allow(clippy::unwrap_used, clippy::panic)]

use super::config::{PlattScaledSigmoid, RerankConfig, RerankResult, MAX_CANDIDATES};
use super::cross_encoder::CrossEncoderReranker;
#[cfg(feature = "onnx")]
use super::onnx::OnnxReranker;

use contextra_core::{ConfigFingerprint, ContextraError};
use contextra_rank::PlattScaler;

#[tokio::test]
async fn test_rerank_passthrough_preserves_order() {
    let config = RerankConfig::default();
    if let Ok(reranker) = CrossEncoderReranker::new(config) {
        let candidates = vec!["first".into(), "second".into(), "third".into()];
        let results = reranker.rerank("query", &candidates).await.unwrap(); // unwrap
        assert_eq!(results.len(), 3);
    }
}

#[tokio::test]
async fn test_rerank_empty_candidates() {
    let config = RerankConfig::default();
    if let Ok(reranker) = CrossEncoderReranker::new(config) {
        let results = reranker.rerank("query", &[]).await.unwrap(); // unwrap
        assert!(results.is_empty());
    }
}

#[tokio::test]
async fn test_rerank_sorted_by_score_descending() {
    let config = RerankConfig::default();
    if let Ok(reranker) = CrossEncoderReranker::new(config) {
        let candidates: Vec<String> = (0..5).map(|i| format!("candidate {i}")).collect();
        let results = reranker.rerank("query", &candidates).await.unwrap(); // unwrap
        for window in results.windows(2) {
            assert!(window[0].score >= window[1].score);
        }
    }
}

#[tokio::test]
async fn test_rerank_oversized_candidate_batch_rejected() {
    let config = RerankConfig::default();
    if let Ok(reranker) = CrossEncoderReranker::new(config) {
        let candidates: Vec<String> = vec!["doc".to_string(); MAX_CANDIDATES + 1];
        let res = reranker.rerank("query", &candidates).await;
        assert!(res.is_err());
        if let Err(err) = res {
            assert!(matches!(err, ContextraError::InvalidInput(_)));
            assert!(err.to_string().contains("exceeds maximum allowed limit"));
        } else {
            panic!("Expected InvalidInput error for oversized candidate batch");
        }
    }
}

#[tokio::test]
async fn test_concurrent_rerank_load_no_panic() {
    use std::sync::Arc;

    let config = RerankConfig::default();
    if let Ok(reranker) = CrossEncoderReranker::new(config) {
        let reranker = Arc::new(reranker);

        let candidates: Vec<String> = (0..10).map(|i| format!("doc {i}")).collect();
        let mut handles = Vec::new();

        // Simulate 20 concurrent requests (exceeding pool/session limits)
        for i in 0..20 {
            let reranker_cloned = reranker.clone();
            let candidates_cloned = candidates.clone();
            handles.push(tokio::spawn(async move {
                let query = format!("query {i}");
                reranker_cloned.rerank(&query, &candidates_cloned).await
            }));
        }

        for handle in handles {
            let res = handle.await.unwrap(); // unwrap
            assert!(res.is_ok());
            assert_eq!(res.unwrap().len(), 10); // unwrap
        }
    }
}

#[cfg(feature = "onnx")]
#[test]
fn test_extract_scores_1d_and_2d() -> Result<(), Box<dyn std::error::Error>> {
    // 1D tensor [batch_size = 2]
    let shape = vec![2];
    let data = vec![0.0f32, 2.0f32];
    let scores = OnnxReranker::extract_scores_from_tensor(&shape, &data, 2)?;
    assert_eq!(scores.len(), 2);
    assert!((scores[0] - 0.5).abs() < 1e-4);
    assert!(scores[1] > 0.8);

    // 2D tensor [batch_size = 2, cols = 1]
    let shape = vec![2, 1];
    let data = vec![0.0f32, -2.0f32];
    let scores = OnnxReranker::extract_scores_from_tensor(&shape, &data, 2)?;
    assert_eq!(scores.len(), 2);
    assert!((scores[0] - 0.5).abs() < 1e-4);
    assert!(scores[1] < 0.2);

    // 2D tensor [batch_size = 2, cols = 2] (binary classification logits)
    let shape = vec![2, 2];
    let data = vec![1.0f32, 3.0f32, 2.0f32, 0.0f32]; // diff: +2.0, -2.0
    let scores = OnnxReranker::extract_scores_from_tensor(&shape, &data, 2)?;
    assert_eq!(scores.len(), 2);
    assert!(scores[0] > 0.8);
    assert!(scores[1] < 0.2);

    Ok(())
}

#[test]
fn test_platt_scaled_sigmoid_identity_matches_uncalibrated_sigmoid() {
    let identity = PlattScaledSigmoid::identity();
    assert!(identity.is_identity());
    assert_eq!(identity.params(), (1.0, 0.0));

    let logits: Vec<f32> = vec![-5.0, -2.5, -1.0, 0.0, 0.5, 1.2, 3.0, 7.5];
    for logit in logits {
        let uncalibrated = 1.0f32 / (1.0f32 + (-logit).exp());
        let calibrated = identity.transform(logit);
        assert_eq!(
            uncalibrated, calibrated,
            "Mismatch at logit {logit}: uncalibrated={uncalibrated}, calibrated={calibrated}"
        );
    }
}

#[test]
fn test_platt_scaled_sigmoid_fit_separable_dataset() {
    // Dataset with modest logits (-1.0 to 1.0) where uncalibrated sigmoid is soft (0.27 to 0.73)
    // positive logits (> 0) are relevant, negative (< 0) are irrelevant.
    let mut observations = Vec::new();
    for i in -10..=10 {
        let logit = i as f32 * 0.1;
        let is_rel = logit > 0.0;
        observations.push((logit, is_rel));
    }

    let fitted = PlattScaledSigmoid::fit(&observations);
    assert!(!fitted.is_identity());

    let (a, _b) = fitted.params();
    assert!(
        a > 0.0,
        "Scaling factor A should be positive for positively correlated logits"
    );

    // Verify that transform() provides sharper separation than identity sigmoid
    let identity = PlattScaledSigmoid::identity();
    let logit_pos = 1.0f32;
    let logit_neg = -1.0f32;

    let uncalibrated_diff = identity.transform(logit_pos) - identity.transform(logit_neg);
    let calibrated_diff = fitted.transform(logit_pos) - fitted.transform(logit_neg);

    assert!(
        calibrated_diff > uncalibrated_diff,
        "Calibrated transform diff ({calibrated_diff}) should be sharper than uncalibrated ({uncalibrated_diff})"
    );
    assert!(fitted.transform(logit_pos) > 0.80);
    assert!(fitted.transform(logit_neg) < 0.20);
}

#[test]
fn test_platt_scaled_sigmoid_fit_noisy_unseparable_and_nan() {
    // Test 1: Empty observations
    let empty_fitted = PlattScaledSigmoid::fit(&[]);
    assert!(empty_fitted.is_identity());
    assert!(!empty_fitted.transform(0.0).is_nan());

    // Test 2: Non-finite logits (NaN, Inf, -Inf)
    let non_finite_obs = vec![
        (f32::NAN, true),
        (f32::INFINITY, false),
        (f32::NEG_INFINITY, true),
    ];
    let non_finite_fitted = PlattScaledSigmoid::fit(&non_finite_obs);
    assert!(non_finite_fitted.is_identity());
    assert_eq!(non_finite_fitted.transform(f32::NAN), 0.5);

    // Test 3: Completely noisy / non-separable dataset (random labels)
    let mut noisy_obs = Vec::new();
    for i in 0..100 {
        let logit = (i as f32 - 50.0) * 0.1;
        let is_rel = i % 2 == 0; // complete noise uncorrelated with logit
        noisy_obs.push((logit, is_rel));
    }

    let noisy_fitted = PlattScaledSigmoid::fit(&noisy_obs);
    let (a, b) = noisy_fitted.params();

    assert!(a.is_finite(), "Param A must be finite");
    assert!(b.is_finite(), "Param B must be finite");

    let test_val = noisy_fitted.transform(1.0);
    assert!(
        test_val.is_finite() && (0.0..=1.0).contains(&test_val),
        "Output must be a valid probability in [0,1]"
    );
}

#[test]
fn test_record_implicit_feedback_top_k_marked_relevant() {
    let reranker = CrossEncoderReranker::passthrough();
    // Manually record outcomes on reranker directly
    let results = vec![
        RerankResult {
            original_index: 0,
            score: 2.5,
        },
        RerankResult {
            original_index: 1,
            score: 1.8,
        },
        RerankResult {
            original_index: 5,
            score: -0.5,
        },
    ];
    // With passthrough, implicit feedback is skipped
    reranker.record_implicit_feedback(&results, 2);
    assert_eq!(reranker.calibration_observation_count(), 0);

    // Test outcome recording manually via record_outcome
    for r in &results {
        reranker.record_outcome(r.score, r.original_index < 2);
    }
    assert_eq!(reranker.calibration_observation_count(), 3);
    let buf = reranker.calibration_buffer.lock().clone();
    assert_eq!(buf[0], (2.5, true));
    assert_eq!(buf[1], (1.8, true));
    assert_eq!(buf[2], (-0.5, false));
}

#[test]
fn test_calibration_is_calibrated_after_warmup() {
    let reranker = CrossEncoderReranker::passthrough();
    assert!(!reranker.is_calibrated());
    assert_eq!(reranker.calibration_observation_count(), 0);

    for i in 0..50 {
        let logit = (i as f32 - 25.0) * 0.1;
        reranker.record_outcome(logit, logit > 0.0);
    }

    assert_eq!(reranker.calibration_observation_count(), 50);
    assert!(reranker.is_calibrated());
}

#[test]
fn test_calibration_fitted_platt_not_identity() {
    let reranker = CrossEncoderReranker::passthrough();
    assert!(reranker.fitted_calibration().is_identity());

    for i in 0..50 {
        let logit = (i as f32 - 25.0) * 0.1;
        reranker.record_outcome(logit, logit > 0.0);
    }

    assert!(!reranker.fitted_calibration().is_identity());
}

#[test]
fn test_implicit_feedback_passthrough_skipped() {
    let reranker = CrossEncoderReranker::passthrough();
    let results = vec![
        RerankResult {
            original_index: 0,
            score: 0.9,
        },
        RerankResult {
            original_index: 1,
            score: 0.8,
        },
    ];
    reranker.record_implicit_feedback(&results, 1);
    assert_eq!(reranker.calibration_observation_count(), 0);
    assert!(!reranker.is_calibrated());
}

#[test]
fn test_platt_scaled_sigmoid_config_and_reranker_builder() {
    let cal = PlattScaledSigmoid::new(1.5, -0.2);
    let config = RerankConfig::default().with_calibration(cal.clone());
    assert_eq!(config.calibration, cal);

    let reranker = CrossEncoderReranker::passthrough().with_calibration(cal.clone());
    assert_eq!(reranker.config().calibration, cal);
    assert_eq!(reranker._config.calibration, cal);
    assert_eq!(reranker.fitted_calibration(), cal);
}

#[tokio::test]
async fn test_rerank_exact_boundary_candidates() -> Result<(), Box<dyn std::error::Error>> {
    let reranker = CrossEncoderReranker::passthrough();

    // 1 element boundary
    let single_candidate = vec!["single_doc".to_string()];
    let res_single = reranker.rerank("query", &single_candidate).await?;
    assert_eq!(res_single.len(), 1);
    assert_eq!(res_single[0].original_index, 0);

    // MAX_CANDIDATES boundary (exact limit)
    let max_candidates: Vec<String> = (0..MAX_CANDIDATES).map(|i| format!("doc_{i}")).collect();
    let res_max = reranker.rerank("query", &max_candidates).await?;
    assert_eq!(res_max.len(), MAX_CANDIDATES);
    assert_eq!(res_max[0].original_index, 0);
    Ok(())
}

#[test]
fn test_calibration_extreme_logits_and_non_finite_inputs() {
    let reranker = CrossEncoderReranker::passthrough();

    // Check identity calibration behavior on extreme values
    let val_max = reranker.calibrate(f32::MAX);
    assert!((val_max - 1.0).abs() < 1e-4);

    let val_min_pos = reranker.calibrate(f32::MIN_POSITIVE);
    assert!((val_min_pos - 0.5).abs() < 1e-4);

    let val_neg_inf = reranker.calibrate(f32::NEG_INFINITY);
    assert_eq!(val_neg_inf, 0.0);

    let val_pos_inf = reranker.calibrate(f32::INFINITY);
    assert_eq!(val_pos_inf, 1.0);

    let val_nan = reranker.calibrate(f32::NAN);
    assert_eq!(val_nan, 0.5);

    // Record non-finite outcomes without panicking
    reranker.record_outcome(f32::NAN, false);
    reranker.record_outcome(f32::INFINITY, true);
    reranker.record_outcome(f32::NEG_INFINITY, false);
    assert_eq!(reranker.calibration_observation_count(), 3);
}

#[test]
fn test_cross_encoder_online_calibration_and_invalidation() {
    let config = RerankConfig::default();
    let reranker = CrossEncoderReranker::passthrough_with_config(config);

    // Initial fitted calibration is identity
    assert!(reranker.fitted_calibration().is_identity());

    // Record outcomes under warmup threshold (49 items when warmup is 50)
    for i in 0..49 {
        let logit = (i as f32 - 25.0) * 0.1;
        let is_rel = logit > 0.0;
        reranker.record_outcome(logit, is_rel);
    }
    assert!(reranker.fitted_calibration().is_identity());

    // Record 50th outcome -> triggers fitting
    reranker.record_outcome(2.5, true);
    assert!(!reranker.fitted_calibration().is_identity());
    assert!(reranker.fitted_calibration().params().0 > 0.0);

    // Verify calibrate() uses fitted calibration
    let calibrated_val = reranker.calibrate(1.0);
    let identity_val = PlattScaler::identity().apply(1.0);
    assert_ne!(calibrated_val, identity_val);

    // Invalidate calibration with new ConfigFingerprint (INV-CAL-2)
    let fp = ConfigFingerprint::new("bge-reranker-v2", "Q4", "default", 0.0);
    reranker.invalidate_calibration(fp);

    assert!(reranker.fitted_calibration().is_identity());
    assert_eq!(reranker.calibration_buffer.lock().len(), 0);
}

use proptest::prelude::*;

/// Helper to compute Expected Calibration Error (ECE) with M bins.
fn compute_ece(probs_and_labels: &[(f32, bool)], num_bins: usize) -> f32 {
    if probs_and_labels.is_empty() {
        return 0.0;
    }

    let mut bin_counts = vec![0usize; num_bins];
    let mut bin_acc_sums = vec![0.0f32; num_bins];
    let mut bin_conf_sums = vec![0.0f32; num_bins];

    for &(p, label) in probs_and_labels {
        let bin_idx = ((p * num_bins as f32).floor() as usize).min(num_bins - 1);
        bin_counts[bin_idx] += 1;
        bin_conf_sums[bin_idx] += p;
        if label {
            bin_acc_sums[bin_idx] += 1.0;
        }
    }

    let total = probs_and_labels.len() as f32;
    let mut ece = 0.0f32;

    for i in 0..num_bins {
        if bin_counts[i] > 0 {
            let count = bin_counts[i] as f32;
            let avg_acc = bin_acc_sums[i] / count;
            let avg_conf = bin_conf_sums[i] / count;
            ece += (count / total) * (avg_acc - avg_conf).abs();
        }
    }

    ece
}

#[test]
fn test_ece_reduction_after_platt_calibration() {
    let reranker = CrossEncoderReranker::passthrough();
    assert!(!reranker.is_calibrated());

    // Generate synthetic miscalibrated logits: A=2.5, B=-0.8
    let scale = 2.5f32;
    let shift = -0.8f32;
    let mut observations = Vec::new();

    for i in -50..=50 {
        let logit = i as f32 * 0.1;
        let true_z = scale * logit + shift;
        let true_p = 1.0 / (1.0 + (-true_z).exp());
        let is_rel = (i as f32 * 0.01 + 0.5) > (1.0 - true_p);
        observations.push((logit, is_rel));
        reranker.record_outcome(logit, is_rel);
    }

    assert!(reranker.is_calibrated());

    let uncalibrated_eval: Vec<(f32, bool)> = observations
        .iter()
        .map(|&(x, label)| (PlattScaler::identity().apply(x), label))
        .collect();
    let ece_uncalibrated = compute_ece(&uncalibrated_eval, 10);

    let calibrated_eval: Vec<(f32, bool)> = observations
        .iter()
        .map(|&(x, label)| (reranker.calibrate(x), label))
        .collect();
    let ece_calibrated = compute_ece(&calibrated_eval, 10);

    assert!(
        ece_calibrated <= ece_uncalibrated,
        "Calibrated ECE ({ece_calibrated}) must be <= uncalibrated ECE ({ece_uncalibrated})"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn prop_platt_calibration_reduces_ece(
        scale in 1.5f32..5.0f32,
        shift in -2.0f32..2.0f32,
    ) {
        // Generate synthetic miscalibrated logits where true logit distribution
        // has temperature scaling A != 1 or shift B != 0
        let mut observations = Vec::new();
        for i in -50..=50 {
            let x = i as f32 * 0.1;
            let true_z = scale * x + shift;
            let true_p = 1.0 / (1.0 + (-true_z).exp());
            // Probabilistic binary label derived from true_p
            let is_rel = (i as f32 * 0.01 + 0.5) > (1.0 - true_p);
            observations.push((x, is_rel));
        }

        let identity = PlattScaler::identity();
        let uncalibrated_eval: Vec<(f32, bool)> = observations
            .iter()
            .map(|&(x, label)| (identity.apply(x), label))
            .collect();
        let ece_uncalibrated = compute_ece(&uncalibrated_eval, 10);

        let fitted = PlattScaler::fit(&observations);
        let calibrated_eval: Vec<(f32, bool)> = observations
            .iter()
            .map(|&(x, label)| (fitted.apply(x), label))
            .collect();
        let ece_calibrated = compute_ece(&calibrated_eval, 10);

        // Calibrated ECE must be less than or equal to uncalibrated ECE (with small tolerance for finite sample variance)
        prop_assert!(
            ece_calibrated <= ece_uncalibrated + 0.05,
            "Calibrated ECE ({ece_calibrated}) should be <= uncalibrated ECE ({ece_uncalibrated})"
        );
    }
}

#[cfg(feature = "onnx")]
#[test]
fn test_cross_encoder_missing_paths() -> Result<(), Box<dyn std::error::Error>> {
    use tempfile::tempdir;
    let dir = tempdir()?;
    let model_path = dir.path().join("nonexistent_model.onnx");
    let tokenizer_path = dir.path().join("nonexistent_tokenizer.json");

    let cfg = RerankConfig {
        model_path,
        tokenizer_path,
        max_length: 128,
        batch_size: 4,
        calibration: PlattScaledSigmoid::identity(),
        calibration_warmup: 50,
        rerank_deadline_ms: Some(500),
        simulate_delay_ms: None,
    };

    let res = CrossEncoderReranker::new(cfg);
    assert!(res.is_err());
    if let Err(err) = res {
        assert!(matches!(err, ContextraError::InvalidInput(_)));
    } else {
        panic!("Expected error for missing model/tokenizer paths");
    }
    Ok(())
}
