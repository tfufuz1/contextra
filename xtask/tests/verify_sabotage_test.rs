// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Sabotage tests for `check_loom_mocks` and `verify` steps.

use std::fs;
use tempfile::tempdir;

#[test]
fn test_check_loom_mocks_sabotage_detects_mock() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let loom_file = root.join("crates/contextra-test/tests/loom_unapproved_mock.rs");
    fs::create_dir_all(loom_file.parent().unwrap()).unwrap();

    fs::write(
        &loom_file,
        "struct MockRepository;\nfn test_loom() { let _m = MockRepository; }\n",
    )
    .unwrap();

    // Create minimal Cargo.toml with [workspace] so find_root_dir recognizes root
    fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = [\"crates/contextra-test\"]\n").unwrap();

    let root_str = root.to_string_lossy().to_string();
    let res = xtask::check_loom_mocks::run_check_loom_mocks(&["--root".to_string(), root_str]);

    assert_ne!(res, 0, "check_loom_mocks must fail when unapproved Mock pattern is found");
}

#[test]
fn test_verify_step_fails_on_command_failure() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let run_dir = root.join("target/evidence/test_run");
    fs::create_dir_all(&run_dir).unwrap();

    let runner = xtask::verify::steps::StepRunner {
        root,
        run_dir: &run_dir,
        rel_run_dir: "target/evidence/test_run",
        toolchain_info: "rustc 1.89.0-test",
        commit_sha: "testsha123",
        allow_skips: &[],
    };

    let ev = runner.run_step("sabotage-fail", &["false"], None, None);
    assert_eq!(ev.status, xtask::verify::evidence::StepStatus::Failed);
    assert_ne!(ev.exit_code, 0);
    assert!(ev.bytes > 0 || ev.exit_code == 1);
}

#[test]
fn test_verify_step_missing_tool_skipped() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let run_dir = root.join("target/evidence/test_run");
    fs::create_dir_all(&run_dir).unwrap();

    let runner = xtask::verify::steps::StepRunner {
        root,
        run_dir: &run_dir,
        rel_run_dir: "target/evidence/test_run",
        toolchain_info: "rustc 1.89.0-test",
        commit_sha: "testsha123",
        allow_skips: &[],
    };

    let ev = runner.run_step(
        "missing-tool-step",
        &["non_existent_tool_xyz_123"],
        None,
        Some("non_existent_tool_xyz_123"),
    );
    assert_eq!(
        ev.status,
        xtask::verify::evidence::StepStatus::SkippedToolMissing
    );
    assert_eq!(ev.exit_code, 127);
}
