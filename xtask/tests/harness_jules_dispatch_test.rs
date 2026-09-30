use std::fs;
use tempfile::TempDir;

#[path = "../src/harness/jules_dispatch.rs"]
mod jules_dispatch;

struct MockHttpTransport {
    pub response: Result<String, String>,
}

impl jules_dispatch::JulesHttpTransport for MockHttpTransport {
    fn post(
        &self,
        _url: &str,
        _header: &str,
        _api_key: &str,
        _body: &str,
    ) -> Result<String, String> {
        self.response.clone()
    }
}

fn jules_dispatch_test_setup_repo() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
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
        root.join(".jules/jules-api.toml"),
        r#"verified = false
[endpoints]
base_url = "https://jules.googleapis.com/v1alpha"
sessions = "/sessions"
"#,
    )
    .unwrap();
    fs::write(root.join(".jules/PREAMBLE.md"), "1. Start session.").unwrap();

    let card_path = root.join(".jules/tasks/T-2026-0001.toml");
    fs::write(
        &card_path,
        r#"id = "T-2026-0001"
title = "Dispatch Test"
crate = "contextra-core"
tier = 1
risk = "none"
goal = "Goal"
non_goals = []
scope = ["crates/contextra-core/src/**"]
forbidden = ["xtask/**", ".github/**", "capabilities.toml", "AGENTS.md", "rust-toolchain.toml", "deny.toml", ".jules/setup/**"]
invariants = ["Zero-Panic"]
acceptance = ["cargo test -p contextra-core"]
evidence_required = []

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
fn test_jules_dispatch_dry_run() {
    let (_temp, root, card_path) = jules_dispatch_test_setup_repo();
    let mock = MockHttpTransport {
        response: Ok("{}".to_string()),
    };

    let (code, res) = jules_dispatch::jules_dispatch_execute(&root, &card_path, false, &mock);
    assert_eq!(code, 0);
    let payload = res.get("payload").unwrap();
    assert_eq!(payload.get("requirePlanApproval").unwrap(), true);
    assert_eq!(payload.get("automationMode").unwrap(), "MANUAL");

    // Exercise functions in jules_dispatch
    let _ = jules_dispatch::jules_dispatch_find_root(&root);
    let _ = jules_dispatch::CurlHttpTransport;
    let _ = jules_dispatch::run_jules_dispatch(&[
        "--card".to_string(),
        card_path.to_string_lossy().to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
    ]);
}

#[test]
fn test_jules_dispatch_reject_auto_create_pr() {
    let (_temp, root, card_path) = jules_dispatch_test_setup_repo();
    fs::write(
        root.join(".jules/jules-api.toml"),
        "automation_mode = \"AUTO_CREATE_PR\"\n",
    )
    .unwrap();

    let mock = MockHttpTransport {
        response: Ok("{}".to_string()),
    };
    let (code, res) = jules_dispatch::jules_dispatch_execute(&root, &card_path, false, &mock);
    assert_eq!(code, 2);
    assert!(res
        .get("summary")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("AUTO_CREATE_PR"));
}

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_jules_dispatch_send_with_key_and_redaction() {
    let _guard = ENV_LOCK.lock().unwrap();
    let (_temp, root, card_path) = jules_dispatch_test_setup_repo();

    let mock_response =
        r#"{"name": "sessions/sess-9999", "secret-key-12345": "exposed"}"#.to_string();
    let mock = MockHttpTransport {
        response: Ok(mock_response),
    };

    // Set variable explicitly for the test run
    std::env::set_var("JULES_API_KEY", "secret-key-12345");
    let (code, res) = jules_dispatch::jules_dispatch_execute(&root, &card_path, true, &mock);
    std::env::set_var("JULES_API_KEY", "");

    assert_eq!(code, 0);
    assert_eq!(res.get("session_id").unwrap(), "sess-9999");

    let session_file = root.join(".jules/local/sessions/sess-9999.json");
    assert!(session_file.exists());
    let session_content = fs::read_to_string(&session_file).unwrap();
    assert!(!session_content.contains("secret-key-12345"));
    assert!(session_content.contains("[REDACTED_API_KEY]"));
}

#[test]
fn test_jules_dispatch_missing_key_exit_2() {
    let _guard = ENV_LOCK.lock().unwrap();
    let (_temp, root, card_path) = jules_dispatch_test_setup_repo();
    std::env::set_var("JULES_API_KEY", "");

    let mock = MockHttpTransport {
        response: Ok("{}".to_string()),
    };
    let (code, res) = jules_dispatch::jules_dispatch_execute(&root, &card_path, true, &mock);
    assert_eq!(code, 2);
    assert!(res
        .get("summary")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("JULES_API_KEY"));
}
