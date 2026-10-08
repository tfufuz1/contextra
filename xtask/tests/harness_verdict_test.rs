#[path = "../src/harness/verdict.rs"]
mod verdict;

use std::fs;
use tempfile::tempdir;
use verdict::{normalize_gate_name, run_verdict};

fn setup_test_env(
    required_toml_content: &str,
    gate_jsons: &[(&str, &str)],
) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let req_path = temp.path().join("verdict-required.toml");
    fs::write(&req_path, required_toml_content).unwrap();

    let results_dir = temp.path().join("gate-results");
    fs::create_dir_all(&results_dir).unwrap();

    for (name, json_content) in gate_jsons {
        let json_path = results_dir.join(format!("{}.json", name));
        fs::write(json_path, json_content).unwrap();
    }

    (temp, req_path, results_dir)
}

#[test]
fn test_normalize_gate_name_unit() {
    assert_eq!(
        normalize_gate_name("gate-determinism-check"),
        "determinism-check"
    );
    assert_eq!(
        normalize_gate_name("determinism-check"),
        "determinism-check"
    );
    assert_eq!(normalize_gate_name("gate-gate-weakening"), "gate-weakening");
}

#[test]
fn test_verdict_gate_prefix_normalization_matching() {
    let toml = r#"
[[gate]]
name = "gate-determinism-check"
blocking = true

[[gate]]
name = "unsafe-audit"
blocking = true
"#;

    let g1 = r#"{
        "gate": "determinism-check",
        "status": "pass",
        "summary": "Determinism check passed",
        "findings": []
    }"#;

    let g2 = r#"{
        "gate": "gate-unsafe-audit",
        "status": "pass",
        "summary": "Unsafe audit passed",
        "findings": []
    }"#;

    // g1 artifact saved as "determinism-check.json", g2 saved as "gate-unsafe-audit.json"
    let (_temp, req_path, results_dir) = setup_test_env(
        toml,
        &[("determinism-check", g1), ("gate-unsafe-audit", g2)],
    );

    let code = run_verdict(&[
        "--required".to_string(),
        req_path.to_str().unwrap().to_string(),
        "--results-dir".to_string(),
        results_dir.to_str().unwrap().to_string(),
    ]);

    assert_eq!(code, 0);
}

#[test]
fn test_verdict_all_pass() {
    let toml = r#"
[[gate]]
name = "protected-paths"
blocking = true

[[gate]]
name = "doc-truth"
blocking = true
"#;

    let g1 = r#"{
        "gate": "protected-paths",
        "status": "pass",
        "summary": "Protected paths clean",
        "findings": []
    }"#;

    let g2 = r#"{
        "gate": "doc-truth",
        "status": "not_applicable",
        "summary": "No doc changes",
        "findings": []
    }"#;

    let (_temp, req_path, results_dir) =
        setup_test_env(toml, &[("protected-paths", g1), ("doc-truth", g2)]);

    let code = run_verdict(&[
        "--required".to_string(),
        req_path.to_str().unwrap().to_string(),
        "--results-dir".to_string(),
        results_dir.to_str().unwrap().to_string(),
    ]);

    assert_eq!(code, 0);
}

#[test]
fn test_verdict_single_gate_fail() {
    let toml = r#"
[[gate]]
name = "protected-paths"
blocking = true
"#;

    let g1 = r#"{
        "gate": "protected-paths",
        "status": "fail",
        "summary": "Protected path touched",
        "findings": [
            {
                "id": "PROT-01",
                "severity": "error",
                "file": "CODEOWNERS",
                "line": 10,
                "message": "CODEOWNERS modified without approval",
                "fix": "Revert CODEOWNERS"
            }
        ]
    }"#;

    let (_temp, req_path, results_dir) = setup_test_env(toml, &[("protected-paths", g1)]);

    let code = run_verdict(&[
        "--required".to_string(),
        req_path.to_str().unwrap().to_string(),
        "--results-dir".to_string(),
        results_dir.to_str().unwrap().to_string(),
    ]);

    assert_eq!(code, 1);
}

