//! Integration test canary suite verifying that quality gates detect defects and pass on valid input.

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::all)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Dummy find_root_dir to satisfy module internal references in test context.
pub fn find_root_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[path = "../../src/harness/protected_paths.rs"]
mod protected_paths;

#[path = "../../src/harness/gate_weakening.rs"]
mod gate_weakening;

#[path = "../../src/harness/determinism_check.rs"]
mod determinism_check;

#[path = "../../src/check_unsafe_islands.rs"]
mod check_unsafe_islands;

#[path = "../../src/harness/scope_guard.rs"]
mod scope_guard;

#[path = "../../src/harness/test_integrity.rs"]
mod test_integrity;

#[path = "../../src/harness/symbol_exists.rs"]
mod symbol_exists;

#[path = "../../src/harness/ratchet.rs"]
mod ratchet;

#[path = "../../src/check_ring_layering.rs"]
mod check_ring_layering;

#[path = "../../src/panic_inventory.rs"]
mod panic_inventory;

/// Helper to create an isolated temporary Git repository for canary testing.
fn create_temp_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let path = temp_dir.path();

    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(path)
            .args(args)
            .status()
            .expect("failed to execute git command");
        assert!(status.success(), "git command failed: {:?}", args);
    };

    run_git(&["init", "-b", "main"]);
    run_git(&["config", "user.name", "Canary Tester"]);
    run_git(&["config", "user.email", "canary@example.com"]);

    temp_dir
}

/// Helper to commit all changes in the temporary repository.
fn git_add_commit(root: &Path, msg: &str) -> String {
    let run_git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(root)
            .args(args)
            .status()
            .expect("failed to execute git command");
        assert!(status.success(), "git command failed: {:?}", args);
    };

    run_git(&["add", "."]);
    run_git(&["commit", "-m", msg]);

    let output = Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("git rev-parse HEAD failed");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// 1. Gate: protected-paths
#[test]
fn test_canary_protected_paths() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("governance")).unwrap();
    fs::create_dir_all(root.join("docs/decisions")).unwrap();
    fs::create_dir_all(root.join("xtask/src/harness")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join("governance/protected-paths.toml"),
        r#"
[[protected]]
glob = "governance/**"
reason = "Governance"

[[protected]]
glob = "xtask/src/harness/protected_paths.rs"
reason = "Self protection"

[[protected]]
glob = "AGENTS.md"
reason = "Agents"
"#,
    )
    .unwrap();

    fs::write(root.join("AGENTS.md"), "agents v1\n").unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn hello() {}\n").unwrap();
    fs::write(
        root.join("docs/decisions/ADR-001-test.md"),
        "# ADR 001\n\n* **Status**: Accepted\nApproved\n",
    )
    .unwrap();

    let base_rev = git_add_commit(root, "Initial commit");

    // Defect Fixture: Modify protected file AGENTS.md without ADR trailer
    fs::write(root.join("AGENTS.md"), "agents v2 defect\n").unwrap();
    let _ = git_add_commit(root, "chore: modify agents without trailer");

    let root_str = root.to_str().unwrap().to_string();
    let code_defect = protected_paths::run_protected_paths(&[
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        base_rev,
    ]);
    assert_ne!(
        code_defect, 0,
        "protected-paths canary MUST fail when protected file is changed without trailer"
    );

    // Counter Fixture: Non-protected change passes
    let temp_dir_clean = create_temp_repo();
    let root_clean = temp_dir_clean.path();
    fs::create_dir_all(root_clean.join("governance")).unwrap();
    fs::create_dir_all(root_clean.join("docs/decisions")).unwrap();
    fs::create_dir_all(root_clean.join("xtask/src/harness")).unwrap();
    fs::create_dir_all(root_clean.join("src")).unwrap();
    fs::write(
        root_clean.join("governance/protected-paths.toml"),
        r#"
[[protected]]
glob = "governance/**"
reason = "Governance"

[[protected]]
glob = "xtask/src/harness/protected_paths.rs"
reason = "Self protection"

[[protected]]
glob = "AGENTS.md"
reason = "Agents"
"#,
    )
    .unwrap();
    fs::write(root_clean.join("AGENTS.md"), "agents v1\n").unwrap();
    fs::write(root_clean.join("src/lib.rs"), "pub fn hello() {}\n").unwrap();
    fs::write(
        root_clean.join("docs/decisions/ADR-001-test.md"),
        "# ADR 001\n\n* **Status**: Accepted\nApproved\n",
    )
    .unwrap();

    let base_clean = git_add_commit(root_clean, "Initial commit");

    fs::write(
        root_clean.join("src/lib.rs"),
        "pub fn hello() { println!(\"hi\"); }\n",
    )
    .unwrap();
    let _ = git_add_commit(root_clean, "feat: update safe file");

    let clean_str = root_clean.to_str().unwrap().to_string();
    let code_clean = protected_paths::run_protected_paths(&[
        "--root".to_string(),
        clean_str,
        "--base".to_string(),
        base_clean,
    ]);
    assert_eq!(
        code_clean, 0,
        "protected-paths canary MUST pass when non-protected file is changed"
    );
}

