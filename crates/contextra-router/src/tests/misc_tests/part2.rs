use super::super::fixtures::*;
use crate::{DecisionId, DecisionIdGenerator, RoutingOutcome, SlmProfile};
use contextra_db::{Contextra, ContextraConfig};
use contextra_types::TokenBudget;
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_confidence_metrics_serde() -> Result<(), Box<dyn std::error::Error>> {
    use crate::router::ConfidenceMetrics;

    let uncal = ConfidenceMetrics {
        score_lower: None,
        score_upper: None,
        calibrated: false,
        quantile_threshold: 0.5,
        non_conformity_score: 0.2,
        selection_margin: 1.5,
    };

    let json_uncal = serde_json::to_string(&uncal)?;
    let val_uncal: serde_json::Value = serde_json::from_str(&json_uncal)?;
    assert_eq!(val_uncal["calibrated"], false);
    assert_eq!(val_uncal["non_conformity_score"], 0.2);
    assert_eq!(val_uncal["selection_margin"], 1.5);
    assert_eq!(val_uncal["quantile_threshold"], 0.5);

    let deserialized_uncal: ConfidenceMetrics = serde_json::from_str(&json_uncal)?;
    assert_eq!(deserialized_uncal, uncal);

    let cal = ConfidenceMetrics {
        score_lower: Some(0.4),
        score_upper: Some(0.8),
        calibrated: true,
        quantile_threshold: 0.6,
        non_conformity_score: 0.1,
        selection_margin: 2.0,
    };

    let json_cal = serde_json::to_string(&cal)?;
    let val_cal: serde_json::Value = serde_json::from_str(&json_cal)?;
    assert_eq!(val_cal["calibrated"], true);
    assert_eq!(val_cal["score_lower"], 0.4);
    assert_eq!(val_cal["score_upper"], 0.8);
    assert_eq!(val_cal["quantile_threshold"], 0.6);
    assert_eq!(val_cal["non_conformity_score"], 0.1);
    assert_eq!(val_cal["selection_margin"], 2.0);

    let deserialized_cal: ConfidenceMetrics = serde_json::from_str(&json_cal)?;
    assert_eq!(deserialized_cal, cal);

    Ok(())
}

