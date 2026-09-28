// xtask/tests/harness_dependency_gate_test.rs

#[path = "../src/harness/dependency_gate.rs"]
mod dependency_gate;

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
fn test_dependency_gate_no_changes() {
    let root = get_repo_root();
    let root_str = root.to_string_lossy();
    let args = vec![
        "dependency-gate".to_string(),
        "--root".to_string(),
        root_str.to_string(),
        "--json".to_string(),
    ];
    let exit = dependency_gate::run_dependency_gate(&args);
    assert_eq!(exit, 0);
}
