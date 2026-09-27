//! Tests for panic_inventory module.

#[path = "../src/panic_inventory.rs"]
mod panic_inventory;

use panic_inventory::{run_panic_inventory_in_root, PanicKind};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_panic_inventory_synthetic_fixture() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();

    let crates_dir = root.join("crates").join("my-test-crate").join("src");
    fs::create_dir_all(&crates_dir).unwrap();

    let lib_rs_content = r#"
pub fn add(a: i32, b: i32) -> i32 {
    let opt: Option<i32> = Some(a + b);
    let val = opt.unwrap(); // line 4
    val
}

pub fn bad_func() {
    panic!("error"); // line 9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_file() {
        let opt: Option<i32> = None;
        let _ = opt.expect("should fail"); // line 18
    }
}
"#;

    fs::write(crates_dir.join("lib.rs"), lib_rs_content).unwrap();

    let baseline_dir = root.join(".github");
    fs::create_dir_all(&baseline_dir).unwrap();
    fs::write(baseline_dir.join("unwrap_baseline.txt"), "").unwrap();

    let entries = run_panic_inventory_in_root(root, None, false, false).unwrap();

    assert_eq!(entries.len(), 3);

    let unwrap_entry = entries
        .iter()
        .find(|e| e.kind == PanicKind::Unwrap)
        .unwrap();
    assert_eq!(unwrap_entry.line, 4);
    assert!(!unwrap_entry.is_test_code);

    let panic_entry = entries.iter().find(|e| e.kind == PanicKind::Panic).unwrap();
    assert_eq!(panic_entry.line, 9);
    assert!(!panic_entry.is_test_code);

    let expect_entry = entries
        .iter()
        .find(|e| e.kind == PanicKind::Expect)
        .unwrap();
    assert_eq!(expect_entry.line, 18);
    assert!(expect_entry.is_test_code);
}

#[test]
fn test_panic_inventory_strict_mode_baseline_matching() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();

    let crates_dir = root.join("crates").join("my-test-crate").join("src");
    fs::create_dir_all(&crates_dir).unwrap();

    let lib_rs_content = r#"
pub fn add(a: i32, b: i32) -> i32 {
    let opt: Option<i32> = Some(a + b);
    let val = opt.unwrap(); // line 4
    val
}
"#;
    fs::write(crates_dir.join("lib.rs"), lib_rs_content).unwrap();

    let baseline_dir = root.join(".github");
    fs::create_dir_all(&baseline_dir).unwrap();

    // Case 1: line 4 is in baseline -> strict mode passes
    fs::write(
        baseline_dir.join("unwrap_baseline.txt"),
        "crates/my-test-crate/src/lib.rs:4\n",
    )
    .unwrap();

    let res = run_panic_inventory_in_root(root, None, false, true);
    assert!(res.is_ok());

    // Case 2: baseline empty -> strict mode fails
    fs::write(baseline_dir.join("unwrap_baseline.txt"), "").unwrap();
    let res = run_panic_inventory_in_root(root, None, false, true);
    assert!(res.is_err());
}
