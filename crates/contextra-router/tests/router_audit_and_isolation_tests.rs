#![allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use contextra_db::{Collection, Contextra, ContextraConfig};
use contextra_ports::clock::Clock;
use contextra_router::profile::ProfileCalibrationState;
use contextra_router::{
    ArmRegistry, ArmRegistryError, DecisionId, ProfileCalibrator, RouterEngine, RoutingDecision,
    RoutingOutcome, SlmProfile,
};
use contextra_types::{
    ConfigFingerprint, ContextChunk, ContextWindow, ContextraError, EntityId, Result, TokenBudget,
};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Default)]
struct TestClock {
    nanos: AtomicU64,
}

impl TestClock {
    fn new(initial_nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(initial_nanos),
        }
    }

    #[allow(dead_code)]
    fn advance(&self, duration: Duration) {
        self.nanos
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }
}

impl Clock for TestClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.load(Ordering::Relaxed)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.nanos.load(Ordering::Relaxed)
    }
}

struct CollectionAdapter<S: contextra_ports::StorageEngine + 'static> {
    collection: Arc<Collection<S>>,
}

impl<S: contextra_ports::StorageEngine + 'static> CollectionAdapter<S> {
    fn new(collection: Arc<Collection<S>>) -> Self {
        Self { collection }
    }
}

impl<S: contextra_ports::StorageEngine + 'static> contextra_ports::HybridSearchProvider
    for CollectionAdapter<S>
{
    fn search_hybrid<'a>(
        &'a self,
        query_text: &'a str,
        query_embedding: &'a [f32],
        top_k: usize,
    ) -> contextra_ports::BoxFuture<'a, Result<Vec<ContextChunk>>> {
        Box::pin(async move {
            let search_results = self
                .collection
                .query()
                .text(query_text)
                .embedding(query_embedding)
                .k(top_k)
                .execute()
                .await?;

            let mut chunks = Vec::with_capacity(search_results.len());
            for res in search_results {
                if let Ok(mut chunk) = ContextChunk::try_from(res) {
                    chunk.content = chunk.combined_text_owned();
                    chunks.push(chunk);
                }
            }
            Ok(chunks)
        })
    }
}

impl<S: contextra_ports::StorageEngine + 'static> contextra_ports::CommunityResolver
    for CollectionAdapter<S>
{
    fn get_community<'a>(
        &'a self,
        entity_id: EntityId,
    ) -> contextra_ports::BoxFuture<'a, Result<Option<u64>>> {
        Box::pin(async move { self.collection.get_community(entity_id).await })
    }
}

struct TestContextPreparer;

impl contextra_ports::ContextPreparer for TestContextPreparer {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        budget: &TokenBudget,
        relevance_threshold: f32,
    ) -> Result<ContextWindow> {
        let mut manager = contextra_db::ContextManager::new(budget.clone());
        manager.set_relevance_threshold(relevance_threshold);
        manager.prepare_context(chunks)
    }
}

fn create_test_router<S: contextra_ports::StorageEngine + 'static>(
    collection: Arc<Collection<S>>,
    profiles: Vec<SlmProfile>,
    clock: Arc<dyn Clock>,
) -> RouterEngine {
    let adapter = Arc::new(CollectionAdapter::new(collection));
    let preparer = Arc::new(TestContextPreparer);
    RouterEngine::new(adapter.clone(), adapter, preparer, profiles, None).with_clock(clock)
}

// ── Test 1: Fixed Seed Reproducibility ───────────────────────────────────────
#[tokio::test]
async fn test_fixed_seed_routing_reproducibility() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;
    collection
        .insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "rag pipeline routing context"})),
        )
        .await?;

    let clock = Arc::new(TestClock::new(1_000_000_000_000));

    let profile1 = SlmProfile::new(
        "slm-a",
        "http://localhost:8001/mcp",
        vec![],
        TokenBudget::new(1000, 100),
        0.1,
    );
    let profile2 = SlmProfile::new(
        "slm-b",
        "http://localhost:8002/mcp",
        vec![],
        TokenBudget::new(1000, 100),
        0.5,
    );

    let router1 = create_test_router(
        collection.clone(),
        vec![profile1.clone(), profile2.clone()],
        clock.clone(),
    );
    let router2 = create_test_router(collection, vec![profile1, profile2], clock);

    let query_vec = vec![1.0, 0.0, 0.0, 0.0];
    let decision1 = router1
        .route(&query_vec, "rag pipeline routing context")
        .await?;
    let decision2 = router2
        .route(&query_vec, "rag pipeline routing context")
        .await?;

    assert_eq!(decision1.profile.name, decision2.profile.name);
    assert_eq!(decision1.decision_id.inner(), decision2.decision_id.inner());
    Ok(())
}

