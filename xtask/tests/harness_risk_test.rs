// xtask/tests/harness_risk_test.rs

#[path = "../src/harness/risk.rs"]
mod risk;

fn get_repo_root() -> std::path::PathBuf {
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return std::path::PathBuf::from(path_str);
            }
        }
    }
    std::path::PathBuf::from("..")
}

#[test]
fn test_risk_fixed_score() {
    let root = get_repo_root();
    let root_str = root.to_string_lossy();
    let args = vec![
        "risk".to_string(),
        "--root".to_string(),
        root_str.to_string(),
        "--json".to_string(),
    ];
    let exit = risk::run_risk(&args);
    assert_eq!(exit, 0);
}