#[test]
fn test_serde_helpers_sorted_u64_set() -> Result<(), Box<dyn std::error::Error>> {
    use crate::serde_helpers::sorted_u64_set;
    use std::collections::HashSet;

    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct TestContainer {
        #[serde(with = "sorted_u64_set")]
        set: HashSet<u64>,
    }

    let container = TestContainer {
        set: [42, 10, 5, 100].into_iter().collect(),
    };

    let json = serde_json::to_string(&container)?;
    assert_eq!(json, r#"{"set":[5,10,42,100]}"#);

    let deserialized: TestContainer = serde_json::from_str(&json)?;
    assert_eq!(deserialized, container);
    Ok(())
}

#[tokio::test]
async fn test_record_outcome_trains_calibration() {
    let dir = tempfile::tempdir().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap();
    let collection = db.collection("default").await.unwrap();

    let vec_coding = vec![1.0, 0.0, 0.0, 0.0];
    collection
        .insert(
            "coding_doc",
            &vec_coding,
            Some(json!({"text": "function test() {}"})),
        )
        .await
        .unwrap();

    use contextra_types::ConfigFingerprint;
    let fp = ConfigFingerprint::new("llama-3b", "F16", "template", 0.1);
    let profile = SlmProfile::new(
        "default",
        "http://localhost:9999/mcp",
        vec![],
        TokenBudget::new(1000, 100),
        0.01,
    )
    .with_fingerprint(fp);

    let router = create_test_router(collection, vec![profile], None);
    let decision = router.route(&vec_coding, "function test").await.unwrap();

    let cal_before = router.calibration_stats();

    let recorded = router.record_outcome(decision.decision_id, RoutingOutcome::Success);
    assert!(recorded);

    let cal_after = router.calibration_stats();
    assert!(
        cal_after["default"].conformal.window_total > cal_before["default"].conformal.window_total,
        "window_total should increase after record_outcome"
    );
}

#[test]
fn test_decision_id_and_routing_outcome_methods() {
    let gen1 = DecisionIdGenerator::new(0);
    let gen2 = DecisionIdGenerator::new(0);
    let id1 = gen1.next();
    let id2 = gen2.next();
    assert_eq!(id1.inner(), 0);
    assert_eq!(id2.inner(), 0);

    let id1_next = gen1.next();
    assert_eq!(id1_next.inner(), 1);
    assert_ne!(id1.inner(), id1_next.inner());

    let success = RoutingOutcome::Success;
    let escalated = RoutingOutcome::Escalated {
        escalated_to: "large-slm".to_string(),
    };
    let rejected = RoutingOutcome::Rejected {
        reason: Some("incorrect answer".to_string()),
    };

    assert_eq!(success.non_conformity_score(), 0.0);
    assert_eq!(escalated.non_conformity_score(), 0.7);
    assert_eq!(rejected.non_conformity_score(), 1.0);
}

#[tokio::test]
async fn test_record_outcome_unknown_id_returns_false() {
    let dir = tempfile::tempdir().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap();
    let collection = db.collection("default").await.unwrap();

    let profile = SlmProfile::new(
        "default",
        "http://localhost:9999/mcp",
        vec![],
        TokenBudget::new(1000, 100),
        0.01,
    );

    let router = create_test_router(collection, vec![profile], None);
    let unknown_id = DecisionId::from_raw(9999);

    assert!(!router.record_outcome(unknown_id, RoutingOutcome::Success));
}

#[tokio::test]
async fn test_lyapunov_drift_status_integration() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;

    let profile = SlmProfile::new(
        "test-slm",
        "http://localhost:9999/mcp",
        vec![],
        TokenBudget::new(1000, 100),
        0.01,
    );

    let router = create_test_router(collection, vec![profile], None);

    // Initial status for unknown profile should be None
    assert_eq!(router.drift_status("unknown-slm"), None);

    // Set baseline distribution for test-slm
    let baseline: Vec<f32> = (0..100).map(|i| (i as f32) / 100.0).collect();
    assert!(router.set_lyapunov_baseline("test-slm", &baseline));

    // Status before route call
    assert_eq!(router.drift_status("test-slm"), None);
    Ok(())
}

#[test]
fn test_lyapunov_observe_score_and_analyze_drift_detection() {
    use crate::lyapunov::{LyapunovDriftWatcher, LyapunovResult};

    let mut watcher = LyapunovDriftWatcher::new(20);
    let baseline: Vec<f32> = (0..100).map(|i| (i as f32) / 100.0 * 0.2).collect();
    watcher.set_baseline(&baseline);

    // Feed 50 stable scores close to baseline
    for i in 0..50 {
        let score = (i % 20) as f32 / 100.0;
        watcher.observe_score(score);
    }

    match watcher.analyze() {
        LyapunovResult::Stable { lyapunov_exponent } => {
            assert!(
                lyapunov_exponent <= 0.05,
                "Expected lyapunov_exponent <= 0.05, got {}",
                lyapunov_exponent
            );
        }
        other => panic!("Expected Stable after 50 stable scores, got {:?}", other),
    }

    // Feed 10 outlier scores (scores = 0.95)
    let mut last_res = LyapunovResult::InsufficientData;
    for _ in 0..10 {
        last_res = watcher.observe_score(0.95);
    }

    assert_eq!(watcher.analyze(), last_res);
    match last_res {
        LyapunovResult::DriftDetected {
            lyapunov_exponent,
            reason,
        } => {
            assert!(lyapunov_exponent > 0.0);
            assert!(reason.kl_divergence > 0.0);
        }
        other => panic!(
            "Expected DriftDetected after 10 outlier scores, got {:?}",
            other
        ),
    }
}

