// xtask/tests/harness_orphan_symbols_test.rs

use std::fs;
use tempfile::tempdir;
use xtask::harness::orphan_symbols;

#[test]
fn test_orphan_symbols_unreferenced_function() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let crate_src = root.join("crates/testcrate/src");
    fs::create_dir_all(&crate_src).unwrap();

    let lib_code = r#"
pub fn never_called() {
    println!("I am never called");
}
"#;
    fs::write(crate_src.join("lib.rs"), lib_code).unwrap();

    let args = vec![
        "orphan-symbols".to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--exclude-crate".to_string(),
        "none".to_string(),
        "--json".to_string(),
    ];

    let code = orphan_symbols::run_orphan_symbols(&args);

    assert_eq!(
        code, 2,
        "Status must be 'fail' (exit code 2) for orphan symbol"
    );
}

#[test]
fn test_orphan_symbols_referenced_function() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let crate_src = root.join("crates/testcrate/src");
    fs::create_dir_all(&crate_src).unwrap();

    let lib_code = r#"
pub fn never_called() {
    println!("I am called in other.rs");
}
"#;
    fs::write(crate_src.join("lib.rs"), lib_code).unwrap();

    let other_code = r#"
fn caller() {
    never_called();
}
"#;
    fs::write(crate_src.join("other.rs"), other_code).unwrap();

    let args = vec![
        "orphan-symbols".to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--exclude-crate".to_string(),
        "none".to_string(),
        "--json".to_string(),
    ];

    let code = orphan_symbols::run_orphan_symbols(&args);

    assert_eq!(
        code, 0,
        "Status must be 'pass' (exit code 0) when symbol is referenced"
    );
}
