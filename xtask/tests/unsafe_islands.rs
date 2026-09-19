//! Integration wrapper test for check-unsafe-islands

use std::process::Command;

#[test]
fn test_unsafe_islands_subcommand_default_exit_zero() {
    let manifest_path = if std::path::Path::new("xtask/Cargo.toml").exists() {
        "xtask/Cargo.toml"
    } else if std::path::Path::new("Cargo.toml").exists() {
        "Cargo.toml"
    } else {
        "../Cargo.toml"
    };

    let output = Command::new("cargo")
        .args(["run", "--manifest-path", manifest_path, "--", "check-unsafe-islands"])
        .output()
        .expect("Failed to execute check-unsafe-islands");

    assert!(
        output.status.success(),
        "check-unsafe-islands in default transition mode must exit 0, got stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