#[test]
fn test_lyapunov_update_empty_scores_or_uninitialized_baseline() {
    use crate::lyapunov::{LyapunovDriftWatcher, LyapunovResult};

    let mut watcher = LyapunovDriftWatcher::new(10);
    // Empty update without baseline returns InsufficientData
    let res_empty = watcher.update(&[]);
    assert_eq!(res_empty, LyapunovResult::InsufficientData);

    // Update with less than 30 auto-baseline scores returns InsufficientData
    let res_small = watcher.update(&[0.1, 0.2]);
    assert_eq!(res_small, LyapunovResult::InsufficientData);
    assert_eq!(watcher.baseline_distribution.len(), 2);

    // Score boundary clamping in histogram bins
    let mut watcher_clamped = LyapunovDriftWatcher::new(5);
    watcher_clamped.set_baseline(&[0.5; 50]);
    // Scores out of [0.0, 1.0] bound (-0.5, 1.5, NaN) clamped safely without panic
    let res_clamped = watcher_clamped.update(&[-0.5, 1.5, f32::NAN]);
    assert_eq!(res_clamped, LyapunovResult::InsufficientData);
}

#[tokio::test]
async fn test_pending_decisions_evicted_after_ttl() -> Result<(), Box<dyn std::error::Error>> {
    use crate::router::{MAX_PENDING_DECISIONS, PENDING_DECISION_TTL};

    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;

    let vec_data = vec![1.0, 0.0, 0.0, 0.0];
    collection
        .insert(
            "doc1",
            &vec_data,
            Some(json!({"text": "eviction test content"})),
        )
        .await?;

    let profile = SlmProfile::new(
        "p1",
        "http://localhost:8000/mcp",
        vec![],
        TokenBudget::new(1000, 100),
        0.1,
    );

    let router = create_test_router(collection, vec![profile], None);

    // Fill pending_decisions with MAX_PENDING_DECISIONS stale entries older than TTL (300s)
    let stale_timestamp =
        std::time::Instant::now() - (PENDING_DECISION_TTL + std::time::Duration::from_secs(10));
    {
        let mut map = router.pending_decisions.write();
        for _ in 0..MAX_PENDING_DECISIONS {
            map.insert(
                router.decision_ids.next(),
                ("p1".to_string(), stale_timestamp),
            );
        }
    }

    assert_eq!(router.pending_decision_count(), MAX_PENDING_DECISIONS);

    // Calling route() triggers evict_stale_decisions()
    let decision = router.route(&vec_data, "eviction test content").await?;

    // Stale entries should be evicted, leaving only the new decision
    assert_eq!(router.pending_decision_count(), 1);
    assert!(router.record_outcome(decision.decision_id, RoutingOutcome::Success));

    Ok(())
}

#[tokio::test]
async fn test_pending_decisions_max_capacity_enforced() -> Result<(), Box<dyn std::error::Error>> {
    use crate::router::{MAX_PENDING_DECISIONS, PENDING_DECISION_TTL};

    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;

    let vec_data = vec![1.0, 0.0, 0.0, 0.0];
    collection
        .insert(
            "doc1",
            &vec_data,
            Some(json!({"text": "capacity test content"})),
        )
        .await?;

    let profile = SlmProfile::new(
        "p1",
        "http://localhost:8000/mcp",
        vec![],
        TokenBudget::new(1000, 100),
        0.1,
    );

    let router = create_test_router(collection, vec![profile], None);

    // Fill pending_decisions with MAX_PENDING_DECISIONS + 1 stale entries without record_outcome
    let stale_timestamp =
        std::time::Instant::now() - (PENDING_DECISION_TTL + std::time::Duration::from_secs(10));
    {
        let mut map = router.pending_decisions.write();
        for _ in 0..=(MAX_PENDING_DECISIONS) {
            map.insert(
                router.decision_ids.next(),
                ("p1".to_string(), stale_timestamp),
            );
        }
    }

    assert!(router.pending_decision_count() > MAX_PENDING_DECISIONS);

    // Call route()
    let _decision = router.route(&vec_data, "capacity test content").await?;

    // Verify map size is <= MAX_PENDING_DECISIONS
    assert!(router.pending_decision_count() <= MAX_PENDING_DECISIONS);

    Ok(())
}

