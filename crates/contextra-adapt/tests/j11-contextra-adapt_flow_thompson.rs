use contextra_adapt::flow_thompson::{
    FcTsArmSet, FcTsConfig, FlowCorrectedThompsonBandit, SplitMix64,
};

#[test]
fn test_fcts_arm_set_update_arm_and_background_drift_recomputation() {
    let cfg = FcTsConfig {
        dim: 4,
        window_capacity: 10,
        ..Default::default()
    };

    let arm0 = FlowCorrectedThompsonBandit::new(cfg.clone()).unwrap();
    let arm1 = FlowCorrectedThompsonBandit::new(cfg).unwrap();

    let mut arm_set = FcTsArmSet {
        arms: vec![arm0, arm1],
    };

    let mut rng = SplitMix64::new(42);
    let ctx = vec![1.0f32, 0.5f32, -0.2f32, 0.8f32];

    // Select an arm
    let selected = arm_set
        .select_arm(&ctx, &mut rng)
        .expect("select_arm succeeded");
    assert!(selected < 2);

    // Update the selected arm via FcTsArmSet::update_arm (which calls update_with_flow)
    arm_set
        .update_arm(selected as usize, &ctx, 1.0, 10, 1.0)
        .expect("update_arm succeeded");

    // Trigger background drift rate recomputation (which calls ring3_background_task_token)
    arm_set.recompute_drift_rates_in_background();

    // Verify individual arm background recomputation
    arm_set.arms[0].recompute_drift_rate_in_background();
}
