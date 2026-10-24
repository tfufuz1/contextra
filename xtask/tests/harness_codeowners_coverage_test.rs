//! Integration tests for codeowners-coverage harness command.

#[path = "../src/harness/codeowners_coverage.rs"]
mod codeowners_coverage;

use std::fs;
use tempfile::tempdir;

#[test]
fn test_codeowners_coverage_phantom_path() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    fs::create_dir_all(root.join("governance")).expect("governance dir");

    fs::write(
        root.join("governance/protected-paths.toml"),
        r#"
[[protected]]
glob = "justfile"
reason = "Zentraler Task-Runner"
"#,
    )
    .expect("write protected-paths.toml");

    fs::write(root.join("justfile"), "# justfile").expect("write justfile");

    fs::write(
        root.join("CODEOWNERS"),
        r#"
/justfile @contextra/build-infra
/nonexistent-file.txt @contextra/build-infra
"#,
    )
    .expect("write CODEOWNERS");

    let root_str = root.to_string_lossy().to_string();
    let args = vec!["--root".to_string(), root_str];
    let exit_code = codeowners_coverage::run_codeowners_coverage(&args);
    assert_eq!(
        exit_code, 1,
        "codeowners-coverage should fail on phantom path"
    );
}

#[test]
fn test_codeowners_coverage_missing_glob() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    fs::create_dir_all(root.join("governance")).expect("governance dir");

    fs::write(
        root.join("governance/protected-paths.toml"),
        r#"
[[protected]]
glob = "justfile"
reason = "Zentraler Task-Runner"

[[protected]]
glob = "deny.toml"
reason = "Cargo Deny"
"#,
    )
    .expect("write protected-paths.toml");

    fs::write(root.join("justfile"), "# justfile").expect("write justfile");
    fs::write(root.join("deny.toml"), "# deny.toml").expect("write deny.toml");

    fs::write(
        root.join("CODEOWNERS"),
        r#"
/justfile @contextra/build-infra
"#,
    )
    .expect("write CODEOWNERS");

    let root_str = root.to_string_lossy().to_string();
    let args = vec!["--root".to_string(), root_str];
    let exit_code = codeowners_coverage::run_codeowners_coverage(&args);
    assert_eq!(
        exit_code, 1,
        "codeowners-coverage should fail on unmapped protected path"
    );
}