#[test]
fn test_cascade_confidence_uses_conformal_alpha() -> Result<(), Box<dyn std::error::Error>> {
    use crate::profile::ProfileCalibrationState;
    use crate::router::COMMUNITY_RELEVANCE_BOOST;
    use contextra_types::{ConfigFingerprint, ContextChunk, DocId};
    use std::collections::HashMap;

    let fp = ConfigFingerprint::new("model1", "Q4_K_M", "prompt", 0.7);
    let profile = SlmProfile::new(
        "alpha-slm",
        "http://localhost/alpha",
        vec![1],
        TokenBudget::new(1000, 100),
        0.4,
    )
    .with_fingerprint(fp.clone());

    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let rt = tokio::runtime::Runtime::new()?;
    let db = rt.block_on(Contextra::open_with_config(dir.path(), config))?;
    let collection = rt.block_on(db.collection("default"))?;

    let router = create_test_router(collection, vec![profile.clone()], None);

    let mut cal_state = ProfileCalibrationState::new(0.4);
    cal_state.conformal.alpha = 0.15;
    cal_state.conformal.window_total = 100; // calibrated
    cal_state.last_calibrated_fingerprint = Some(fp);

    let mut calibration: HashMap<String, ProfileCalibrationState> = HashMap::new();
    calibration.insert("alpha-slm".to_string(), cal_state);

    let chunk = ContextChunk {
        doc_id: DocId::new(1),
        content: "alpha test".to_string(),
        relevance: 0.5,
        token_count: 5,
        metadata: None,
        contextual_prefix: None,
        links: Vec::new(),
    };
    let chunks = vec![(chunk, Some(1))];

    let (_, _, metrics) = router.select_profile_cascade(&chunks, &[profile], &mut calibration)?;

    let score = 0.5 * COMMUNITY_RELEVANCE_BOOST;
    assert!(metrics.calibrated);
    let expected_lower = score * (1.0 - 0.15); // score * 0.85
    let expected_upper = score * (1.0 + 0.15); // score * 1.15

    let lower = metrics.score_lower.ok_or("score_lower missing")?;
    let upper = metrics.score_upper.ok_or("score_upper missing")?;

    assert!(
        (lower - expected_lower).abs() < 1e-5,
        "Expected {expected_lower}, got {lower}"
    );
    assert!(
        (upper - expected_upper).abs() < 1e-5,
        "Expected {expected_upper}, got {upper}"
    );

    Ok(())
}

#[test]
fn test_cascade_non_conformity_score_not_zero() -> Result<(), Box<dyn std::error::Error>> {
    use crate::profile::ProfileCalibrationState;
    use contextra_types::{ContextChunk, DocId};
    use std::collections::HashMap;

    let profile = SlmProfile::new(
        "nc-slm",
        "http://localhost/nc",
        vec![1],
        TokenBudget::new(1000, 100),
        0.1,
    );

    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let rt = tokio::runtime::Runtime::new()?;
    let db = rt.block_on(Contextra::open_with_config(dir.path(), config))?;
    let collection = rt.block_on(db.collection("default"))?;

    let router = create_test_router(collection, vec![profile.clone()], None);

    let mut cal_state = ProfileCalibrationState::new(0.1);
    cal_state.conformal.quantile_threshold = 0.8; // higher than score
    let mut calibration: HashMap<String, ProfileCalibrationState> = HashMap::new();
    calibration.insert("nc-slm".to_string(), cal_state);

    let chunk = ContextChunk {
        doc_id: DocId::new(1),
        content: "nc test".to_string(),
        relevance: 0.4,
        token_count: 5,
        metadata: None,
        contextual_prefix: None,
        links: Vec::new(),
    };
    let chunks = vec![(chunk, Some(1))];

    let (_, _, metrics) = router.select_profile_cascade(&chunks, &[profile], &mut calibration)?;

    assert!(
        metrics.non_conformity_score != 0.0,
        "non_conformity_score should not be 0.0"
    );
    let expected_nc = (1.0 - (0.48 / 0.8f32)).clamp(0.0, 1.0);
    assert!(
        (metrics.non_conformity_score - expected_nc).abs() < 1e-5,
        "Expected {expected_nc}, got {}",
        metrics.non_conformity_score
    );

    Ok(())
}

