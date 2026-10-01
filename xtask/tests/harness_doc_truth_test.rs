// Contextra — Test for Harness doc_truth

#[path = "../src/harness/doc_truth.rs"]
mod doc_truth;

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Helper to set up a git repository in a temp directory.
fn create_temp_git_repo() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let root = dir.path();

    let status = Command::new("git")
        .args(["init"])
        .current_dir(root)
        .status()
        .expect("Failed to run git init");
    assert!(status.success());

    // Configure git user for commits if needed
    let _ = Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(root)
        .status();
    let _ = Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(root)
        .status();

    dir
}

/// Creates a valid minimal workspace structure in the temp repo.
fn setup_valid_workspace(root: &std::path::Path) {
    // 1. Root Cargo.toml
    fs::write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = [
    "crates/contextra-types",
    "crates/contextra-simd",
]
"#,
    )
    .unwrap();

    // 2. capabilities.toml
    fs::write(
        root.join("capabilities.toml"),
        r#"[crates.contextra-types]
path = "crates/contextra-types"
unsafe_island = false

[crates.contextra-simd]
path = "crates/contextra-simd"
unsafe_island = true
"#,
    )
    .unwrap();

    // 3. AGENTS.md (Root)
    fs::write(
        root.join("AGENTS.md"),
        r"# AGENTS.md
Stand: 2026-09-28

## Start
- `cargo run --manifest-path xtask/Cargo.toml -- jules-preflight`

## Invarianten
- Unsafe-Isolierung: vier Unsafe-Inseln laut `capabilities.toml` (`contextra-simd`).
",
    )
    .unwrap();

    // 4. .jules directory
    fs::create_dir_all(root.join(".jules")).unwrap();

    // 5. .jules/JULES_CONTEXT.md
    fs::write(
        root.join(".jules/JULES_CONTEXT.md"),
        r"# JULES_CONTEXT.md

| Crate | Pfad |
|---|---|
| `contextra-types` | `crates/contextra-types/AGENTS.md` |
| `contextra-simd` | `crates/contextra-simd/AGENTS.md` |
",
    )
    .unwrap();

    // 6. .jules/SESSION_BOOTSTRAP.md
    fs::write(
        root.join(".jules/SESSION_BOOTSTRAP.md"),
        r"# SESSION_BOOTSTRAP.md
See `crates/contextra-types/src/lib.rs`
",
    )
    .unwrap();

    // 7. .jules/COMMON_LLM_ERRORS.md
    fs::write(root.join(".jules/COMMON_LLM_ERRORS.md"), "# Common Errors\n").unwrap();
    // 8. .jules/SCHEDULED_AUDIT.md
    fs::write(root.join(".jules/SCHEDULED_AUDIT.md"), "# Scheduled Audit\n").unwrap();
    // 9. .jules/AUDIT_INTAKE_PROTOCOL.md
    fs::write(root.join(".jules/AUDIT_INTAKE_PROTOCOL.md"), "# Audit Intake\n").unwrap();
    // 10. .jules/ISSUE_AUTOMATION_POLICY.md
    fs::write(root.join(".jules/ISSUE_AUTOMATION_POLICY.md"), "# Automation Policy\n").unwrap();

    // 11. Crates
    // contextra-types (non-island)
    let types_dir = root.join("crates/contextra-types");
    fs::create_dir_all(types_dir.join("src")).unwrap();
    fs::write(types_dir.join("Cargo.toml"), r#"[package]
name = "contextra-types"
version = "0.1.0"
edition = "2021"
"#).unwrap();
    fs::write(types_dir.join("AGENTS.md"), "# Types AGENTS\n").unwrap();
    fs::write(
        types_dir.join("src/lib.rs"),
        "#![forbid(unsafe_code)]\npub struct TenantId(pub u64);\n",
    )
    .unwrap();

    // contextra-simd (unsafe-island)
    let simd_dir = root.join("crates/contextra-simd");
    fs::create_dir_all(simd_dir.join("src")).unwrap();
    fs::write(simd_dir.join("Cargo.toml"), r#"[package]
name = "contextra-simd"
version = "0.1.0"
edition = "2021"
"#).unwrap();
    fs::write(simd_dir.join("AGENTS.md"), "# SIMD AGENTS\n").unwrap();
    fs::write(
        simd_dir.join("src/lib.rs"),
        "pub fn simd_add() { unsafe { } }\n",
    )
    .unwrap();

    // 12. xtask structure
    let xtask_dir = root.join("xtask/src");
    fs::create_dir_all(&xtask_dir).unwrap();
    fs::write(
        xtask_dir.join("main.rs"),
        r#"fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("jules-preflight") => {},
        _ => {}
    }
}"#,
    )
    .unwrap();
}

#[test]
fn test_doc_truth_valid_workspace_passes() {
    let temp_repo = create_temp_git_repo();
    let root = temp_repo.path();
    setup_valid_workspace(root);

    let code = doc_truth::run_doc_truth(&["--root".to_string(), root.to_str().unwrap().to_string()]);
    assert_eq!(code, 0, "Valid workspace must pass doc-truth check");
}

