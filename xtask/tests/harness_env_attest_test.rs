// Test for harness module env-attest

#[path = "../src/harness/env_attest.rs"]
mod env_attest;

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Mutex;

static PATH_MUTEX: Mutex<()> = Mutex::new(());

fn setup_temp_env() -> (tempfile::TempDir, tempfile::TempDir) {
    let root_dir = tempfile::tempdir().expect("failed to create temp root dir");
    let bin_dir = tempfile::tempdir().expect("failed to create temp bin dir");

    let root = root_dir.path();
    let bin = bin_dir.path();

    // Write rust-toolchain.toml
    fs::write(
        root.join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"1.89.0\"\ncomponents = [\"clippy\", \"rustfmt\"]\n",
    )
    .unwrap();

    // Write required-tools.toml
    let jules_setup = root.join(".jules/setup");
    fs::create_dir_all(&jules_setup).unwrap();
    fs::write(
        jules_setup.join("required-tools.toml"),
        r#"
[[tool]]
name = "mockreq"
version_cmd = ["mockreq", "--version"]
required = true

[[tool]]
name = "mockopt"
version_cmd = ["mockopt", "--version"]
required = false
"#,
    )
    .unwrap();

    // Create mock executables in bin_dir
    create_mock_bin(
        bin,
        "rustc",
        "#!/bin/sh\necho 'rustc 1.89.0 (2026-01-01)'\n",
    );
    create_mock_bin(
        bin,
        "cargo",
        "#!/bin/sh\necho 'cargo 1.89.0 (2026-01-01)'\n",
    );
    create_mock_bin(bin, "mockreq", "#!/bin/sh\necho 'mockreq v1.0'\n");

    (root_dir, bin_dir)
}

fn create_mock_bin(dir: &std::path::Path, name: &str, content: &str) {
    let file_path = dir.join(name);
    fs::write(&file_path, content).unwrap();
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&file_path, perms).unwrap();
}

#[test]
fn test_env_attest_pass() {
    let _guard = PATH_MUTEX.lock().unwrap();
    let (root_dir, bin_dir) = setup_temp_env();
    let orig_path = env::var("PATH").unwrap_or_default();
    let new_path_str = format!("{}:{}", bin_dir.path().display(), orig_path);

    env::set_var("PATH", &new_path_str);

    // Test direct helper functions
    let pin = env_attest::env_attest_read_toolchain_pin(root_dir.path()).unwrap();
    assert_eq!(pin, "1.89.0");

    let tools = env_attest::env_attest_read_required_tools(root_dir.path()).unwrap();
    assert_eq!(tools.len(), 2);

    let ver = env_attest::env_attest_check_cmd(&["mockreq".to_string(), "--version".to_string()])
        .unwrap();
    assert_eq!(ver, "mockreq v1.0");

    let report = env_attest::env_attest_execute(root_dir.path());
    assert_eq!(report.status, "pass");

    // Check generated report file
    let report_file = root_dir.path().join(".jules/local/env-attest.json");
    assert!(report_file.exists());

    let code = env_attest::run_env_attest(&[
        "--root".to_string(),
        root_dir.path().to_str().unwrap().to_string(),
        "--json".to_string(),
    ]);
    assert_eq!(code, 0);

    env::set_var("PATH", orig_path);
}

#[test]
fn test_env_attest_wrong_toolchain() {
    let _guard = PATH_MUTEX.lock().unwrap();
    let (root_dir, bin_dir) = setup_temp_env();
    let orig_path = env::var("PATH").unwrap_or_default();

    // Override rustc with wrong version 1.87.0
    create_mock_bin(bin_dir.path(), "rustc", "#!/bin/sh\necho 'rustc 1.87.0'\n");

    let new_path_str = format!("{}:{}", bin_dir.path().display(), orig_path);
    env::set_var("PATH", &new_path_str);

    let report = env_attest::env_attest_execute(root_dir.path());
    assert_eq!(report.status, "fail");
    assert!(report
        .findings
        .iter()
        .any(|f| f.id == "rustc_version_mismatch"));

    let code = env_attest::run_env_attest(&[
        "--root".to_string(),
        root_dir.path().to_str().unwrap().to_string(),
    ]);
    assert_eq!(code, 1);

    env::set_var("PATH", orig_path);
}

#[test]
fn test_env_attest_missing_required_tool() {
    let _guard = PATH_MUTEX.lock().unwrap();
    let (root_dir, _bin_dir) = setup_temp_env();
    let orig_path = env::var("PATH").unwrap_or_default();

    let bin_only_rust = tempfile::tempdir().expect("failed temp dir");
    create_mock_bin(
        bin_only_rust.path(),
        "rustc",
        "#!/bin/sh\necho 'rustc 1.89.0'\n",
    );
    create_mock_bin(
        bin_only_rust.path(),
        "cargo",
        "#!/bin/sh\necho 'cargo 1.89.0'\n",
    );

    // Pass bin_only_rust in front without bin_dir containing mockreq
    let empty_path = tempfile::tempdir().expect("empty temp dir");
    let new_path_str = format!(
        "{}:{}",
        bin_only_rust.path().display(),
        empty_path.path().display()
    );
    env::set_var("PATH", &new_path_str);

    let report = env_attest::env_attest_execute(root_dir.path());
    assert_eq!(report.status, "fail");
    assert!(report
        .findings
        .iter()
        .any(|f| f.id == "required_tool_missing_mockreq"));

    env::set_var("PATH", orig_path);
}
