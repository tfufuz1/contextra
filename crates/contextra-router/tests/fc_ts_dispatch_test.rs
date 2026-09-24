#![cfg(feature = "flow-corrected-thompson")]

use contextra_adapt::{FcTsArmSet, FcTsConfig, FlowCorrectedThompsonBandit};
use contextra_router::{deterministic_fc_ts_rng, select_profile_fc_ts, FcTsDispatchError};

#[test]
fn test_fc_ts_dispatch_determinism() {
    let cfg = FcTsConfig {
        dim: 4,
        ..Default::default()
    };
    let arm1 = FlowCorrectedThompsonBandit::new(cfg.clone()).expect("valid config");
    let arm2 = FlowCorrectedThompsonBandit::new(cfg).expect("valid config");
    let arm_set = FcTsArmSet {
        arms: vec![arm1, arm2],
    };

    let profile_names = vec!["profile_a".to_string(), "profile_b".to_string()];
    let context = vec![1.0, 0.5, -0.2, 0.8];

    let seed = 123456789;
    let mut rng1 = deterministic_fc_ts_rng(seed);
    let (idx1, name1) = select_profile_fc_ts(&profile_names, &arm_set, &context, &mut rng1)
        .expect("select_profile_fc_ts should succeed");

    let mut rng2 = deterministic_fc_ts_rng(seed);
    let (idx2, name2) = select_profile_fc_ts(&profile_names, &arm_set, &context, &mut rng2)
        .expect("select_profile_fc_ts should succeed");

    assert_eq!(idx1, idx2);
    assert_eq!(name1, name2);
    assert_eq!(name1, profile_names[idx1]);
}

#[test]
fn test_fc_ts_dispatch_seed_variation() {
    let cfg = FcTsConfig {
        dim: 4,
        ..Default::default()
    };
    let arm1 = FlowCorrectedThompsonBandit::new(cfg.clone()).expect("valid config");
    let arm2 = FlowCorrectedThompsonBandit::new(cfg).expect("valid config");
    let arm_set = FcTsArmSet {
        arms: vec![arm1, arm2],
    };

    let profile_names = vec!["profile_a".to_string(), "profile_b".to_string()];
    let context = vec![0.1, 0.2, 0.3, 0.4];

    for seed in 0..20 {
        let mut rng = deterministic_fc_ts_rng(seed);
        let res = select_profile_fc_ts(&profile_names, &arm_set, &context, &mut rng);
        assert!(res.is_ok());
        let (idx, name) = res.unwrap();
        assert!(idx < profile_names.len());
        assert_eq!(name, profile_names[idx]);
    }
}

#[test]
fn test_fc_ts_dispatch_profile_arm_mismatch() {
    let cfg = FcTsConfig {
        dim: 2,
        ..Default::default()
    };
    let arm1 = FlowCorrectedThompsonBandit::new(cfg).expect("valid config");
    let arm_set = FcTsArmSet { arms: vec![arm1] };

    let profile_names = vec!["profile_a".to_string(), "profile_b".to_string()];
    let context = vec![0.5, 0.5];
    let mut rng = deterministic_fc_ts_rng(42);

    let err = select_profile_fc_ts(&profile_names, &arm_set, &context, &mut rng)
        .expect_err("should fail due to profile/arm length mismatch");

    match err {
        FcTsDispatchError::ProfileArmMismatch { profiles, arms } => {
            assert_eq!(profiles, 2);
            assert_eq!(arms, 1);
        }
        _ => panic!("unexpected error type: {err:?}"),
    }
}

#[test]
fn test_fc_ts_dispatch_empty_arm_set() {
    let arm_set = FcTsArmSet { arms: vec![] };
    let profile_names: Vec<String> = vec![];
    let context = vec![1.0, 2.0];
    let mut rng = deterministic_fc_ts_rng(42);

    let err = select_profile_fc_ts(&profile_names, &arm_set, &context, &mut rng)
        .expect_err("should fail due to empty arm set");

    match err {
        FcTsDispatchError::ArmSet(contextra_adapt::FcTsError::InvalidConfig(msg)) => {
            assert!(msg.contains("empty"));
        }
        _ => panic!("unexpected error type: {err:?}"),
    }
}
