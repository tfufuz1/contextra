#[path = "../src/harness/gate_guard.rs"]
mod gate_guard;

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn create_temp_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let p = temp_dir.path();

    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(p)
            .args(args)
            .status()
            .expect("failed to execute git command");
        assert!(status.success(), "git command failed: {:?}", args);
    };

    run_git(&["init", "-b", "main"]);
    run_git(&["config", "user.name", "Tester"]);
    run_git(&["config", "user.email", "tester@example.com"]);

    fs::create_dir_all(p.join(".jules/tasks")).expect("failed to create tasks dir");
    fs::create_dir_all(p.join("governance")).expect("failed to create governance dir");
    fs::create_dir_all(p.join("docs/decisions")).expect("failed to create docs/decisions dir");
    fs::create_dir_all(p.join("src")).expect("failed to create src dir");

    fs::write(
        p.join("governance/protected-paths.toml"),
        r#"
[[protected]]
glob = "governance/**"
reason = "Governance"

[[protected]]
glob = "xtask/src/harness/protected_paths.rs"
reason = "Self protection"
"#,
    )
    .expect("failed to write protected paths");

    fs::write(
        p.join(".jules/tasks/task-001.toml"),
        r#"
id = "task-001"
scope = ["src/**"]
forbidden = ["xtask/**"]
"#,
    )
    .expect("failed to write task card");

    fs::write(p.join("src/lib.rs"), "pub fn ok() {}\n").expect("failed to write src/lib.rs");

    run_git(&["add", "."]);
    run_git(&["commit", "-m", "Initial commit\n\nTask-Card: task-001"]);

    temp_dir
}

fn commit_changes(root: &Path, msg: &str) {
    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(root)
            .args(args)
            .status()
            .expect("failed to execute git command");
        assert!(status.success(), "git command failed: {:?}", args);
    };

    run_git(&["add", "."]);
    run_git(&["commit", "-m", msg]);
}

#[test]
fn test_gate_guard_clean_repo_passes() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    fs::write(
        root.join("src/lib.rs"),
        "pub fn ok() { println!(\"clean\"); }\n",
    )
    .expect("failed to update src/lib.rs");
    commit_changes(root, "feat: clean in-scope change");

    let root_str = root.to_string_lossy().to_string();
    let card_path = root.join(".jules/tasks/task-001.toml");
    let card_str = card_path.to_string_lossy().to_string();

    let args = vec![
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--card".to_string(),
        card_str,
    ];

    let code = gate_guard::run_gate_guard(&args);
    assert_eq!(code, 0, "gate-guard MUST pass on clean repository");
}

#[test]
fn test_gate_guard_canary_scope_guard_defect() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    // Defect: out-of-scope file
    fs::create_dir_all(root.join("xtask")).expect("failed to create xtask dir");
    fs::write(root.join("xtask/forbidden.rs"), "fn forbidden() {}\n")
        .expect("failed to write forbidden.rs");
    commit_changes(root, "feat: forbidden edit");

    let root_str = root.to_string_lossy().to_string();
    let card_path = root.join(".jules/tasks/task-001.toml");
    let card_str = card_path.to_string_lossy().to_string();

    let args = vec![
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--card".to_string(),
        card_str,
    ];

    let code = gate_guard::run_gate_guard(&args);
    assert_ne!(code, 0, "gate-guard MUST fail on scope-guard defect");
}

#[test]
fn test_gate_guard_canary_protected_paths_defect() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    // Defect: protected file touched without authorization
    fs::write(root.join("governance/protected-paths.toml"), "# modified\n")
        .expect("failed to modify protected file");
    commit_changes(root, "chore: modify protected paths");

    let root_str = root.to_string_lossy().to_string();
    let card_path = root.join(".jules/tasks/task-001.toml");
    let card_str = card_path.to_string_lossy().to_string();

    let args = vec![
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--card".to_string(),
        card_str,
    ];

    let code = gate_guard::run_gate_guard(&args);
    assert_ne!(code, 0, "gate-guard MUST fail on protected-paths defect");
}

#[test]
fn test_gate_guard_canary_gate_weakening_defect() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    // Defect: #[allow(unused)] attribute introduced
    fs::write(
        root.join("src/lib.rs"),
        "#[allow(unused)]\npub fn ok() {}\n",
    )
    .expect("failed to write allow attr");
    commit_changes(root, "chore: introduce allow attribute");

    let root_str = root.to_string_lossy().to_string();
    let card_path = root.join(".jules/tasks/task-001.toml");
    let card_str = card_path.to_string_lossy().to_string();

    let args = vec![
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--card".to_string(),
        card_str,
    ];

    let code = gate_guard::run_gate_guard(&args);
    assert_ne!(code, 0, "gate-guard MUST fail on gate-weakening defect");
}

#[test]
fn test_gate_guard_canary_diff_budget_defect() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    // Defect: Diff budget exceeded
    let mut huge_code = String::new();
    for i in 0..1000 {
        huge_code.push_str(&format!("// line {}\n", i));
    }
    fs::write(root.join("src/lib.rs"), huge_code).expect("failed to write huge diff");
    commit_changes(root, "feat: huge diff exceeding budget");

    let root_str = root.to_string_lossy().to_string();
    let card_path = root.join(".jules/tasks/task-001.toml");
    let card_str = card_path.to_string_lossy().to_string();

    let args = vec![
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        "HEAD~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--card".to_string(),
        card_str,
    ];

    let code = gate_guard::run_gate_guard(&args);
    assert_ne!(code, 0, "gate-guard MUST fail on diff-budget defect");
}
