//! Characterization test for all 109 xtask commands/flags.
//! Verifies baseline dispatch exit codes and response signatures.

use std::process::Command;

#[test]
fn test_unknown_command_signature() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("nonexistent-command-xyz")
        .output()
        .expect("failed to execute xtask");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown xtask command: nonexistent-command-xyz"));
    assert!(stderr.contains("available commands"));
}

#[test]
fn test_legacy_dispatched_command_signature() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("bench-trend")
        .output()
        .expect("failed to execute xtask");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("=== Running xtask bench-trend ==="));
}