#[test]
fn test_quantization_level_default_and_try_new() {
    use crate::profile::QuantizationLevel;

    assert_eq!(QuantizationLevel::default(), QuantizationLevel::Unknown);

    let valid = SlmProfile::try_new(
        "valid",
        "http://localhost:8000",
        vec![1],
        TokenBudget::default(),
        0.5,
    );
    assert!(valid.is_ok());

    let invalid_name = SlmProfile::try_new(
        "   ",
        "http://localhost:8000",
        vec![1],
        TokenBudget::default(),
        0.5,
    );
    assert!(invalid_name.is_err());

    let invalid_endpoint =
        SlmProfile::try_new("valid", "   ", vec![1], TokenBudget::default(), 0.5);
    assert!(invalid_endpoint.is_err());

    let invalid_score = SlmProfile::try_new(
        "valid",
        "http://localhost:8000",
        vec![1],
        TokenBudget::default(),
        f32::NAN,
    );
    assert!(invalid_score.is_err());

    let invalid_neg_score = SlmProfile::try_new(
        "valid",
        "http://localhost:8000",
        vec![1],
        TokenBudget::default(),
        -0.1,
    );
    assert!(invalid_neg_score.is_err());
}

#[test]
fn test_conformal_calibrator_empirical_rate_and_reset() {
    use crate::profile::ConformalCalibrator;

    let mut cal = ConformalCalibrator::default();
    assert_eq!(cal.empirical_error_rate(), 0.0);

    cal.update(0.9);
    assert_eq!(cal.empirical_error_rate(), 1.0);

    cal.update(0.1);
    assert_eq!(cal.empirical_error_rate(), 0.5);

    cal.reset_window();
    assert_eq!(cal.window_errors, 0);
    assert_eq!(cal.window_total, 0);
    assert_eq!(cal.empirical_error_rate(), 0.0);
}

#[test]
fn test_profile_calibration_state_edge_cases() {
    use crate::profile::ProfileCalibrationState;
    use contextra_types::ConfigFingerprint;

    let mut state = ProfileCalibrationState::new(0.5);
    assert_eq!(state.average_confidence(), 1.0);

    assert!(!state.is_calibrated(None));

    let fp = ConfigFingerprint::new("model", "F16", "hash", 0.7);
    assert!(!state.is_calibrated(Some(&fp)));

    state.conformal.window_total = 100;
    state.last_calibrated_fingerprint = Some(fp.clone());
    assert!(state.is_calibrated(Some(&fp)));

    state.check_and_invalidate_fingerprint(None);
    assert_eq!(state.last_calibrated_fingerprint, None);
    assert_eq!(state.conformal.window_total, 0);
}

#[test]
fn test_lyapunov_uncovered_branch_paths() {
    use crate::lyapunov::{LyapunovDriftWatcher, LyapunovResult};

    let mut watcher = LyapunovDriftWatcher::new(10);
    // 1. update with empty baseline and empty current_scores -> InsufficientData
    let res = watcher.update(&[]);
    assert_eq!(res, LyapunovResult::InsufficientData);

    // 2. update with empty baseline and small current_scores (< 30) -> InsufficientData
    let res = watcher.update(&[0.1, 0.2]);
    assert_eq!(res, LyapunovResult::InsufficientData);

    // 3. update with baseline populated, but current_scores empty -> returns latest_result or InsufficientData
    watcher.set_baseline(&(0..35).map(|i| i as f32 / 35.0).collect::<Vec<_>>());
    let res = watcher.update(&[]);
    assert_eq!(res, LyapunovResult::InsufficientData);
}