// 2. Gate: gate-weakening
#[test]
fn test_canary_gate_weakening() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn foo() {}\n").unwrap();
    let base_rev = git_add_commit(root, "Initial commit");

    // Defect Fixture: Add #[allow(unused)] in Rust code
    fs::write(
        root.join("src/lib.rs"),
        "#[allow(unused)]\npub fn foo() {}\n",
    )
    .unwrap();
    let _ = git_add_commit(root, "chore: add allow attribute");

    let root_str = root.to_str().unwrap().to_string();
    let code_defect = gate_weakening::run_gate_weakening(&[
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        base_rev,
        "--head".to_string(),
        "HEAD".to_string(),
    ]);
    assert_ne!(
        code_defect, 0,
        "gate-weakening canary MUST fail when #[allow(...)] is introduced"
    );

    // Counter Fixture: Clean code change passes
    let temp_dir_clean = create_temp_repo();
    let root_clean = temp_dir_clean.path();
    fs::create_dir_all(root_clean.join("src")).unwrap();
    fs::write(root_clean.join("src/lib.rs"), "pub fn foo() {}\n").unwrap();
    let base_clean = git_add_commit(root_clean, "Initial commit");

    fs::write(
        root_clean.join("src/lib.rs"),
        "pub fn foo() {}\npub fn bar() -> i32 { 42 }\n",
    )
    .unwrap();
    let _ = git_add_commit(root_clean, "feat: add clean function");

    let clean_str = root_clean.to_str().unwrap().to_string();
    let code_clean = gate_weakening::run_gate_weakening(&[
        "--root".to_string(),
        clean_str,
        "--base".to_string(),
        base_clean,
        "--head".to_string(),
        "HEAD".to_string(),
    ]);
    assert_eq!(
        code_clean, 0,
        "gate-weakening canary MUST pass on clean diffs"
    );
}

// 3. Gate: determinism-check
#[test]
fn test_canary_determinism_check() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("crates/contextra-core/src")).unwrap();
    fs::write(
        root.join("capabilities.toml"),
        r#"
[crates.contextra-core]
ring = "Ring 0"
path = "crates/contextra-core"
"#,
    )
    .unwrap();

    // Defect Fixture: Non-deterministic SystemTime::now() call in Ring 0 code
    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "pub fn get_time() {\n    let _ = std::time::SystemTime::now();\n}\n",
    )
    .unwrap();

    let root_str = root.to_str().unwrap().to_string();
    let code_defect =
        determinism_check::run_determinism_check(&["--root".to_string(), root_str.clone()]);
    assert_ne!(
        code_defect, 0,
        "determinism-check canary MUST fail when SystemTime::now() is called in Ring 0"
    );

    // Counter Fixture: Deterministic code passes
    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "pub fn get_value() -> u32 {\n    42\n}\n",
    )
    .unwrap();

    let code_clean = determinism_check::run_determinism_check(&["--root".to_string(), root_str]);
    assert_eq!(
        code_clean, 0,
        "determinism-check canary MUST pass on deterministic code"
    );
}

