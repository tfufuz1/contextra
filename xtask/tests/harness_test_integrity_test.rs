#[path = "../src/harness/test_integrity.rs"]
mod test_integrity;

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

    fs::create_dir_all(p.join("tests")).unwrap();
    fs::write(p.join("tests/my_test.rs"), "#[test]\nfn test_valid() { assert_eq!(1, 1); }\n").unwrap();
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
fn test_test_integrity_valid_test() {
    let dir = setup_temp_repo();
    let p = dir.path();

    fs::write(
        p.join("tests/my_test.rs"),
        "#[test]\nfn test_valid() { assert_eq!(1 + 1, 2); }\n",
    )
    .unwrap();
    Command::new("git")
        .args(["commit", "-am", "valid test change"])
        .current_dir(p)
        .output()
        .unwrap();

    let root_arg = format!("{}", p.display());
    let args = vec![
        "--root".to_string(),
        root_arg,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
    ];

    let code = test_integrity::run_test_integrity(&args);
    assert_eq!(code, 0);
}

#[test]
fn test_test_integrity_no_assertion_and_assert_true() {
    let dir = setup_temp_repo();
    let p = dir.path();

    fs::write(
        p.join("tests/my_test.rs"),
        "#[test]\nfn test_no_assert() { let _x = 1; }\n#[test]\nfn test_tautology() { assert!(true); }\n",
    )
    .unwrap();
    Command::new("git")
        .args(["commit", "-am", "bad tests"])
        .current_dir(p)
        .output()
        .unwrap();

    let root_arg = format!("{}", p.display());
    let args = vec![
        "--root".to_string(),
        root_arg,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--json".to_string(),
    ];

    let code = test_integrity::run_test_integrity(&args);
    assert_eq!(code, 1);
}

#[test]
fn test_test_integrity_ignore_and_deleted_file() {
    let dir = setup_temp_repo();
    let p = dir.path();

    fs::remove_file(p.join("tests/my_test.rs")).unwrap();
    fs::write(
        p.join("tests/ignored_test.rs"),
        "#[test]\n#[ignore]\nfn test_ignored() { assert!(1 == 1); }\n",
    )
    .unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(p)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "deleted test and added ignored test"])
        .current_dir(p)
        .output()
        .unwrap();

    let root_arg = format!("{}", p.display());
    let args = vec![
        "--root".to_string(),
        root_arg,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
    ];

    let code = test_integrity::run_test_integrity(&args);
    assert_eq!(code, 1);
}

#[test]
fn test_test_integrity_insta_bless_in_script() {
    let dir = setup_temp_repo();
    let p = dir.path();

    fs::write(p.join("bless.sh"), "#!/bin/bash\ncargo insta accept --bless\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(p)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "add bless script"])
        .current_dir(p)
        .output()
        .unwrap();

    let root_arg = format!("{}", p.display());
    let args = vec![
        "--root".to_string(),
        root_arg,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
    ];

    let code = test_integrity::run_test_integrity(&args);
    assert_eq!(code, 1);
}
