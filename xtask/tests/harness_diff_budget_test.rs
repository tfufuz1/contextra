#[path = "../src/harness/diff_budget.rs"]
mod diff_budget;

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
fn test_diff_budget_within_limits() {
    let dir = setup_temp_repo();
    let p = dir.path();

    fs::write(p.join("file1.rs"), "fn f1() { println!(\"1\"); }\n").unwrap();
    Command::new("git")
        .args(["commit", "-am", "small change"])
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

    let code = diff_budget::run_diff_budget(&args);
    assert_eq!(code, 0);
}

#[test]
fn test_diff_budget_exceed_lines() {
    let dir = setup_temp_repo();
    let p = dir.path();

    let mut big_code = String::new();
    for i in 0..700 {
        big_code.push_str(&format!("// line {}\n", i));
    }
    fs::write(p.join("file1.rs"), big_code).unwrap();
    Command::new("git")
        .args(["commit", "-am", "large change"])
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

    let code = diff_budget::run_diff_budget(&args);
    assert_eq!(code, 1);
}

#[test]
fn test_diff_budget_exclusions() {
    let dir = setup_temp_repo();
    let p = dir.path();

    fs::write(p.join("Cargo.lock"), "lockcontent\n").unwrap();
    fs::create_dir_all(p.join("docs/generated")).unwrap();
    fs::write(p.join("docs/generated/doc.md"), "doccontent\n").unwrap();
    fs::write(p.join("test.snap"), "snapcontent\n").unwrap();
    fs::write(p.join("gen.rs"), "// @generated\nfn gen() {}\n").unwrap();

    assert!(diff_budget::diff_budget_is_excluded("Cargo.lock", p));
    assert!(diff_budget::diff_budget_is_excluded(
        "docs/generated/doc.md",
        p
    ));
    assert!(diff_budget::diff_budget_is_excluded("test.snap", p));
    assert!(diff_budget::diff_budget_is_excluded("gen.rs", p));
}

#[test]
fn test_diff_budget_card_override() {
    let dir = setup_temp_repo();
    let p = dir.path();

    fs::create_dir_all(p.join(".jules/tasks")).unwrap();
    fs::write(
        p.join(".jules/tasks/task-budget.toml"),
        r#"
id = "task-budget"
[budget]
lines = 50
"#,
    )
    .unwrap();

    let mut mid_code = String::new();
    for i in 0..100 {
        mid_code.push_str(&format!("// line {}\n", i));
    }
    fs::write(p.join("file1.rs"), mid_code).unwrap();
    Command::new("git")
        .args(["commit", "-am", "mid change\n\nTask-Card: task-budget"])
        .current_dir(p)
        .output()
        .unwrap();

    let root_arg = format!("{}", p.display());
    let card_arg = format!("{}/.jules/tasks/task-budget.toml", p.display());
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

    let code = diff_budget::run_diff_budget(&args);
    assert_eq!(code, 1);
}
