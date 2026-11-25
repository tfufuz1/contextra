// xtask/tests/harness_flake_report_test.rs

#[path = "../src/harness/flake_report.rs"]
mod flake_report;

#[test]
fn test_flake_report_missing_nextest() {
    let args = vec![
        "flake-report".to_string(),
        "--runs".to_string(),
        "1".to_string(),
        "--json".to_string(),
    ];

    // Assuming environment or mock behavior
    let exit = flake_report::run_flake_report(&args);
    // Either 0 or 2 depending on whether nextest is installed in host or mock
    assert!(exit == 0 || exit == 2);
}
