use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_router::profile::{ConformalCalibrator, ProfileCalibrationState};
use contextra_router::router::ConfidenceMetrics;
use contextra_router::{
    dispatch_to_slm, route_and_dispatch_to_slm, ArmRegistry, DecisionId, RouterEngine,
    RoutingDecision, SlmProfile,
};
use contextra_types::{
    ContextChunk, ContextWindow, DocId, EntityId, Result, RetrievalStrategy, TokenBudget,
};
use std::collections::HashSet;
use std::sync::Arc;

struct MockSearch;
impl HybridSearchProvider for MockSearch {
    fn search_hybrid<'a>(
        &'a self,
        _query_text: &'a str,
        _query_embedding: &'a [f32],
        _k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        Box::pin(async move {
            Ok(vec![ContextChunk {
                doc_id: DocId::new(1),
                content: "mock content".into(),
                relevance: 0.9,
                token_count: 2,
                metadata: None,
                contextual_prefix: None,
                links: vec![],
            }])
        })
    }
}

struct MockCommunity;
impl CommunityResolver for MockCommunity {
    fn get_community<'a>(&'a self, _entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        Box::pin(async move { Ok(Some(1)) })
    }
}

struct MockPreparer;
impl ContextPreparer for MockPreparer {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        _budget: &TokenBudget,
        _relevance_threshold: f32,
    ) -> Result<ContextWindow> {
        let total_tokens = chunks.iter().map(|c| c.token_count).sum();
        Ok(ContextWindow {
            chunks,
            total_tokens,
            truncated: false,
        })
    }
}

#[test]
fn test_closure_arm_registry_wiring() {
    let registry = ArmRegistry::default();

    // 1. arm_for & strategy_for
    let arm_idx = registry.arm_for(RetrievalStrategy::Vector);
    assert_eq!(arm_idx, 0);

    let strategy = registry.strategy_for(0).expect("arm 0 exists");
    assert_eq!(strategy, RetrievalStrategy::Vector);

    // 2. resolve_strategy
    let resolved = registry
        .resolve_strategy(RetrievalStrategy::Hybrid)
        .expect("Hybrid registered");
    assert_eq!(resolved.0, 3);
    assert_eq!(resolved.1, RetrievalStrategy::Hybrid);

    // 3. contains_strategy
    assert!(registry.contains_strategy(RetrievalStrategy::Graph));

    // 4. active_strategies
    let active = registry.active_strategies();
    assert_eq!(active.len(), 5);
    assert_eq!(active[0], (0, RetrievalStrategy::Vector));
    assert_eq!(active[1], (1, RetrievalStrategy::Text));
    assert_eq!(active[2], (2, RetrievalStrategy::Graph));
    assert_eq!(active[3], (3, RetrievalStrategy::Hybrid));
}

#[tokio::test]
async fn test_closure_dispatch_to_slm_wiring() {
    let profile = SlmProfile::new(
        "closure-slm",
        "/invalid/binary/endpoint/path",
        vec![],
        TokenBudget::default(),
        0.5,
    );
    let decision = RoutingDecision {
        profile,
        context: ContextWindow {
            chunks: vec![ContextChunk {
                doc_id: DocId::new(100),
                content: "Closure dispatch test".into(),
                relevance: 0.95,
                token_count: 3,
                metadata: None,
                contextual_prefix: None,
                links: vec![],
            }],
            total_tokens: 3,
            truncated: false,
        },
        confidence: Some(ConfidenceMetrics {
            score_lower: Some(0.90),
            score_upper: Some(0.95),
            calibrated: true,
            quantile_threshold: 0.5,
            non_conformity_score: 0.05,
            selection_margin: 0.1,
        }),
        decision_id: DecisionId::from_raw(999),
        drift_status: None,
    };

    // Test direct dispatch_to_slm
    let dispatch_res = dispatch_to_slm(&decision).await;
    assert!(dispatch_res.is_err());
    let err_msg = dispatch_res.unwrap_err().to_string();
    assert!(
        err_msg.contains("Fehler bei MCP-Dispatch"),
        "Unexpected error msg: {err_msg}"
    );

    // Test route_and_dispatch_to_slm end-to-end call path
    let search = Arc::new(MockSearch);
    let comm = Arc::new(MockCommunity);
    let prep = Arc::new(MockPreparer);
    let slm_prof = SlmProfile::new(
        "mock-slm",
        "/invalid/path/endpoint",
        vec![1],
        TokenBudget::default(),
        0.1,
    );
    let router = RouterEngine::new(search, comm, prep, vec![slm_prof], None);

    let route_dispatch_res = route_and_dispatch_to_slm(&router, &[0.1, 0.2], "hello query").await;
    assert!(route_dispatch_res.is_err());
    assert!(route_dispatch_res
        .unwrap_err()
        .to_string()
        .contains("Fehler bei MCP-Dispatch"));
}

#[test]
fn test_closure_profile_and_calibration_wiring() {
    // 1. with_resource_cost_estimate & try_new_with_cost
    let profile = SlmProfile::try_new_with_cost(
        "costed-slm",
        "http://localhost:9090/mcp",
        HashSet::from([1, 2, 3]),
        TokenBudget::new(1000, 100),
        0.4,
        15.5,
    )
    .expect("valid profile with cost");

    assert_eq!(profile.resource_cost_estimate, 15.5);
    assert_eq!(profile.estimated_cost(), 15.5);

    // 2. ProfileCalibrationState & average_confidence
    let mut cal_state = ProfileCalibrationState::new(0.4);
    assert_eq!(cal_state.average_confidence(), 1.0);
    assert!(cal_state.is_healthy());

    cal_state.times_selected = 4;
    cal_state.cumulative_confidence = 3.6;
    assert_eq!(cal_state.average_confidence(), 0.9);

    // 3. ConformalCalibrator & empirical_error_rate
    let mut calibrator = ConformalCalibrator::new(0.05, 0.01, 0.4);
    assert_eq!(calibrator.empirical_error_rate(), 0.0);

    calibrator.update(0.9); // error score > 0.4
    calibrator.update(0.1); // non-error score <= 0.4
    assert_eq!(calibrator.empirical_error_rate(), 0.5);
    assert_eq!(cal_state.empirical_error_rate(), 0.0);

    // 4. reset_window
    calibrator.reset_window();
    assert_eq!(calibrator.empirical_error_rate(), 0.0);

    cal_state.reset_window();
    assert_eq!(cal_state.empirical_error_rate(), 0.0);
}