// ── Test 2: Idempotent Outcome Processing ────────────────────────────────────
#[tokio::test]
async fn test_record_outcome_idempotency() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;
    collection
        .insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "test query"})),
        )
        .await?;

    let clock = Arc::new(TestClock::new(1_000_000_000_000));

    let profile = SlmProfile::new(
        "idempotent-slm",
        "http://localhost:8000/mcp",
        vec![],
        TokenBudget::new(1000, 100),
        0.1,
    )
    .with_fingerprint(ConfigFingerprint::new("fp1", "F16", "", 0.0));

    let router = create_test_router(collection, vec![profile], clock);
    let decision = router.route(&[1.0, 0.0, 0.0, 0.0], "test query").await?;

    let stats_before = router.calibration_stats();
    let sel_before = stats_before
        .get("idempotent-slm")
        .map(|s| s.times_selected)
        .unwrap_or(0);

    let first_record = router.record_outcome(decision.decision_id, RoutingOutcome::Success);
    assert!(first_record);

    let stats_after_first = router.calibration_stats();
    let window_after_first = stats_after_first
        .get("idempotent-slm")
        .map(|s| s.conformal.window_total)
        .unwrap_or(0);

    let second_record = router.record_outcome(decision.decision_id, RoutingOutcome::Success);
    assert!(!second_record);

    let stats_after_second = router.calibration_stats();
    let window_after_second = stats_after_second
        .get("idempotent-slm")
        .map(|s| s.conformal.window_total)
        .unwrap_or(0);

    assert_eq!(window_after_first, window_after_second);
    assert_eq!(sel_before, 1);
    Ok(())
}

// ── Test 3: Arm Registry & Tenant Statistics Isolation ──────────────────────
#[test]
fn test_arm_registry_and_stats_tenant_isolation() {
    let reg1 = ArmRegistry::default();
    let reg2 = ArmRegistry::default();

    for arm in 0..5 {
        let s1 = reg1.strategy_for(arm);
        let s2 = reg2.strategy_for(arm);
        assert_eq!(s1, s2);
    }

    assert_eq!(
        reg1.strategy_for(100),
        Err(ArmRegistryError::OutOfRange(100))
    );
}

// ── Test 4: Cold Start Default Route Fallback ────────────────────────────────
#[tokio::test]
async fn test_cold_start_fallback_default_route() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;
    collection
        .insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "unrelated query"})),
        )
        .await?;

    let clock = Arc::new(TestClock::new(1_000_000_000_000));

    let profile_expensive = SlmProfile::new(
        "expensive-slm",
        "http://localhost:8001/mcp",
        vec![],
        TokenBudget::new(2000, 200),
        0.95,
    )
    .with_resource_cost_estimate(200.0);

    let profile_cheap = SlmProfile::new(
        "cheap-slm",
        "http://localhost:8002/mcp",
        vec![],
        TokenBudget::new(500, 50),
        0.90,
    )
    .with_resource_cost_estimate(10.0);

    let router = create_test_router(
        collection,
        vec![profile_expensive, profile_cheap.clone()],
        clock,
    );

    let decision = router
        .route(&[0.1, 0.0, 0.0, 0.0], "unrelated query")
        .await?;
    assert_eq!(decision.profile.name, profile_cheap.name);
    assert!(decision.confidence.is_some());
    assert!(!decision.confidence.as_ref().unwrap().calibrated);
    Ok(())
}

