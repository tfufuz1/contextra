use std::fs;
use tempfile::TempDir;

#[path = "../src/harness/task_card.rs"]
mod task_card;

fn task_card_test_setup_repo() -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    fs::create_dir_all(root.join(".jules/tasks")).unwrap();
    fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
    fs::write(
        root.join("capabilities.toml"),
        "[crates.contextra-core]\nstatus = \"active\"\n",
    )
    .unwrap();
    fs::write(
        root.join(".jules/tasks/_TEMPLATE.toml"),
        r#"id = "T-2026-0001"
title = "Kurzer, prägnanter Titel der Aufgabe"
crate = "contextra-core"
tier = 1
risk = "none"
goal = "Ziel"
non_goals = []
scope = ["crates/contextra-core/src/**"]
forbidden = ["xtask/**", ".github/**", "capabilities.toml", "AGENTS.md", "rust-toolchain.toml", "deny.toml", ".jules/setup/**"]
invariants = ["Zero-Panic"]
acceptance = ["cargo test -p contextra-core"]
evidence_required = ["Test evidence"]

[budget]
files = 5
lines = 200
iterations = 5
"#,
    )
    .unwrap();
    (temp_dir, root)
}

#[test]
fn test_task_card_new_and_lint_pass() {
    let (_temp, root) = task_card_test_setup_repo();

    let created_path = task_card::task_card_run_new(&root, "contextra-core", "Test Task").unwrap();
    assert!(created_path.exists());

    let findings = task_card::task_card_lint_file(&root, &created_path);
    let has_errors = findings.iter().any(|f| f.severity == "error");
    assert!(!has_errors, "Erwartet keine Fehler bei Template/New");

    let exit_code = task_card::run_task_card(&[
        "lint".to_string(),
        created_path.to_string_lossy().to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
    ]);
    assert_eq!(exit_code, 0);
}

#[test]
fn test_task_card_lint_failures() {
    let (_temp, root) = task_card_test_setup_repo();
    let invalid_card_path = root.join(".jules/tasks/invalid.toml");

    fs::write(
        &invalid_card_path,
        r#"id = "T-2026-9999"
title = "Ungültig"
crate = "contextra-unknown"
tier = 1
risk = "sec"
goal = "Goal"
non_goals = []
scope = ["**"]
forbidden = []
invariants = []
acceptance = ["invalid-cmd"]
evidence_required = []

[budget]
files = 30
lines = 2000
iterations = 15
"#,
    )
    .unwrap();

    let findings = task_card::task_card_lint_file(&root, &invalid_card_path);
    assert!(!findings.is_empty());
    assert!(findings.iter().any(|f| f.id == "invalid-crate"));
    assert!(findings.iter().any(|f| f.id == "full-glob-scope"));
    assert!(findings.iter().any(|f| f.id == "missing-protected-path"));
    assert!(findings.iter().any(|f| f.id == "invalid-acceptance-cmd"));
    assert!(findings.iter().any(|f| f.id == "missing-evidence"));
    assert!(findings.iter().any(|f| f.id == "budget-files-exceeded"));
    assert!(findings.iter().any(|f| f.id == "budget-lines-exceeded"));
    assert!(findings
        .iter()
        .any(|f| f.id == "budget-iterations-exceeded"));
}

#[test]
fn test_task_card_protected_change_override() {
    let (_temp, root) = task_card_test_setup_repo();
    let card_path = root.join(".jules/tasks/protected.toml");

    fs::write(
        &card_path,
        r#"id = "T-2026-8888"
title = "Protected Change"
crate = "contextra-core"
tier = 1
risk = "none"
goal = "Goal"
non_goals = []
scope = ["xtask/**"]
forbidden = []
invariants = []
acceptance = ["cargo test"]
evidence_required = []
protected_change = "ADR-001"

[budget]
files = 5
lines = 200
iterations = 5
"#,
    )
    .unwrap();

    let findings = task_card::task_card_lint_file(&root, &card_path);
    assert!(!findings.iter().any(|f| f.id == "missing-protected-path"));
}
