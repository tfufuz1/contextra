#[path = "../src/harness/scope_guard.rs"]
mod scope_guard;

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
    fs::create_dir_all(p.join(".jules/tasks")).unwrap();
    fs::write(
        p.join(".jules/tasks/task-01.toml"),
        r#"
id = "task-01"
scope = ["file1.rs", "crates/mycrate/**"]
forbidden = ["crates/forbidden/**"]
"#,
    )
    .unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(p)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Initial commit\n\nTask-Card: task-01"])
        .current_dir(p)
        .output()
        .unwrap();

    dir
}

#[test]
fn test_scope_guard_in_scope() {
    let dir = setup_temp_repo();
    let p = dir.path();

    // Make commit in scope
    fs::write(p.join("file1.rs"), "fn f1() { println!(\"hi\"); }\n").unwrap();
    Command::new("git")
        .args(["commit", "-am", "in scope change"])
        .current_dir(p)
        .output()
        .unwrap();

    let root_arg = format!("{}", p.display());
    let card_arg = format!("{}/.jules/tasks/task-01.toml", p.display());
    let args = vec![
        "--root".to_string(),
        root_arg,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--card".to_string(),
        card_arg,
    ];

    let code = scope_guard::run_scope_guard(&args);
    assert_eq!(code, 0);
}

#[test]
fn test_scope_guard_out_of_scope_and_forbidden() {
    let dir = setup_temp_repo();
    let p = dir.path();

    fs::create_dir_all(p.join("crates/mycrate")).unwrap();
    fs::create_dir_all(p.join("crates/forbidden")).unwrap();
    fs::write(p.join("other.rs"), "fn other() {}\n").unwrap();
    fs::write(p.join("crates/forbidden/secret.rs"), "fn secret() {}\n").unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(p)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "out of scope and forbidden change"])
        .current_dir(p)
        .output()
        .unwrap();

    let root_arg = format!("{}", p.display());
    let card_arg = format!("{}/.jules/tasks/task-01.toml", p.display());
    let args = vec![
        "--root".to_string(),
        root_arg,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--card".to_string(),
        card_arg,
        "--json".to_string(),
    ];

    let code = scope_guard::run_scope_guard(&args);
    assert_eq!(code, 1);
}

#[test]
fn test_scope_guard_no_card_required_vs_optional() {
    let dir = setup_temp_repo();
    let p = dir.path();

    fs::write(p.join("file1.rs"), "fn f1() { // change\n }\n").unwrap();
    Command::new("git")
        .args(["commit", "-am", "change without trailer"])
        .current_dir(p)
        .output()
        .unwrap();

    let root_arg = format!("{}", p.display());
    let args_req = vec![
        "--root".to_string(),
        root_arg.clone(),
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--card-mode".to_string(),
        "required".to_string(),
    ];
    let code_req = scope_guard::run_scope_guard(&args_req);
    assert_eq!(code_req, 2);

    let args_opt = vec![
        "--root".to_string(),
        root_arg,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--card-mode".to_string(),
        "optional".to_string(),
    ];
    let code_opt = scope_guard::run_scope_guard(&args_opt);
    assert_eq!(code_opt, 0);
}

#[test]
fn test_scope_guard_rename_and_helpers() {
    assert!(scope_guard::scope_guard_matches_glob("a/b/c.rs", "**/*.rs"));
    assert!(scope_guard::scope_guard_matches_glob("foo.rs", "foo.rs"));
    assert_eq!(
        scope_guard::scope_guard_find_card_id("Task-Card: my-task-123"),
        Some("my-task-123".to_string())
    );
}
