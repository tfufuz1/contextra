pub use xtask::check_unsafe_islands;
pub use xtask::panic_inventory;

#[path = "../src/harness/doctrine_scan.rs"]
mod doctrine_scan;

use std::fs;
use tempfile::TempDir;

#[test]
fn test_allow_override_detected() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let caps = r#"
[crates.test-crate]
ring = "Ring 0"
maturity = "experimental"
path = "crates/test-crate"
"#;
    fs::write(root.join("capabilities.toml"), caps).unwrap();

    let crate_src = root.join("crates/test-crate/src");
    fs::create_dir_all(&crate_src).unwrap();

    let code_content = r#"#![forbid(unsafe_code)]

#[allow(clippy::unwrap_used)]
pub fn foo() {
    let x: Option<i32> = Some(1);
    let _ = x;
}
"#;
    fs::write(crate_src.join("lib.rs"), code_content).unwrap();

    let args = vec![
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--crate".to_string(),
        "test-crate".to_string(),
        "--json".to_string(),
    ];

    let code = doctrine_scan::run_doctrine_scan(&args);
    assert_eq!(code, 0);
}

#[test]
fn test_status_contradiction_detected() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let caps = r#"
[crates.completed-crate]
ring = "Ring 0"
status = "fertig"
path = "crates/completed-crate"
"#;
    fs::write(root.join("capabilities.toml"), caps).unwrap();

    let crate_src = root.join("crates/completed-crate/src");
    fs::create_dir_all(&crate_src).unwrap();

    let code_content = r#"#![forbid(unsafe_code)]

pub fn do_something() {
    let x: Option<i32> = Some(42);
    let _val = x.unwrap();
}
"#;
    fs::write(crate_src.join("lib.rs"), code_content).unwrap();

    let args = vec![
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--crate".to_string(),
        "completed-crate".to_string(),
        "--json".to_string(),
    ];

    let code = doctrine_scan::run_doctrine_scan(&args);
    assert_eq!(
        code, 1,
        "Crate with 'fertig' status and .unwrap() must fail doctrine scan"
    );
}
