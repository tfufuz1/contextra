// Test for harness module session-report

#[path = "../src/harness/session_report.rs"]
mod session_report;

use std::fs;
use std::process::Command;

fn setup_temp_repo() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().expect("failed to create temp repo");
    let path = temp_dir.path();

    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(path)
            .args(args)
            .status()
            .expect("failed git command");
        assert!(status.success());
    };

    run_git(&["init"]);
    run_git(&["config", "user.name", "Test User"]);
    run_git(&["config", "user.email", "test@example.com"]);

    fs::write(path.join("file1.txt"), "init\n").unwrap();
    run_git(&["add", "file1.txt"]);
    run_git(&["commit", "-m", "initial commit"]);

    // Create branch main
    run_git(&["branch", "-M", "main"]);

    // Create commit on HEAD
    fs::write(path.join("file2.txt"), "new file\n").unwrap();
    run_git(&["add", "file2.txt"]);
    run_git(&["commit", "-m", "feat: second commit"]);

    temp_dir
}

#[test]
fn test_session_report_generation_full() {
    let repo = setup_temp_repo();
    let root = repo.path();

    let card_file = root.join("card.md");
    fs::write(
        &card_file,
        "Implement harness system.\nRelevant Invariant: INV-HARNESS-1.\n",
    )
    .unwrap();

    let results_dir = root.join("results");
    fs::create_dir_all(&results_dir).unwrap();

    let gate_res = serde_json::json!({
        "gate": "fmt",
        "status": "pass",
        "summary": "cargo fmt clean",
        "findings": []
    });
    fs::write(
        results_dir.join("gate_fmt.json"),
        serde_json::to_string_pretty(&gate_res).unwrap(),
    )
    .unwrap();

    let findings_file = root.join("findings.txt");
    fs::write(&findings_file, "Minor refactoring note in crate X.\n").unwrap();

    let body = session_report::session_report_generate(
        root,
        "main~1",
        "HEAD",
        Some(&card_file),
        None,
        Some(&results_dir),
        Some(&findings_file),
    );

    assert!(body.contains("## Ziel\nImplement harness system."));
    assert!(body.contains("## Änderungen\n- file2.txt"));
    assert!(body.contains("INV-HARNESS-1"));
    assert!(body.contains("- **fmt**: PASS — cargo fmt clean"));
    assert!(body.contains("Minor refactoring note in crate X."));
}

#[test]
fn test_session_report_kein_beleg() {
    let repo = setup_temp_repo();
    let root = repo.path();

    let body =
        session_report::session_report_generate(root, "HEAD", "HEAD", None, None, None, None);

    assert!(body.contains("## Ziel\nKEIN BELEG"));
    assert!(body.contains("## Änderungen\nKEIN BELEG"));
    assert!(body.contains("## Verifikation\nKEIN BELEG"));
}

#[test]
fn test_session_report_run_command() {
    let repo = setup_temp_repo();
    let root = repo.path();

    let code = session_report::run_session_report(&[
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
        "--base".to_string(),
        "main~1".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--goal".to_string(),
        "Direct goal argument test".to_string(),
    ]);

    assert_eq!(code, 0);

    let pr_body_file = root.join(".jules/local/PR_BODY.md");
    assert!(pr_body_file.exists());

    let content = fs::read_to_string(&pr_body_file).unwrap();
    assert!(content.contains("Direct goal argument test"));
    assert!(content.contains("file2.txt"));
}
