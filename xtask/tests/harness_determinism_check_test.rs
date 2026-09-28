#[path = "../src/harness/determinism_check.rs"]
mod determinism_check;

use std::fs;
use tempfile::TempDir;

#[test]
fn test_determinism_check_harness() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Create mock git repo
    let _ = std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output();

    let caps = r#"
[crates.contextra-simd]
ring = "Ring 0"
path = "crates/contextra-simd"

[crates.contextra-store]
ring = "Ring 1"
path = "crates/contextra-store"
"#;
    fs::write(root.join("capabilities.toml"), caps).unwrap();

    let crate_dir = root.join("crates/contextra-store/src");
    fs::create_dir_all(&crate_dir).unwrap();

    fs::write(crate_dir.join("clean.rs"), "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();
    fs::write(crate_dir.join("dirty.rs"), "pub fn now() { let _ = std::time::SystemTime::now(); }\n").unwrap();

    let args = vec![
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    let exit_code = determinism_check::run_determinism_check(&args);
    assert_eq!(exit_code, 1);

    // Verify all pub functions are exercised
    let (root_path, json_flag, write_baseline, dynamic_crate) =
        determinism_check::determinism_check_parse_args(&args);
    assert_eq!(root_path, root);
    assert!(json_flag);
    assert!(!write_baseline);
    assert_eq!(dynamic_crate, None);

    let crates = determinism_check::determinism_check_get_ring01_crates(root).unwrap();
    assert_eq!(crates.len(), 2);

    assert!(determinism_check::determinism_check_is_test_or_bench(
        std::path::Path::new("crates/contextra-store/tests/foo.rs"),
        "let x = 1;"
    ));

    let mut findings = vec![];
    determinism_check::determinism_check_scan_file(
        &crate_dir.join("dirty.rs"),
        "crates/contextra-store/src/dirty.rs",
        &mut findings,
    );
    assert!(!findings.is_empty());
}
