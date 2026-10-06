// xtask/tests/orphan_symbols.rs

use std::fs;
use tempfile::tempdir;
use xtask::harness::orphan_symbols;

#[test]
fn test_fixture_orphan_symbol() {
    let dir = tempdir().expect("Failed to create temp dir");
    let root = dir.path();

    let src_dir = root.join("crates/a/src");
    fs::create_dir_all(&src_dir).expect("Failed to create dir");

    let lib_code = r#"
pub fn foo() {
    println!("I am unreferenced");
}
"#;
    fs::write(src_dir.join("lib.rs"), lib_code).expect("Failed to write lib.rs");

    let args = vec![
        "orphan-symbols".to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--exclude-crate".to_string(),
        "none".to_string(),
        "--json".to_string(),
    ];

    let code = orphan_symbols::run_orphan_symbols(&args);
    assert_eq!(code, 2, "Status must be 'fail' (exit code 2) for orphan symbol");
}

#[test]
fn test_fixture_nur_tests_symbol() {
    let dir = tempdir().expect("Failed to create temp dir");
    let root = dir.path();

    let src_dir = root.join("crates/a/src");
    fs::create_dir_all(&src_dir).expect("Failed to create dir");

    let lib_code = r#"
pub fn bar() {
    println!("I am used only in tests");
}
"#;
    fs::write(src_dir.join("lib.rs"), lib_code).expect("Failed to write lib.rs");

    let tests_code = r#"
#[test]
fn test_bar() {
    bar();
}
"#;
    fs::write(src_dir.join("tests.rs"), tests_code).expect("Failed to write tests.rs");

    let args = vec![
        "orphan-symbols".to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--exclude-crate".to_string(),
        "none".to_string(),
        "--json".to_string(),
    ];

    let code = orphan_symbols::run_orphan_symbols(&args);
    assert_eq!(code, 0, "Status must be 'pass' (exit code 0) when symbol is used in tests");
}

#[test]
fn test_fixture_text_ref_python() {
    let dir = tempdir().expect("Failed to create temp dir");
    let root = dir.path();

    let src_dir = root.join("crates/a/src");
    let tests_dir = root.join("crates/a/tests");
    fs::create_dir_all(&src_dir).expect("Failed to create src dir");
    fs::create_dir_all(&tests_dir).expect("Failed to create tests dir");

    let lib_code = r#"
pub fn baz() {
    println!("I am referenced in python");
}
"#;
    fs::write(src_dir.join("lib.rs"), lib_code).expect("Failed to write lib.rs");

    let py_code = r#"
# Calling baz from Python bindings
baz()
"#;
    fs::write(tests_dir.join("x.py"), py_code).expect("Failed to write x.py");

    let args = vec![
        "orphan-symbols".to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--exclude-crate".to_string(),
        "none".to_string(),
        "--json".to_string(),
    ];

    let code = orphan_symbols::run_orphan_symbols(&args);
    assert_eq!(code, 0, "Symbol referenced in Python text is not ORPHAN");
}

#[test]
fn test_fixture_macro_rules() {
    let dir = tempdir().expect("Failed to create temp dir");
    let root = dir.path();

    let src_dir = root.join("crates/a/src");
    fs::create_dir_all(&src_dir).expect("Failed to create dir");

    let lib_code = r#"
macro_rules! my_macro {
    () => {
        pub fn m() {}
    };
}
"#;
    fs::write(src_dir.join("lib.rs"), lib_code).expect("Failed to write lib.rs");

    let out_file = root.join("report.md");

    let args = vec![
        "orphan-symbols".to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--exclude-crate".to_string(),
        "none".to_string(),
        "--out-file".to_string(),
        out_file.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    let code = orphan_symbols::run_orphan_symbols(&args);
    assert_eq!(code, 0, "Symbol in macro_rules is excluded from declarations");

    let report_content = fs::read_to_string(&out_file).expect("Failed to read report file");
    assert!(report_content.contains("## Zusammenfassung"));
    assert!(report_content.contains("## ORPHAN"));
    assert!(report_content.contains("## NUR_TESTS"));
    assert!(report_content.contains("## VERDACHT"));
    assert!(report_content.contains("## Hinweis"));
}

#[test]
fn test_fixture_pub_use_reexport() {
    let dir = tempdir().expect("Failed to create temp dir");
    let root = dir.path();

    let src_dir = root.join("crates/a/src");
    fs::create_dir_all(&src_dir).expect("Failed to create dir");

    let sub_code = r#"
pub fn q() {}
"#;
    fs::write(src_dir.join("sub.rs"), sub_code).expect("Failed to write sub.rs");

    let lib_code = r#"
mod sub;
pub use sub::q;
"#;
    fs::write(src_dir.join("lib.rs"), lib_code).expect("Failed to write lib.rs");

    let args = vec![
        "orphan-symbols".to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--exclude-crate".to_string(),
        "none".to_string(),
        "--json".to_string(),
    ];

    let code = orphan_symbols::run_orphan_symbols(&args);
    assert_eq!(code, 0, "Symbol q re-exported via pub use is not ORPHAN");
}

#[test]
fn test_fixture_excluded_boundary_crate() {
    let dir = tempdir().expect("Failed to create temp dir");
    let root = dir.path();

    let src_dir = root.join("crates/contextra/src");
    fs::create_dir_all(&src_dir).expect("Failed to create dir");

    let lib_code = r#"
pub fn boundary_unused_fn() {}
"#;
    fs::write(src_dir.join("lib.rs"), lib_code).expect("Failed to write lib.rs");

    let args = vec![
        "orphan-symbols".to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--exclude-crate".to_string(),
        "contextra".to_string(),
        "--json".to_string(),
    ];

    let code = orphan_symbols::run_orphan_symbols(&args);
    assert_eq!(code, 0, "Excluded boundary crate orphan does not trigger exit 2");
}
