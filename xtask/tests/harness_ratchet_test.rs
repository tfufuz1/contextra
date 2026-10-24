//! Integration test for ratchet harness module.

#[path = "../src/harness/ratchet.rs"]
mod ratchet;

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn setup_temp_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let path = temp_dir.path();

    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(path)
            .args(args)
            .status()
            .expect("failed to execute git command");
        assert!(status.success());
    };

    run_git(&["init"]);
    run_git(&["config", "user.name", "Test User"]);
    run_git(&["config", "user.email", "test@example.com"]);

    fs::create_dir_all(path.join("governance")).unwrap();
    fs::create_dir_all(path.join("docs/decisions")).unwrap();
    fs::create_dir_all(path.join(".github")).unwrap();
    fs::create_dir_all(path.join("crates/contextra-crypto/src")).unwrap();

    fs::write(
        path.join("crates/contextra-crypto/src/lib.rs"),
        "#![forbid(unsafe_code)]\npub fn test_unwrap() { let x: Option<i32> = None; let _ = x.unwrap(); }\n",
    )
    .unwrap();

    fs::write(
        path.join("docs/decisions/ADR-003-test.md"),
        "# ADR 003\nBaseline increase approved\n",
    )
    .unwrap();

    run_git(&["add", "."]);
    run_git(&["commit", "-m", "Initial commit"]);

    temp_dir
}

#[test]
fn test_ratchet_init_and_check() {
    let temp_dir = setup_temp_repo();
    let root = temp_dir.path().to_str().unwrap();

    // 1. Initialize ratchet
    let code = ratchet::run_ratchet(&[
        "update".to_string(),
        "--root".to_string(),
        root.to_string(),
        "--init".to_string(),
        "--json".to_string(),
    ]);
    assert_eq!(code, 0);

    // 2. Check should pass
    let code_check = ratchet::run_ratchet(&[
        "check".to_string(),
        "--root".to_string(),
        root.to_string(),
        "--json".to_string(),
    ]);
    assert_eq!(code_check, 0);
}

#[test]
fn test_ratchet_check_fails_on_increased_unwrap() {
    let temp_dir = setup_temp_repo();
    let root = temp_dir.path().to_str().unwrap();

    // Initialize
    let code = ratchet::run_ratchet(&[
        "update".to_string(),
        "--root".to_string(),
        root.to_string(),
        "--init".to_string(),
    ]);
    assert_eq!(code, 0);

    // Add another unwrap
    fs::write(
        temp_dir.path().join("crates/contextra-crypto/src/lib.rs"),
        "#![forbid(unsafe_code)]\npub fn test_unwrap() { let x: Option<i32> = None; let _ = x.unwrap(); let _ = x.expect(\"err\"); }\n",
    )
    .unwrap();

    // Check should fail
    let code_check =
        ratchet::run_ratchet(&["check".to_string(), "--root".to_string(), root.to_string()]);
    assert_eq!(code_check, 1);
}

#[test]
fn test_ratchet_update_increase_rejected_without_adr() {
    let temp_dir = setup_temp_repo();
    let root = temp_dir.path().to_str().unwrap();

    // Initialize
    let code = ratchet::run_ratchet(&[
        "update".to_string(),
        "--root".to_string(),
        root.to_string(),
        "--init".to_string(),
    ]);
    assert_eq!(code, 0);

    // Add another unwrap
    fs::write(
        temp_dir.path().join("crates/contextra-crypto/src/lib.rs"),
        "#![forbid(unsafe_code)]\npub fn test_unwrap() { let x: Option<i32> = None; let _ = x.unwrap(); let _ = x.expect(\"err\"); }\n",
    )
    .unwrap();

    // Update without ADR should return 2 (Exit 2)
    let code_update =
        ratchet::run_ratchet(&["update".to_string(), "--root".to_string(), root.to_string()]);
    assert_eq!(code_update, 2);
}

#[test]
fn test_ratchet_update_increase_allowed_with_adr() {
    let temp_dir = setup_temp_repo();
    let root = temp_dir.path().to_str().unwrap();

    // Initialize
    let code = ratchet::run_ratchet(&[
        "update".to_string(),
        "--root".to_string(),
        root.to_string(),
        "--init".to_string(),
    ]);
    assert_eq!(code, 0);

    // Add another unwrap
    fs::write(
        temp_dir.path().join("crates/contextra-crypto/src/lib.rs"),
        "#![forbid(unsafe_code)]\npub fn test_unwrap() { let x: Option<i32> = None; let _ = x.unwrap(); let _ = x.expect(\"err\"); }\n",
    )
    .unwrap();

    // Update with ADR should pass
    let code_update = ratchet::run_ratchet(&[
        "update".to_string(),
        "--root".to_string(),
        root.to_string(),
        "--adr".to_string(),
        "ADR-003".to_string(),
    ]);
    assert_eq!(code_update, 0);
}