// 4. Gate: check-unsafe-islands
#[test]
fn test_canary_check_unsafe_islands() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("crates/contextra-core/src")).unwrap();

    // Defect Fixture: unsafe keyword in non-island crate
    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "#![forbid(unsafe_code)]\npub unsafe fn bad_unsafe() {}\n",
    )
    .unwrap();

    let res_defect = check_unsafe_islands::run_check_unsafe_islands_at(root, true).unwrap();
    assert!(
        !res_defect.errors.is_empty(),
        "check-unsafe-islands canary MUST produce errors when unsafe is in non-island crate"
    );

    // Counter Fixture: Safe non-island crate and unsafe in permitted island crate (contextra-sys)
    fs::create_dir_all(root.join("crates/contextra-sys/src")).unwrap();
    fs::write(
        root.join("crates/contextra-sys/src/lib.rs"),
        "pub unsafe fn island_unsafe() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "#![forbid(unsafe_code)]\npub fn safe_fn() {}\n",
    )
    .unwrap();

    let res_clean = check_unsafe_islands::run_check_unsafe_islands_at(root, true).unwrap();
    assert!(
        res_clean.errors.is_empty(),
        "check-unsafe-islands canary MUST pass when unsafe is isolated to islands"
    );
}

// 5. Gate: scope-guard
#[test]
fn test_canary_scope_guard() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    fs::create_dir_all(root.join(".jules/tasks")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(
        root.join(".jules/tasks/task-001.toml"),
        "scope = [\"src/in_scope.rs\"]\nforbidden = [\"xtask/**\"]\n",
    )
    .unwrap();

    fs::write(root.join("src/in_scope.rs"), "pub fn ok() {}\n").unwrap();
    let base_rev = git_add_commit(root, "Initial commit");

    // Defect Fixture: Modify out-of-scope file
    fs::write(root.join("src/out_of_scope.rs"), "pub fn bad() {}\n").unwrap();
    let _ = git_add_commit(root, "feat: out of scope file");

    let root_str = root.to_str().unwrap().to_string();
    let code_defect = scope_guard::run_scope_guard(&[
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        base_rev,
        "--head".to_string(),
        "HEAD".to_string(),
        "--card".to_string(),
        ".jules/tasks/task-001.toml".to_string(),
    ]);
    assert_ne!(
        code_defect, 0,
        "scope-guard canary MUST fail when an out-of-scope file is touched"
    );

    // Counter Fixture: Modify only in-scope file
    let temp_dir_clean = create_temp_repo();
    let root_clean = temp_dir_clean.path();
    fs::create_dir_all(root_clean.join(".jules/tasks")).unwrap();
    fs::create_dir_all(root_clean.join("src")).unwrap();

    fs::write(
        root_clean.join(".jules/tasks/task-001.toml"),
        "scope = [\"src/in_scope.rs\"]\nforbidden = [\"xtask/**\"]\n",
    )
    .unwrap();
    fs::write(root_clean.join("src/in_scope.rs"), "pub fn ok() {}\n").unwrap();
    let base_clean = git_add_commit(root_clean, "Initial commit");

    fs::write(
        root_clean.join("src/in_scope.rs"),
        "pub fn ok() { println!(\"ok\"); }\n",
    )
    .unwrap();
    let _ = git_add_commit(root_clean, "feat: in scope edit");

    let clean_str = root_clean.to_str().unwrap().to_string();
    let code_clean = scope_guard::run_scope_guard(&[
        "--root".to_string(),
        clean_str,
        "--base".to_string(),
        base_clean,
        "--head".to_string(),
        "HEAD".to_string(),
        "--card".to_string(),
        ".jules/tasks/task-001.toml".to_string(),
    ]);
    assert_eq!(
        code_clean, 0,
        "scope-guard canary MUST pass when changes are within scope"
    );
}

