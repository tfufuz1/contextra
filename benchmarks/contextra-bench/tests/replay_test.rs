//! Integration tests for the off-policy replay harness and JSON lines log ingestion.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_bench::replay::{
    load_log_records_from_jsonl, run_replay_evaluation, write_log_records_to_jsonl,
    ReplayLogRecord, ReplayOptions,
};
use tempfile::tempdir;

#[test]
fn test_jsonl_log_roundtrip() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_logs.jsonl");

    let records = vec![
        ReplayLogRecord {
            timestamp: 100,
            context: vec![0.1, 0.2, 0.3, 0.4],
            logged_action: 0,
            propensity: 0.25,
            reward: 0.8,
            oracle_reward: Some(0.9),
            all_propensities: Some(vec![0.25, 0.25, 0.25, 0.25]),
        },
        ReplayLogRecord {
            timestamp: 101,
            context: vec![-0.1, 0.5, 0.0, 0.2],
            logged_action: 1,
            propensity: 0.85,
            reward: 0.5,
            oracle_reward: Some(0.6),
            all_propensities: Some(vec![0.05, 0.85, 0.05, 0.05]),
        },
    ];

    write_log_records_to_jsonl(&records, &file_path).unwrap();
    let loaded = load_log_records_from_jsonl(&file_path).unwrap();

    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].timestamp, 100);
    assert_eq!(loaded[0].logged_action, 0);
    assert!((loaded[0].reward - 0.8).abs() < 1e-6);
    assert_eq!(loaded[1].logged_action, 1);
    assert!((loaded[1].propensity - 0.85).abs() < 1e-6);
}

#[test]
fn test_replay_smoke_evaluation() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().to_str().unwrap().to_string();

    let options = ReplayOptions {
        smoke: true,
        log_file: None,
        output_dir: out_dir,
    };

    let report = run_replay_evaluation(options).unwrap();

    assert!(report.smoke_profile);
    assert_eq!(report.seeds_per_scenario, 5);
    assert_eq!(report.steps_per_seed, 300);
    assert_eq!(report.total_scenarios, 4);
    assert_eq!(report.scenario_results.len(), 4);

    for sc in &report.scenario_results {
        assert!(sc.fc_ts_regret.mean >= 0.0);
        assert!(sc.sm_regret.mean >= 0.0);
        assert!(sc.random_regret.mean >= 0.0);
    }
}
