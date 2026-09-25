#[path = "../src/check_mutation_score_gate.rs"]
mod check_mutation_score_gate;

use check_mutation_score_gate::{
    check_gate, get_mutation_threshold_for_crate, MutationGateError,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_get_mutation_threshold_for_crate() {
    assert_eq!(
        get_mutation_threshold_for_crate("contextra-crypto"),
        Some(70.0)
    );
    assert_eq!(
        get_mutation_threshold_for_crate("contextra-store"),
        Some(70.0)
    );
    assert_eq!(
        get_mutation_threshold_for_crate("contextra-checkpoint"),
        Some(70.0)
    );
    assert_eq!(get_mutation_threshold_for_crate("contextra-graph"), None);
    assert_eq!(get_mutation_threshold_for_crate("contextra-vector"), None);
}

#[test]
fn test_check_gate_no_history_found() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let res = check_gate("contextra-store", root);
    assert!(matches!(res, Err(MutationGateError::NoHistoryFound(k)) if k == "contextra-store"));
}

#[test]
fn test_check_gate_threshold_not_met() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let docs_dir = root.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    let history_file = docs_dir.join("mutation_score_history.jsonl");

    let line = r#"{"date":"2026-09-25","commit":"abc1234","crate":"contextra-store","total_mutants":100,"caught":50,"missed":50,"score_pct":50.0}"#;
    fs::write(&history_file, format!("{}\n", line)).unwrap();

    let res = check_gate("contextra-store", root);
    match res {
        Err(MutationGateError::ThresholdNotMet {
            crate_name,
            actual_pct,
            required_pct,
            caught,
            total,
        }) => {
            assert_eq!(crate_name, "contextra-store");
            assert_eq!(actual_pct, 50.0);
            assert_eq!(required_pct, 70.0);
            assert_eq!(caught, 50);
            assert_eq!(total, 100);
        }
        other => panic!("Expected ThresholdNotMet error, got {:?}", other),
    }
}

#[test]
fn test_check_gate_threshold_met_success() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let docs_dir = root.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    let history_file = docs_dir.join("mutation_score_history.jsonl");

    let line = r#"{"date":"2026-09-25","commit":"abc1234","crate":"contextra-store","total_mutants":100,"caught":80,"missed":20,"score_pct":80.0}"#;
    fs::write(&history_file, format!("{}\n", line)).unwrap();

    let res = check_gate("contextra-store", root).unwrap();
    assert_eq!(res.crate_name, "contextra-store");
    assert_eq!(res.total_mutants, 100);
    assert_eq!(res.caught_mutants, 80);
    assert_eq!(res.score_pct, 80.0);
    assert_eq!(res.required_threshold_pct, Some(70.0));
    assert!(!res.informational);
}

#[test]
fn test_check_gate_informational_crate() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let docs_dir = root.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    let history_file = docs_dir.join("mutation_score_history.jsonl");

    let line = r#"{"date":"2026-09-25","commit":"abc1234","crate":"contextra-graph","total_mutants":100,"caught":40,"missed":60,"score_pct":40.0}"#;
    fs::write(&history_file, format!("{}\n", line)).unwrap();

    let res = check_gate("contextra-graph", root).unwrap();
    assert_eq!(res.crate_name, "contextra-graph");
    assert_eq!(res.score_pct, 40.0);
    assert_eq!(res.required_threshold_pct, None);
    assert!(res.informational);
}
