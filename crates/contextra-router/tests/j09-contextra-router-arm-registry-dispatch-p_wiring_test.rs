use contextra_router::profile::{ConformalCalibrator, ProfileCalibrationState};
use contextra_router::{dispatch_to_slm, ArmRegistry, DecisionId, RoutingDecision, SlmProfile};
use contextra_types::{ContextWindow, RetrievalStrategy, TokenBudget};

#[tokio::test]
async fn test_wiring_dispatch_to_slm() {
    let profile = SlmProfile::new(
        "test-slm",
        "/nonexistent/binary/path/for/test",
        vec![],
        TokenBudget::default(),
        0.5,
    );
    let decision = RoutingDecision {
        profile,
        context: ContextWindow {
            chunks: vec![],
            total_tokens: 0,
            truncated: false,
        },
        confidence: None,
        decision_id: DecisionId::from_raw(42),
        drift_status: None,
    };

    // Test dispatch_to_slm function from dispatch.rs
    let result = dispatch_to_slm(&decision).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Fehler bei MCP-Dispatch"));
}

#[test]
fn test_wiring_arm_registry_symbols() {
    let registry = ArmRegistry::default();

    // Test arm_for
    let arm_0 = registry.arm_for(RetrievalStrategy::Vector);
    assert_eq!(arm_0, 0);

    let arm_4 = registry.arm_for(RetrievalStrategy::Global {
        max_community_nodes: None,
        min_community_size: None,
    });
    assert_eq!(arm_4, 4);

    // Test strategy_for
    let strat_0 = registry.strategy_for(0).unwrap();
    assert_eq!(strat_0, RetrievalStrategy::Vector);

    let strat_4 = registry.strategy_for(4).unwrap();
    match strat_4 {
        RetrievalStrategy::Global { .. } => {}
        _ => panic!("Expected Global strategy for arm 4"),
    }

    assert!(registry.strategy_for(99).is_err());
}

#[test]
fn test_wiring_profile_calibration_and_cost_symbols() {
    // Test with_resource_cost_estimate
    let profile = SlmProfile::new(
        "slm-costed",
        "http://localhost:8000/mcp",
        vec![],
        TokenBudget::new(500, 50),
        0.5,
    )
    .with_resource_cost_estimate(25.5);

    assert_eq!(profile.resource_cost_estimate, 25.5);
    assert_eq!(profile.estimated_cost(), 25.5);

    // Test average_confidence
    let mut cal_state = ProfileCalibrationState::new(0.5);
    assert_eq!(cal_state.average_confidence(), 1.0);

    cal_state.times_selected = 2;
    cal_state.cumulative_confidence = 3.0;
    assert_eq!(cal_state.average_confidence(), 1.5);

    // Test empirical_error_rate
    let mut calibrator = ConformalCalibrator::new(0.05, 0.01, 0.5);
    assert_eq!(calibrator.empirical_error_rate(), 0.0);

    calibrator.update(0.8); // error
    calibrator.update(0.2); // non-error
    assert_eq!(calibrator.empirical_error_rate(), 0.5);

    // Test reset_window
    calibrator.reset_window();
    assert_eq!(calibrator.empirical_error_rate(), 0.0);
}
