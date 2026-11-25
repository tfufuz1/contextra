//! Integration wrapper test for check-result-dropped-io gate

use std::process::Command;

#[test]
fn test_check_result_dropped_io_subcommand_workspace_passes() {
    let manifest_path = if std::path::Path::new("xtask/Cargo.toml").exists() {
        "xtask/Cargo.toml"
    } else if std::path::Path::new("Cargo.toml").exists() {
        "Cargo.toml"
    } else {
        "../Cargo.toml"
    };

    let output = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            manifest_path,
            "--",
            "check-result-dropped-io",
        ])
        .output()
        .expect("Failed to execute check-result-dropped-io");

    assert!(
        output.status.success(),
        "check-result-dropped-io on workspace must exit 0, got stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