// 6. Gate: test-integrity
#[test]
fn test_canary_test_integrity() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(
        root.join("tests/dummy_test.rs"),
        "#[test]\nfn test_valid() {\n    assert_eq!(1, 1);\n}\n",
    )
    .unwrap();
    let base_rev = git_add_commit(root, "Initial commit");

    // Defect Fixture: Delete test file without replacement
    let status = Command::new("git")
        .current_dir(root)
        .args(["rm", "tests/dummy_test.rs"])
        .status()
        .unwrap();
    assert!(status.success());
    let _ = git_add_commit(root, "chore: delete test file");

    let root_str = root.to_str().unwrap().to_string();
    let code_defect = test_integrity::run_test_integrity(&[
        "--root".to_string(),
        root_str,
        "--base".to_string(),
        base_rev,
        "--head".to_string(),
        "HEAD".to_string(),
    ]);
    assert_ne!(
        code_defect, 0,
        "test-integrity canary MUST fail when a test file is deleted without replacement"
    );

    // Counter Fixture: Add a valid test file with assertions
    let temp_dir_clean = create_temp_repo();
    let root_clean = temp_dir_clean.path();
    fs::create_dir_all(root_clean.join("tests")).unwrap();
    fs::write(
        root_clean.join("tests/dummy_test.rs"),
        "#[test]\nfn test_valid() {\n    assert_eq!(1, 1);\n}\n",
    )
    .unwrap();
    let base_clean = git_add_commit(root_clean, "Initial commit");

    fs::write(
        root_clean.join("tests/new_test.rs"),
        "#[test]\nfn test_another() {\n    assert_eq!(2, 2);\n}\n",
    )
    .unwrap();
    let _ = git_add_commit(root_clean, "feat: add valid test");

    let clean_str = root_clean.to_str().unwrap().to_string();
    let code_clean = test_integrity::run_test_integrity(&[
        "--root".to_string(),
        clean_str,
        "--base".to_string(),
        base_clean,
        "--head".to_string(),
        "HEAD".to_string(),
    ]);
    assert_eq!(
        code_clean, 0,
        "test-integrity canary MUST pass when valid tests are added"
    );
}

// 7. Gate: symbol-exists
#[test]
fn test_canary_symbol_exists() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("crates/contextra-core/src")).unwrap();
    fs::write(
        root.join("capabilities.toml"),
        r#"
[crates.contextra-core]
ring = "Ring 0"
path = "crates/contextra-core"
"#,
    )
    .unwrap();

    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "pub struct RealStruct;\n",
    )
    .unwrap();

    let root_str = root.to_str().unwrap().to_string();

    // Defect Fixture: Lookup invented non-existent symbol
    let code_defect = symbol_exists::run_symbol_exists(&[
        "--root".to_string(),
        root_str.clone(),
        "contextra-core::InventedSymbolDoesNotExist".to_string(),
    ]);
    assert_ne!(
        code_defect, 0,
        "symbol-exists canary MUST fail for invented/non-existent symbols"
    );

    // Counter Fixture: Lookup real existing symbol
    let code_clean = symbol_exists::run_symbol_exists(&[
        "--root".to_string(),
        root_str,
        "contextra-core::RealStruct".to_string(),
    ]);
    assert_eq!(
        code_clean, 0,
        "symbol-exists canary MUST pass for existing valid symbols"
    );
}

// 8. Gate: ratchet
#[test]
fn test_canary_ratchet() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    fs::create_dir_all(root.join("governance")).unwrap();
    fs::create_dir_all(root.join(".github")).unwrap();
    fs::create_dir_all(root.join("crates/contextra-core/src")).unwrap();

    fs::write(
        root.join("governance/ratchet.toml"),
        r#"
unwrap_legacy_grep = 0
unwrap_exact = 0
expect_exact = 0
allow_attrs = 0
ignore_attrs = 0
forbid_unsafe_crates = 1
[unsafe_blocks]
contextra-sys = 0
"#,
    )
    .unwrap();

    fs::write(root.join(".github/unwrap_baseline.txt"), "0\n").unwrap();
    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "#![forbid(unsafe_code)]\npub fn clean() {}\n",
    )
    .unwrap();

    let root_str = root.to_str().unwrap().to_string();

    // Defect Fixture: Introduce #[allow(...)] attribute exceeding ratchet baseline
    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "#![forbid(unsafe_code)]\n#[allow(unused)]\npub fn degraded() {}\n",
    )
    .unwrap();

    let code_defect =
        ratchet::run_ratchet(&["--root".to_string(), root_str.clone(), "check".to_string()]);
    assert_ne!(
        code_defect, 0,
        "ratchet canary MUST fail when metrics exceed ratchet baseline"
    );

    // Counter Fixture: Clean state within baseline
    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "#![forbid(unsafe_code)]\npub fn clean() {}\n",
    )
    .unwrap();

    let code_clean = ratchet::run_ratchet(&["--root".to_string(), root_str, "check".to_string()]);
    assert_eq!(
        code_clean, 0,
        "ratchet canary MUST pass when metrics are within baseline"
    );
}

