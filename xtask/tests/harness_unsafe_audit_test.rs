#[path = "../src/harness/unsafe_audit.rs"]
mod unsafe_audit;

use std::collections::BTreeMap;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_unsafe_audit_harness() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let caps = r#"
[crates.contextra-simd]
ring = "Ring 0"
path = "crates/contextra-simd"
unsafe_island = true

[crates.contextra-core]
ring = "Ring 0"
path = "crates/contextra-core"
unsafe_island = false
"#;
    fs::write(root.join("capabilities.toml"), caps).unwrap();

    let simd_dir = root.join("crates/contextra-simd/src");
    let core_dir = root.join("crates/contextra-core/src");
    fs::create_dir_all(&simd_dir).unwrap();
    fs::create_dir_all(&core_dir).unwrap();

    fs::write(
        simd_dir.join("lib.rs"),
        "// SAFETY: test safety comment\npub unsafe fn foo() {}\n",
    )
    .unwrap();
    fs::write(
        core_dir.join("lib.rs"),
        "#![forbid(unsafe_code)]\npub fn bar() {}\n",
    )
    .unwrap();

    // With baseline file created, scan file works
    fs::create_dir_all(root.join("governance")).unwrap();
    fs::write(
        root.join("governance/unsafe-safety-baseline.toml"),
        "[missing_safety_comments]\n",
    )
    .unwrap();

    let args = vec![
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    let (islands, crates) = unsafe_audit::unsafe_audit_get_islands_and_crates(root).unwrap();
    assert_eq!(islands, vec!["contextra-simd"]);
    assert_eq!(crates.len(), 2);

    let mut findings = vec![];
    let mut counts = BTreeMap::new();
    unsafe_audit::unsafe_audit_scan_file(
        &simd_dir.join("lib.rs"),
        "crates/contextra-simd/src/lib.rs",
        true,
        &mut findings,
        &mut counts,
    );
    assert_eq!(*counts.get("crates/contextra-simd/src/lib.rs").unwrap(), 1);
}

#[test]
fn test_unsafe_audit_missing_baseline_fails() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let caps = r#"
[crates.contextra-core]
ring = "Ring 0"
path = "crates/contextra-core"
unsafe_island = false
"#;
    fs::write(root.join("capabilities.toml"), caps).unwrap();

    let core_dir = root.join("crates/contextra-core/src");
    fs::create_dir_all(&core_dir).unwrap();
    fs::write(
        core_dir.join("lib.rs"),
        "#![forbid(unsafe_code)]\npub fn bar() {}\n",
    )
    .unwrap();

    let args = vec![
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    // Missing baseline file governance/unsafe-safety-baseline.toml must fail (non-zero exit code)
    let exit_code = unsafe_audit::run_unsafe_audit(&args);
    assert_ne!(exit_code, 0);
}