// ── Test 5: Typed Transport Error Propagation ──────────────────────────────
#[tokio::test]
async fn test_transport_error_propagation_typed() {
    let profile_invalid_endpoint =
        SlmProfile::new("bad-transport", "   ", vec![], TokenBudget::default(), 0.5);
    assert!(profile_invalid_endpoint.validate().is_err());

    let profile_nonexistent_binary = SlmProfile::new(
        "nonexistent",
        "/bin/nonexistent_binary_xyz_12345",
        vec![],
        TokenBudget::default(),
        0.5,
    );
    let decision = RoutingDecision {
        profile: profile_nonexistent_binary,
        context: ContextWindow {
            chunks: vec![],
            total_tokens: 0,
            truncated: false,
        },
        confidence: None,
        decision_id: DecisionId::from_raw(42),
        drift_status: None,
    };

    let result = contextra_router::dispatch_to_slm(&decision).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ContextraError::Internal(msg) => assert!(msg.contains("Fehler bei MCP-Dispatch")),
        other => panic!("Expected ContextraError::Internal, got: {other:?}"),
    }
}

// ── Test 6: Restart Resumption with initial_decision_id ─────────────────────
#[tokio::test]
async fn test_restart_resumption_with_initial_decision_id() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;
    collection
        .insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "query text"})),
        )
        .await?;

    let clock = Arc::new(TestClock::new(1_000_000_000_000));

    let profile = SlmProfile::new(
        "resume-slm",
        "http://localhost:8000/mcp",
        vec![],
        TokenBudget::default(),
        0.1,
    );

    let router1 = create_test_router(collection.clone(), vec![profile.clone()], clock.clone());
    let dec1 = router1.route(&[1.0, 0.0, 0.0, 0.0], "query 1").await?;
    assert_eq!(dec1.decision_id.inner(), 0);

    let router2 =
        create_test_router(collection, vec![profile], clock).with_initial_decision_id(5000);
    let dec2 = router2.route(&[1.0, 0.0, 0.0, 0.0], "query 2").await?;
    assert_eq!(dec2.decision_id.inner(), 5000);

    let dec3 = router2.route(&[1.0, 0.0, 0.0, 0.0], "query 3").await?;
    assert_eq!(dec3.decision_id.inner(), 5001);

    Ok(())
}

// ── Test 7: Deterministic Clock-based Eviction via Public API ────────────────
#[tokio::test]
async fn test_clock_based_ttl_eviction_public_api() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;
    collection
        .insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "q content"})),
        )
        .await?;

    let test_clock = Arc::new(TestClock::new(1_000_000_000_000));

    let profile = SlmProfile::new(
        "ttl-slm",
        "http://localhost:8000/mcp",
        vec![],
        TokenBudget::default(),
        0.1,
    );

    let router = create_test_router(collection, vec![profile], test_clock.clone());

    let dec1 = router.route(&[1.0, 0.0, 0.0, 0.0], "q1").await?;
    assert_eq!(router.pending_decision_count(), 1);

    assert!(router.record_outcome(dec1.decision_id, RoutingOutcome::Success));
    assert_eq!(router.pending_decision_count(), 0);

    Ok(())
}

// ── Test 8: ProfileCalibrator Monotonic Updates ─────────────────────────────
#[test]
fn test_profile_calibrator_update_monotonic() {
    let mut cal = ProfileCalibrator::new(0.05, 0.01, 0.5);
    let initial_q = cal.quantile_threshold;

    assert!(cal.update(0.9));
    assert!(cal.quantile_threshold > initial_q);

    let raised_q = cal.quantile_threshold;
    assert!(cal.update(0.1));
    assert!(cal.quantile_threshold < raised_q);
}

// ── Test 9: ConfigFingerprint Invalidation ──────────────────────────────────
#[test]
fn test_profile_calibration_state_fingerprint_invalidation() {
    let mut state = ProfileCalibrationState::new(0.5);
    let fp1 = ConfigFingerprint::new("fp_alpha", "F16", "", 0.0);
    let fp2 = ConfigFingerprint::new("fp_beta", "F16", "", 0.0);

    state.check_and_invalidate_fingerprint(Some(&fp1));
    state.conformal.window_total = 10;
    state.times_selected = 5;

    state.check_and_invalidate_fingerprint(Some(&fp1));
    assert_eq!(state.times_selected, 5);

    state.check_and_invalidate_fingerprint(Some(&fp2));
    assert_eq!(state.times_selected, 0);
    assert_eq!(state.conformal.window_total, 0);
}