// 9. Gate: check-dag (check-ring-layering)
#[test]
fn test_canary_check_dag_ring_layering() {
    // Defect Fixture: Ring 0 backward edge (contextra-types order 0 depending on contextra-graph order 3)
    let mock_json_defect = r#"{
        "packages": [
            {
                "name": "contextra-types",
                "dependencies": [
                    { "name": "contextra-graph", "kind": null }
                ],
                "metadata": { "contextra": { "ring": "0" } }
            },
            {
                "name": "contextra-graph",
                "dependencies": [],
                "metadata": { "contextra": { "ring": "0" } }
            }
        ],
        "workspace_members": ["contextra-types", "contextra-graph"]
    }"#;

    let res_defect = check_ring_layering::check_ring_layering_from_metadata_json(mock_json_defect)
        .expect("metadata json parsing failed");
    let active_violations_defect: Vec<_> = res_defect
        .into_iter()
        .filter(|v| !v.is_stale_allowlist)
        .collect();

    assert!(
        !active_violations_defect.is_empty(),
        "check-dag canary MUST detect backward edge / DAG cycle violations"
    );

    // Counter Fixture: Valid downward dependency edge (Ring 1 contextra-store -> Ring 0 contextra-core)
    let mock_json_clean = r#"{
        "packages": [
            {
                "name": "contextra-core",
                "dependencies": [],
                "metadata": { "contextra": { "ring": "0" } }
            },
            {
                "name": "contextra-store",
                "dependencies": [
                    { "name": "contextra-core", "kind": null }
                ],
                "metadata": { "contextra": { "ring": "1" } }
            }
        ],
        "workspace_members": ["contextra-core", "contextra-store"]
    }"#;

    let res_clean = check_ring_layering::check_ring_layering_from_metadata_json(mock_json_clean)
        .expect("metadata json parsing failed");
    let active_violations_clean: Vec<_> = res_clean
        .into_iter()
        .filter(|v| !v.is_stale_allowlist)
        .collect();

    assert!(
        active_violations_clean.is_empty(),
        "check-dag canary MUST pass for valid DAG dependency structure"
    );
}

// 10. Gate: panic-inventory (Ring 0 Zero-Panic unwrap)
#[test]
fn test_canary_panic_inventory_ring0_unwrap() {
    let temp_dir = create_temp_repo();
    let root = temp_dir.path();

    fs::create_dir_all(root.join(".github")).unwrap();
    fs::create_dir_all(root.join("crates/contextra-core/src")).unwrap();

    fs::write(root.join(".github/unwrap_baseline.txt"), "# empty\n").unwrap();

    // Defect Fixture: Introduce unwrap() in production code
    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "pub fn bad_panic() {\n    let _ = Some(10).unwrap();\n}\n",
    )
    .unwrap();

    let res_defect = panic_inventory::run_panic_inventory_in_root(root, None, true, true);
    assert!(
        res_defect.is_err(),
        "panic-inventory canary MUST fail when unwrap() is introduced in production code"
    );

    // Counter Fixture: Pure Result-based error propagation without unwrap
    fs::write(
        root.join("crates/contextra-core/src/lib.rs"),
        "pub fn safe_function() -> Result<i32, &'static str> {\n    Ok(10)\n}\n",
    )
    .unwrap();

    let res_clean = panic_inventory::run_panic_inventory_in_root(root, None, true, true);
    assert!(
        res_clean.is_ok(),
        "panic-inventory canary MUST pass on clean code with Result handling"
    );
    let entries = res_clean.unwrap();
    let non_test_panics = entries.iter().filter(|e| !e.is_test_code).count();
    assert_eq!(
        non_test_panics, 0,
        "panic-inventory canary MUST find 0 non-test panic entries"
    );
}
