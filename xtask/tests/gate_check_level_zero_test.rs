use std::process::Command;

fn get_manifest_path() -> &'static str {
    if std::path::Path::new("xtask/Cargo.toml").exists() {
        "xtask/Cargo.toml"
    } else if std::path::Path::new("Cargo.toml").exists() {
        "Cargo.toml"
    } else {
        "../Cargo.toml"
    }
}

#[test]
fn test_gate_check_level_zero_missing_crate_arg() {
    let manifest_path = get_manifest_path();
    let output = Command::new("cargo")
        .args(["run", "--manifest-path", manifest_path, "--", "gate-check", "--level", "0"])
        .output()
        .expect("Failed to execute xtask gate-check");

    assert!(!output.status.success(), "gate-check --level 0 without --crate must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}\n{}", stdout, stderr);
    assert!(
        combined.contains("erforderlich") || combined.contains("Fehler"),
        "Expected clear error message about missing --crate parameter, got: {}",
        combined
    );
}

#[test]
fn test_gate_check_level_zero_with_valid_crate() {
    let manifest_path = get_manifest_path();
    let output = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            manifest_path,
            "--",
            "gate-check",
            "--level",
            "0",
            "--crate",
            "contextra-types",
        ])
        .output()
        .expect("Failed to execute xtask gate-check");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);

    assert!(
        output.status.success() || combined.contains("Gate Check FEHLGESCHLAGEN"),
        "gate-check output should either succeed or cleanly report step failure"
    );
    assert!(
        combined.contains("contextra-types"),
        "Output should reference contextra-types crate"
    );
}

#[test]
fn test_gate_check_level_two_not_implemented() {
    let manifest_path = get_manifest_path();
    let output = Command::new("cargo")
        .args(["run", "--manifest-path", manifest_path, "--", "gate-check", "--level", "2"])
        .output()
        .expect("Failed to execute xtask gate-check");

    assert!(!output.status.success(), "gate-check --level 2 must fail as not implemented");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}\n{}", stdout, stderr);

    assert!(
        combined.contains("Gate 2 ist spezifiziert")
            && combined.contains("noch nicht als automatisierte Prüfung hinterlegt"),
        "Expected 'not implemented' message for level 2, got: {}",
        combined
    );
}