#[test]
fn test_slm_profile_nan_and_negative_validation_bounds() {
    // min_relevance_score NaN validation
    let res_nan_score = SlmProfile::try_new(
        "nan-score-slm",
        "http://localhost:8000/mcp",
        vec![1],
        TokenBudget::new(1000, 100),
        f32::NAN,
    );
    assert!(res_nan_score.is_err());
    assert!(res_nan_score
        .unwrap_err()
        .to_string()
        .contains("min_relevance_score"));

    // min_relevance_score negative validation
    let res_neg_score = SlmProfile::try_new(
        "neg-score-slm",
        "http://localhost:8000/mcp",
        vec![1],
        TokenBudget::new(1000, 100),
        -0.5,
    );
    assert!(res_neg_score.is_err());
    assert!(res_neg_score
        .unwrap_err()
        .to_string()
        .contains("min_relevance_score"));

    // resource_cost_estimate NaN validation
    let profile_nan_cost = SlmProfile::new(
        "nan-cost-slm",
        "http://localhost:8000/mcp",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    )
    .with_resource_cost_estimate(f32::NAN);
    assert!(profile_nan_cost.validate().is_err());

    // resource_cost_estimate negative validation
    let profile_neg_cost = SlmProfile::new(
        "neg-cost-slm",
        "http://localhost:8000/mcp",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    )
    .with_resource_cost_estimate(-10.0);
    assert!(profile_neg_cost.validate().is_err());
}

#[tokio::test]
async fn test_overall_drift_status_aggregation() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;

    let p1 = SlmProfile::new("p1", "http://ep1", vec![1], TokenBudget::default(), 0.1);
    let p2 = SlmProfile::new("p2", "http://ep2", vec![2], TokenBudget::default(), 0.1);

    let router = create_test_router(collection, vec![p1, p2], None);

    // Initial state before score observation -> "unbekannt"
    assert_eq!(router.overall_drift_status(), "unbekannt");

    // Force stable result on p1 -> "stabil"
    {
        let current = router.state.load_full();
        let mut new_state = (*current).clone();
        if let Some(watcher) = new_state.lyapunov_watchers.get_mut("p1") {
            watcher.latest_result = Some(crate::lyapunov::LyapunovResult::Stable {
                lyapunov_exponent: -0.1,
            });
        }
        router.state.store(Arc::new(new_state));
    }

    assert_eq!(router.overall_drift_status(), "stabil");

    // Set baseline for p1 and observe scores that trigger warning or critical
    let baseline: Vec<f32> = (0..100).map(|i| (i as f32 / 100.0) * 0.1).collect();
    router.set_lyapunov_baseline("p1", &baseline);

    {
        let current = router.state.load_full();
        let mut new_state = (*current).clone();
        if let Some(watcher) = new_state.lyapunov_watchers.get_mut("p1") {
            // Force a warning level drift
            watcher.latest_result = Some(crate::lyapunov::LyapunovResult::DriftDetected {
                lyapunov_exponent: 0.1,
                reason: crate::lyapunov::DriftReason {
                    kl_divergence: 0.5,
                    lyapunov_exponent: 0.1,
                },
            });
        }
        router.state.store(Arc::new(new_state));
    }

    assert_eq!(router.overall_drift_status(), "warnung");

    {
        let current = router.state.load_full();
        let mut new_state = (*current).clone();
        if let Some(watcher) = new_state.lyapunov_watchers.get_mut("p2") {
            // Force a critical level drift (> 0.2)
            watcher.latest_result = Some(crate::lyapunov::LyapunovResult::DriftDetected {
                lyapunov_exponent: 0.3,
                reason: crate::lyapunov::DriftReason {
                    kl_divergence: 1.5,
                    lyapunov_exponent: 0.3,
                },
            });
        }
        router.state.store(Arc::new(new_state));
    }

    assert_eq!(router.overall_drift_status(), "kritisch");
    Ok(())
}

#[test]
fn test_slm_profile_estimated_cost_fallback() {
    let budget = TokenBudget::new(4096, 512);
    let profile_default =
        SlmProfile::new("default-cost", "http://mcp", vec![], budget.clone(), 0.1);
    assert_eq!(profile_default.estimated_cost(), 4096.0);

    let profile_explicit = SlmProfile::new("explicit-cost", "http://mcp", vec![], budget, 0.1)
        .with_resource_cost_estimate(12.5);
    assert_eq!(profile_explicit.estimated_cost(), 12.5);
}