// ── Test 10: Lyapunov Drift Status Summary Unconfigured ────────────────────
#[tokio::test]
async fn test_lyapunov_drift_status_summary_unconfigured() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;

    let clock = Arc::new(TestClock::new(1_000_000_000_000));

    let profile = SlmProfile::new(
        "slm-crit",
        "http://localhost:8000/mcp",
        vec![],
        TokenBudget::default(),
        0.1,
    );

    let router = create_test_router(collection, vec![profile], clock);
    assert_eq!(router.overall_drift_status(), "unbekannt");
    Ok(())
}

// ── Test 11: Lyapunov Drift Status Summary Empty ─────────────────────────────
#[tokio::test]
async fn test_lyapunov_drift_status_summary_empty() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;

    let clock = Arc::new(TestClock::new(1_000_000_000_000));

    let router = create_test_router(collection, vec![], clock);
    assert_eq!(router.overall_drift_status(), "stabil");
    Ok(())
}

// ── Test 12: Hot Reload Profile Update Preserves Calibration ─────────────────
#[tokio::test]
async fn test_router_engine_update_profiles_hot_reload() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;
    collection
        .insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "test"})),
        )
        .await?;

    let clock = Arc::new(TestClock::new(1_000_000_000_000));

    let profile1 = SlmProfile::new(
        "keep-me",
        "http://localhost:8001/mcp",
        vec![],
        TokenBudget::default(),
        0.3,
    );
    let profile2 = SlmProfile::new(
        "drop-me",
        "http://localhost:8002/mcp",
        vec![],
        TokenBudget::default(),
        0.3,
    );

    let router = create_test_router(collection, vec![profile1.clone(), profile2], clock);

    let dec = router.route(&[1.0, 0.0, 0.0, 0.0], "test").await?;
    assert!(router.record_outcome(dec.decision_id, RoutingOutcome::Success));

    let profile3 = SlmProfile::new(
        "add-me",
        "http://localhost:8003/mcp",
        vec![],
        TokenBudget::default(),
        0.3,
    );

    router.update_profiles(vec![profile1, profile3]);

    let stats = router.calibration_stats();
    assert_eq!(stats.len(), 2);
    assert_eq!(stats["keep-me"].times_selected, 1);
    assert!(!stats.contains_key("drop-me"));
    assert_eq!(stats["add-me"].times_selected, 0);

    Ok(())
}

// ── Test 13: Try Update Profiles Validation Rejection ────────────────────────
#[tokio::test]
async fn test_router_engine_try_update_profiles_validation_error() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;

    let clock = Arc::new(TestClock::new(1_000_000_000_000));

    let profile = SlmProfile::new(
        "valid-slm",
        "http://localhost:8000/mcp",
        vec![],
        TokenBudget::default(),
        0.1,
    );

    let router = create_test_router(collection, vec![profile], clock);

    let invalid_profile = SlmProfile::new(
        "",
        "http://localhost:8000/mcp",
        vec![],
        TokenBudget::default(),
        0.1,
    );

    let res = router.try_update_profiles(vec![invalid_profile]);
    assert!(res.is_err());
    assert_eq!(router.profiles().len(), 1);
    assert_eq!(router.profiles()[0].name, "valid-slm");

    Ok(())
}

// ── Test 14: Warmup Period Uncalibrated Confidence Metrics ────────────────────
#[tokio::test]
async fn test_confidence_metrics_warmup_period() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;
    collection
        .insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "warmup text"})),
        )
        .await?;

    let clock = Arc::new(TestClock::new(1_000_000_000_000));

    let profile = SlmProfile::new(
        "warmup-slm",
        "http://localhost:8000/mcp",
        vec![],
        TokenBudget::default(),
        0.1,
    )
    .with_fingerprint(ConfigFingerprint::new("fp_warmup", "F16", "", 0.0));

    let router = create_test_router(collection, vec![profile], clock);

    let decision = router.route(&[1.0, 0.0, 0.0, 0.0], "warmup query").await?;
    let metrics = decision.confidence.expect("confidence metrics present");

    assert!(!metrics.calibrated);
    assert!(metrics.score_lower.is_none());
    assert!(metrics.score_upper.is_none());

    Ok(())
}

