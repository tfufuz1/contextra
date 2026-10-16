use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_check_dag_dependency_cycle_fails_exit_code_non_zero() {
    let dir = tempdir().expect("Failed to create temp dir");
    let root_path = dir.path();

    // 1. Root Cargo.toml with members
    let root_cargo = r#"[workspace]
members = [
    "crates/crate-a",
    "crates/crate-b"
]
"#;
    fs::write(root_path.join("Cargo.toml"), root_cargo).expect("Failed to write root Cargo.toml");

    // 2. Member crate-a
    let crate_a_dir = root_path.join("crates").join("crate-a");
    fs::create_dir_all(&crate_a_dir).expect("Failed to create crate-a dir");
    let crate_a_cargo = r#"[package]
name = "contextra-crate-a"
version = "0.1.0"

[dependencies]
contextra-crate-b = { path = "../crate-b" }
"#;
    fs::write(crate_a_dir.join("Cargo.toml"), crate_a_cargo).expect("Failed to write crate-a Cargo.toml");

    // 3. Member crate-b forming cycle with crate-a
    let crate_b_dir = root_path.join("crates").join("crate-b");
    fs::create_dir_all(&crate_b_dir).expect("Failed to create crate-b dir");
    let crate_b_cargo = r#"[package]
name = "contextra-crate-b"
version = "0.1.0"

[dependencies]
contextra-crate-a = { path = "../crate-a" }
"#;
    fs::write(crate_b_dir.join("Cargo.toml"), crate_b_cargo).expect("Failed to write crate-b Cargo.toml");

    // Determine path to xtask manifest
    let manifest_path = if Path::new("xtask/Cargo.toml").exists() {
        "xtask/Cargo.toml"
    } else if Path::new("Cargo.toml").exists() {
        "Cargo.toml"
    } else {
        "../Cargo.toml"
    };

    let canonical_manifest = Path::new(manifest_path)
        .canonicalize()
        .expect("Failed to canonicalize manifest path");

    let output = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            canonical_manifest.to_str().unwrap(),
            "--",
            "check-dag",
        ])
        .current_dir(root_path)
        .env("CARGO_MANIFEST_DIR", root_path)
        .output()
        .expect("Failed to execute cargo xtask check-dag");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "cargo xtask check-dag MUST fail with exit code != 0 when dependency cycle exists.\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );

    let combined = format!("{}\n{}", stdout, stderr);
    assert!(
        combined.contains("Dependency cycle detected") || combined.contains("DAG-Fehler"),
        "Output should report dependency cycle error, got:\n{}",
        combined
    );
}

#[test]
fn test_check_dag_normal_codebase_passes() {
    let manifest_path = if Path::new("xtask/Cargo.toml").exists() {
        "xtask/Cargo.toml"
    } else if Path::new("Cargo.toml").exists() {
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
            "check-dag",
        ])
        .output()
        .expect("Failed to execute check-dag");

    assert!(
        output.status.success(),
        "cargo xtask check-dag MUST succeed on normal codebase.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
