use contextra_adapt::rie_greedy::RieGreedyProfile;

#[test]
fn test_rie_greedy_profile_uncertainty_and_confidence_score_wire_trace_precision_inv() {
    let mut profile = RieGreedyProfile::new(4, 1.0, 0.9);

    // Initial: Lambda = I_4, Tr(Lambda^-1) = 4.0
    let trace = profile.trace_precision_inv();
    assert_eq!(trace, 4.0);

    let unc = profile.uncertainty();
    assert_eq!(unc, 1.0); // 4.0 / 4

    let conf = profile.confidence_score();
    assert!((conf - 0.2).abs() < 1e-6); // 1.0 / (1.0 + 4.0) = 0.2

    // Perform update
    let ctx = vec![1.0f32, 0.0, 0.0, 0.0];
    profile.update(&ctx, 1.0).expect("update succeeded");

    // Trace should change and uncertainty/confidence_score reflect updated trace
    let unc_after = profile.uncertainty();
    let conf_after = profile.confidence_score();
    assert!(unc_after.is_finite());
    assert!(conf_after > 0.0 && conf_after <= 1.0);
}
