use std::fs;
use tempfile::TempDir;

#[path = "../src/harness/plan_lint.rs"]
mod plan_lint;

struct MockPlanLintTransport {
    pub response: Result<String, String>,
}

impl plan_lint::PlanLintHttpTransport for MockPlanLintTransport {
    fn approve_plan(
        &self,
        _url: &str,
        _header: &str,
        _api_key: &str,
        _body: &str,
    ) -> Result<String, String> {
        self.response.clone()
    }
}

fn plan_lint_test_setup_repo() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();
    fs::create_dir_all(root.join(".jules/tasks")).unwrap();
    fs::create_dir_all(root.join(".jules/local/sessions")).unwrap();
    fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
    fs::write(
        root.join("capabilities.toml"),
        "[crates.contextra-core]\nstatus = \"active\"\n",
    )
    .unwrap();

    let card_path = root.join(".jules/tasks/T-2026-0001.toml");
    fs::write(
        &card_path,
        r#"id = "T-2026-0001"
title = "Plan Lint Test"
crate = "contextra-core"
tier = 1
risk = "sec"
goal = "Goal"
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

    (temp_dir, root, card_path)
}

#[test]
fn test_plan_lint_valid_plan_pass() {
    let (_temp, root, card_path) = plan_lint_test_setup_repo();
    let plan_text = r#"
1. `crates/contextra-core/src/lib.rs` bearbeiten.
2. Führe erst einen fehlschlagenden Test aus (rot).
3. `cargo test -p contextra-core` ausführen.
"#;

    let (code, res) = plan_lint::plan_lint_process_session(
        &root,
        Some("sess-1"),
        plan_text,
        &card_path,
        false,
        &MockPlanLintTransport {
            response: Ok("{}".to_string()),
        },
    );

    assert_eq!(code, 0);
    assert_eq!(res.get("status").unwrap(), "pass");

    let _ = plan_lint::plan_lint_find_root(&root);
    let _ = plan_lint::PlanLintCurlTransport;
    let _ = plan_lint::run_plan_lint(&[
        "--card".to_string(),
        card_path.to_string_lossy().to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
    ]);
}

#[test]
fn test_plan_lint_out_of_scope_and_forbidden_fails() {
    let (_temp, root, card_path) = plan_lint_test_setup_repo();
    let plan_text = r#"
1. `crates/contextra-store/src/lib.rs` bearbeiten.
2. `xtask/src/main.rs` bearbeiten.
3. `cargo test -p contextra-core` ausführen.
"#;

    let (code, res) = plan_lint::plan_lint_process_session(
        &root,
        Some("sess-1"),
        plan_text,
        &card_path,
        false,
        &MockPlanLintTransport {
            response: Ok("{}".to_string()),
        },
    );

    assert_eq!(code, 1);
    assert_eq!(res.get("status").unwrap(), "fail");
}

#[test]
fn test_plan_lint_unjustified_dependency_fails() {
    let (_temp, root, card_path) = plan_lint_test_setup_repo();
    let plan_text = r#"
1. `crates/contextra-core/src/lib.rs` bearbeiten.
2. Add dependency in `Cargo.toml`.
3. Test rot ausführen.
4. `cargo test -p contextra-core` ausführen.
"#;

    let (code, res) = plan_lint::plan_lint_process_session(
        &root,
        Some("sess-1"),
        plan_text,
        &card_path,
        false,
        &MockPlanLintTransport {
            response: Ok("{}".to_string()),
        },
    );

    assert_eq!(code, 1);
    let findings = res.get("findings").unwrap().as_array().unwrap();
    assert!(findings
        .iter()
        .any(|f| f.get("id").unwrap() == "unjustified-dependency-change"));
}

#[test]
fn test_plan_lint_escalation_after_3_rejections() {
    let (_temp, root, card_path) = plan_lint_test_setup_repo();
    let session_file = root.join(".jules/local/sessions/sess-1.json");
    fs::write(
        &session_file,
        r#"{"session_id": "sess-1", "revision_counter": 2}"#,
    )
    .unwrap();

    let invalid_plan = "Invalid plan without scope or commands";
    let (code, res) = plan_lint::plan_lint_process_session(
        &root,
        Some("sess-1"),
        invalid_plan,
        &card_path,
        false,
        &MockPlanLintTransport {
            response: Ok("{}".to_string()),
        },
    );

    assert_eq!(code, 2);
    assert_eq!(res.get("status").unwrap(), "error");
    assert!(res
        .get("summary")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("Eskalation"));
}
