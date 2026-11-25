// xtask/tests/harness_explain_test.rs

#[path = "../src/harness/explain.rs"]
mod explain;

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
fn test_explain_known_gate() {
    let root = get_repo_root();
    let root_str = root.to_string_lossy();
    let args = vec![
        "explain".to_string(),
        "protected-paths".to_string(),
        "--root".to_string(),
        root_str.to_string(),
        "--json".to_string(),
    ];
    let exit = explain::run_explain(&args);
    assert_eq!(exit, 0);
}

#[test]
fn test_explain_list() {
    let root = get_repo_root();
    let root_str = root.to_string_lossy();
    let args = vec![
        "explain".to_string(),
        "--list".to_string(),
        "--root".to_string(),
        root_str.to_string(),
        "--json".to_string(),
    ];
    let exit = explain::run_explain(&args);
    assert_eq!(exit, 0);
}

#[test]
fn test_explain_unknown_gate() {
    let root = get_repo_root();
    let root_str = root.to_string_lossy();
    let args = vec![
        "explain".to_string(),
        "unknown-gate-xyz".to_string(),
        "--root".to_string(),
        root_str.to_string(),
        "--json".to_string(),
    ];
    let exit = explain::run_explain(&args);
    assert_eq!(exit, 1);
}