#[test]
fn test_verdict_missing_artifact() {
    let toml = r#"
[[gate]]
name = "protected-paths"
blocking = true
"#;

    let (_temp, req_path, results_dir) = setup_test_env(toml, &[]);

    let code = run_verdict(&[
        "--required".to_string(),
        req_path.to_str().unwrap().to_string(),
        "--results-dir".to_string(),
        results_dir.to_str().unwrap().to_string(),
    ]);

    assert_eq!(code, 1);
}

#[test]
fn test_verdict_invalid_json() {
    let toml = r#"
[[gate]]
name = "protected-paths"
blocking = true
"#;

    let (_temp, req_path, results_dir) = setup_test_env(toml, &[("protected-paths", "not json")]);

    let code = run_verdict(&[
        "--required".to_string(),
        req_path.to_str().unwrap().to_string(),
        "--results-dir".to_string(),
        results_dir.to_str().unwrap().to_string(),
    ]);

    assert_eq!(code, 1);
}

#[test]
fn test_verdict_name_mismatch() {
    let toml = r#"
[[gate]]
name = "protected-paths"
blocking = true
"#;

    let g1 = r#"{
        "gate": "wrong-gate-name",
        "status": "pass",
        "summary": "Clean",
        "findings": []
    }"#;

    let (_temp, req_path, results_dir) = setup_test_env(toml, &[("protected-paths", g1)]);

    let code = run_verdict(&[
        "--required".to_string(),
        req_path.to_str().unwrap().to_string(),
        "--results-dir".to_string(),
        results_dir.to_str().unwrap().to_string(),
    ]);

    assert_eq!(code, 1);
}

#[test]
fn test_verdict_non_blocking_fail_returns_green() {
    let toml = r#"
[[gate]]
name = "fuzz-smoke"
blocking = false
"#;

    let g1 = r#"{
        "gate": "fuzz-smoke",
        "status": "fail",
        "summary": "Fuzzing found crash",
        "findings": []
    }"#;

    let (_temp, req_path, results_dir) = setup_test_env(toml, &[("fuzz-smoke", g1)]);

    let code = run_verdict(&[
        "--required".to_string(),
        req_path.to_str().unwrap().to_string(),
        "--results-dir".to_string(),
        results_dir.to_str().unwrap().to_string(),
    ]);

    assert_eq!(code, 0);
}

#[test]
fn test_verdict_json_flag_output() {
    let toml = r#"
[[gate]]
name = "protected-paths"
blocking = true
"#;

    let g1 = r#"{
        "gate": "protected-paths",
        "status": "pass",
        "summary": "Clean",
        "findings": []
    }"#;

    let (_temp, req_path, results_dir) = setup_test_env(toml, &[("protected-paths", g1)]);

    let code = run_verdict(&[
        "--required".to_string(),
        req_path.to_str().unwrap().to_string(),
        "--results-dir".to_string(),
        results_dir.to_str().unwrap().to_string(),
        "--json".to_string(),
    ]);

    assert_eq!(code, 0);
}

#[test]
fn test_verdict_max_three_reasons_deterministic() {
    let toml = r#"
[[gate]]
name = "gate-a"
blocking = true

[[gate]]
name = "gate-b"
blocking = true

[[gate]]
name = "gate-c"
blocking = true

[[gate]]
name = "gate-d"
blocking = true
"#;

    let fail_json = |name: &str| {
        format!(
            r#"{{
            "gate": "{}",
            "status": "fail",
            "summary": "Failed {}",
            "findings": [
                {{
                    "id": "ERR-01",
                    "severity": "error",
                    "file": "file.rs",
                    "line": 1,
                    "message": "Error in {}",
                    "fix": "Fix it"
                }}
            ]
        }}"#,
            name, name, name
        )
    };

    let (_temp, req_path, results_dir) = setup_test_env(
        toml,
        &[
            ("gate-a", &fail_json("gate-a")),
            ("gate-b", &fail_json("gate-b")),
            ("gate-c", &fail_json("gate-c")),
            ("gate-d", &fail_json("gate-d")),
        ],
    );

    let code = run_verdict(&[
        "--required".to_string(),
        req_path.to_str().unwrap().to_string(),
        "--results-dir".to_string(),
        results_dir.to_str().unwrap().to_string(),
    ]);

    assert_eq!(code, 1);
}