#[test]
fn test_doc_truth_phantom_path_fails() {
    let temp_repo = create_temp_git_repo();
    let root = temp_repo.path();
    setup_valid_workspace(root);

    // Add phantom path reference in AGENTS.md
    fs::write(
        root.join("AGENTS.md"),
        r"# AGENTS.md
Unsafe-Isolierung: contextra-simd
Reference to `non_existent_file_xyz.rs`.
",
    )
    .unwrap();

    let code = doc_truth::run_doc_truth(&[
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
        "--strict".to_string(),
    ]);
    assert_eq!(code, 1, "Phantom path reference must trigger failure in strict mode");
}

#[test]
fn test_doc_truth_ignore_and_planned_markers() {
    let temp_repo = create_temp_git_repo();
    let root = temp_repo.path();
    setup_valid_workspace(root);

    // Add ignored path and planned command in AGENTS.md
    fs::write(
        root.join("AGENTS.md"),
        r"# AGENTS.md
Unsafe-Isolierung: contextra-simd
Reference to `non_existent_file_xyz.rs` <!-- doc-ref-ignore -->
Run `cargo xtask future-command` <!-- harness:planned -->
",
    )
    .unwrap();

    let code = doc_truth::run_doc_truth(&[
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
        "--strict".to_string(),
    ]);
    assert_eq!(code, 0, "Ignored paths and planned commands should not fail gate");
}

#[test]
fn test_doc_truth_unknown_crate_fails() {
    let temp_repo = create_temp_git_repo();
    let root = temp_repo.path();
    setup_valid_workspace(root);

    // Add unknown crate reference in JULES_CONTEXT.md
    fs::write(
        root.join(".jules/JULES_CONTEXT.md"),
        r"# JULES_CONTEXT.md
See `contextra-nonexistent-crate`.
",
    )
    .unwrap();

    let code = doc_truth::run_doc_truth(&[
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
        "--strict".to_string(),
    ]);
    assert_eq!(code, 1, "Unknown crate reference must fail doc-truth");
}

#[test]
fn test_doc_truth_unsafe_mismatch_fails() {
    let temp_repo = create_temp_git_repo();
    let root = temp_repo.path();
    setup_valid_workspace(root);

    // Add unsafe code in non-island crate
    fs::write(
        root.join("crates/contextra-types/src/lib.rs"),
        "pub fn bad() { unsafe { } }\n",
    )
    .unwrap();

    let code = doc_truth::run_doc_truth(&[
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
        "--strict".to_string(),
    ]);
    assert_eq!(code, 1, "Unsafe code in non-island crate must fail doc-truth");
}

#[test]
fn test_doc_truth_crate_table_mismatch_fails() {
    let temp_repo = create_temp_git_repo();
    let root = temp_repo.path();
    setup_valid_workspace(root);

    // Remove contextra-simd from JULES_CONTEXT.md crate table
    fs::write(
        root.join(".jules/JULES_CONTEXT.md"),
        r"# JULES_CONTEXT.md

| Crate | Pfad |
|---|---|
| `contextra-types` | `crates/contextra-types/AGENTS.md` |
",
    )
    .unwrap();

    let code = doc_truth::run_doc_truth(&[
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
        "--strict".to_string(),
    ]);
    assert_eq!(code, 1, "Missing crate in JULES_CONTEXT.md crate table must fail doc-truth");
}

#[test]
fn test_doc_truth_ignores_log_files() {
    let temp_repo = create_temp_git_repo();
    let root = temp_repo.path();
    setup_valid_workspace(root);

    // Create .jules/JULES_LOG.md with invalid references
    fs::write(
        root.join(".jules/JULES_LOG.md"),
        "Historical log referencing `invalid_file_123.rs` and `contextra-fake-crate`.\n",
    )
    .unwrap();

    let code = doc_truth::run_doc_truth(&[
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
        "--strict".to_string(),
    ]);
    assert_eq!(code, 0, "JULES_LOG.md must be ignored by doc-truth");
}

#[test]
fn test_doc_truth_json_output() {
    let temp_repo = create_temp_git_repo();
    let root = temp_repo.path();
    setup_valid_workspace(root);

    let code = doc_truth::run_doc_truth(&[
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
        "--json".to_string(),
    ]);
    assert_eq!(code, 0);
}

#[test]
fn test_doc_truth_real_repo_standalone() {
    let root = doc_truth::doc_truth_find_root();
    let mut findings = Vec::new();
    doc_truth::doc_truth_check_paths_pub(&root, &mut findings);
    doc_truth::doc_truth_check_crates_pub(&root, &mut findings);
    doc_truth::doc_truth_check_commands_pub(&root, &mut findings);
    doc_truth::doc_truth_check_unsafe_islands_pub(&root, &mut findings);
    doc_truth::doc_truth_check_crate_table_pub(&root, &mut findings);

    for f in &findings {
        if f.severity == "error" {
            eprintln!("ERROR: {} ({}:{}) -> {}", f.id, f.file, f.line, f.message);
        }
    }

    let code = doc_truth::run_doc_truth(&[
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
    ]);
    assert_eq!(code, 0, "doc-truth on real repo must return exit code 0");
}