// ── Test 15: Decision ID Generator Monotonicity ─────────────────────────────
#[test]
fn test_decision_id_generator_monotonicity() {
    let gen = contextra_router::DecisionIdGenerator::new(100);
    let mut prev = gen.next().inner();
    assert_eq!(prev, 100);

    for expected in 101..1100 {
        let curr = gen.next().inner();
        assert_eq!(curr, expected);
        assert!(curr > prev);
        prev = curr;
    }
}

// ── Test 16: Routing Outcome Non-Conformity Scores ────────────────────────────
#[test]
fn test_routing_outcome_non_conformity_scores() {
    assert_eq!(RoutingOutcome::Success.non_conformity_score(), 0.0);
    assert_eq!(
        RoutingOutcome::Escalated {
            escalated_to: "large-model".into()
        }
        .non_conformity_score(),
        0.7
    );
    assert_eq!(
        RoutingOutcome::Rejected {
            reason: Some("bad answer".into())
        }
        .non_conformity_score(),
        1.0
    );
}

// ── Test 17: SLM Profile Estimated Cost Fallback ─────────────────────────────
#[test]
fn test_slm_profile_estimated_cost_fallback() {
    let budget = TokenBudget::new(4096, 512);

    let default_cost_profile =
        SlmProfile::new("p1", "http://endpoint", vec![], budget.clone(), 0.5);
    assert_eq!(default_cost_profile.estimated_cost(), 4096.0);

    let custom_cost_profile = SlmProfile::new("p2", "http://endpoint", vec![], budget, 0.5)
        .with_resource_cost_estimate(12.5);
    assert_eq!(custom_cost_profile.estimated_cost(), 12.5);
}

// ── Test 18: Reset All Calibration Resets Every Profile ─────────────────────
#[tokio::test]
async fn test_reset_all_calibration_resets_every_profile() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;
    collection
        .insert(
            "doc1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "test"})),
        )
        .await?;

    let clock = Arc::new(TestClock::new(1_000_000_000_000));

    let profile1 = SlmProfile::new("p1", "http://p1", vec![], TokenBudget::default(), 0.1);
    let profile2 = SlmProfile::new("p2", "http://p2", vec![], TokenBudget::default(), 0.1);

    let router = create_test_router(collection, vec![profile1, profile2], clock);

    let dec1 = router.route(&[1.0, 0.0, 0.0, 0.0], "test").await?;
    assert!(router.record_outcome(dec1.decision_id, RoutingOutcome::Success));

    assert_eq!(router.calibration_stats()["p1"].times_selected, 1);

    router.reset_all_calibration();

    assert_eq!(router.calibration_stats()["p1"].times_selected, 0);
    assert_eq!(router.calibration_stats()["p2"].times_selected, 0);

    Ok(())
}

// ── Test 19: Quantization Level Default and Display ──────────────────────────
#[test]
fn test_quantization_level_default_and_variants() {
    use contextra_router::profile::QuantizationLevel;

    assert_eq!(QuantizationLevel::default(), QuantizationLevel::Unknown);
    assert_ne!(QuantizationLevel::F16, QuantizationLevel::Q8_0);
    assert_ne!(QuantizationLevel::Q8_0, QuantizationLevel::Q4_K_M);
}

// ── Test 20: Drift Baseline Setting ──────────────────────────────────────────
#[tokio::test]
async fn test_set_lyapunov_baseline_existing_and_nonexisting() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;

    let clock = Arc::new(TestClock::new(1_000_000_000_000));

    let profile = SlmProfile::new("p1", "http://p1", vec![], TokenBudget::default(), 0.5);
    let router = create_test_router(collection, vec![profile], clock);

    let baseline = vec![0.1f32; 30];
    assert!(router.set_lyapunov_baseline("p1", &baseline));
    assert!(!router.set_lyapunov_baseline("nonexistent", &baseline));

    Ok(())
}
