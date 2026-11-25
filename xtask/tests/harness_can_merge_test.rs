// xtask/tests/harness_can_merge_test.rs

#[path = "../src/harness/can_merge.rs"]
mod can_merge;

#[test]
fn test_can_merge_pass() {
    let temp_dir = tempfile::tempdir().unwrap();
    let res_dir = temp_dir.path().join("results");
    std::fs::create_dir_all(&res_dir).unwrap();

    let json_content = r#"{
        "gate": "protected-paths",
        "status": "pass",
        "summary": "ok",
        "findings": []
    }"#;
    std::fs::write(res_dir.join("protected-paths.json"), json_content).unwrap();

    let args = vec![
        "can-merge".to_string(),
        "--results-dir".to_string(),
        res_dir.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    let exit = can_merge::run_can_merge(&args);
    assert_eq!(exit, 0);
}

#[test]
fn test_can_merge_fail_max_3_reasons() {
    let temp_dir = tempfile::tempdir().unwrap();
    let res_dir = temp_dir.path().join("results");
    std::fs::create_dir_all(&res_dir).unwrap();

    for i in 1..=5 {
        let json_content = format!(
            r#"{{
                "gate": "gate-{i}",
                "status": "fail",
                "summary": "error in gate {i}",
                "findings": []
            }}"#
        );
        std::fs::write(res_dir.join(format!("gate-{i}.json")), json_content).unwrap();
    }

    let args = vec![
        "can-merge".to_string(),
        "--results-dir".to_string(),
        res_dir.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    let exit = can_merge::run_can_merge(&args);
    assert_eq!(exit, 1);
}

#[test]
fn test_can_merge_invalid_args_fail_closed() {
    let args = vec![
        "can-merge".to_string(),
        "--results-dir".to_string(),
        "/nonexistent_dir_path_12345".to_string(),
        "--json".to_string(),
    ];

    let exit = can_merge::run_can_merge(&args);
    assert_eq!(exit, 2);
}
