#[path = "../src/harness/claims_in_pr.rs"]
mod claims_in_pr;

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn setup_temp_repo() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    let p = dir.path();
    Command::new("git")
        .args(["init"])
        .current_dir(p)
        .output()
        .expect("git init failed");
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(p)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(p)
        .output()
        .unwrap();

    fs::write(p.join("file1.rs"), "fn f1() {}\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(p)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(p)
        .output()
        .unwrap();

    dir
}

#[test]
fn test_claims_in_pr_valid() {
    let dir = setup_temp_repo();
    let p = dir.path();

    fs::write(p.join("file1.rs"), "fn f1() { println!(\"1\"); }\n").unwrap();
    Command::new("git")
        .args(["commit", "-am", "update file1"])
        .current_dir(p)
        .output()
        .unwrap();

    let body = r#"
## Ziel
Umsetzung von Feature X.

## Änderungen
- file1.rs

## Invarianten berührt
Keine.

## Verifikation
cargo test: PASS

## Out-of-scope Findings
Keine.

## Risiken/Rollback
Geringes Risiko.

## ADR/Spec-Sync
Nicht erforderlich.
"#;

    let body_path = p.join("body.md");
    fs::write(&body_path, body).unwrap();

    let root_arg = format!("{}", p.display());
    let body_arg = format!("{}", body_path.display());
    let args = vec![
        "--root".to_string(),
        root_arg,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--body-file".to_string(),
        body_arg,
    ];

    let code = claims_in_pr::run_claims_in_pr(&args);
    assert_eq!(code, 0);
}

#[test]
fn test_claims_in_pr_missing_header_and_phantom_claim() {
    let dir = setup_temp_repo();
    let p = dir.path();

    fs::write(p.join("file1.rs"), "fn f1() { // edit\n }\n").unwrap();
    Command::new("git")
        .args(["commit", "-am", "update file1"])
        .current_dir(p)
        .output()
        .unwrap();

    // Body lists phantom file phantom.rs and misses ADR/Spec-Sync header
    let body = r#"
## Ziel
Umsetzung von Feature X.

## Änderungen
- file1.rs
- phantom.rs

## Invarianten berührt
Keine.

## Verifikation
TODO

## Out-of-scope Findings
Keine.

## Risiken/Rollback
Geringes Risiko.
"#;

    let body_path = p.join("body.md");
    fs::write(&body_path, body).unwrap();

    let root_arg = format!("{}", p.display());
    let body_arg = format!("{}", body_path.display());
    let args = vec![
        "--root".to_string(),
        root_arg,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--body-file".to_string(),
        body_arg,
        "--json".to_string(),
    ];

    let code = claims_in_pr::run_claims_in_pr(&args);
    assert_eq!(code, 1);
}

#[test]
fn test_claims_in_pr_tests_added_without_tests() {
    let dir = setup_temp_repo();
    let p = dir.path();

    fs::write(p.join("file1.rs"), "fn f1() { println!(\"2\"); }\n").unwrap();
    Command::new("git")
        .args(["commit", "-am", "update file1 without test"])
        .current_dir(p)
        .output()
        .unwrap();

    let body = r#"
## Ziel
Tests hinzugefügt für Modul.

## Änderungen
- file1.rs

## Invarianten berührt
Keine.

## Verifikation
cargo test: PASS

## Out-of-scope Findings
Keine.

## Risiken/Rollback
Keine.

## ADR/Spec-Sync
Keine.
"#;

    let body_path = p.join("body.md");
    fs::write(&body_path, body).unwrap();

    let root_arg = format!("{}", p.display());
    let body_arg = format!("{}", body_path.display());
    let args = vec![
        "--root".to_string(),
        root_arg,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--body-file".to_string(),
        body_arg,
    ];

    let code = claims_in_pr::run_claims_in_pr(&args);
    assert_eq!(code, 1);
}
