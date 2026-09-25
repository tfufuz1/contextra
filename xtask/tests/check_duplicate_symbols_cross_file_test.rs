#[path = "../src/check_duplicate_symbols_cross_file.rs"]
mod check_duplicate_symbols_cross_file;

use check_duplicate_symbols_cross_file::{
    scan_directory_for_cross_file_duplicates, scan_workspace_for_cross_file_duplicates,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_same_directory_duplicate_trait_detected() {
    let temp = tempdir().unwrap();
    let dir = temp.path().join("my_module");
    fs::create_dir_all(&dir).unwrap();

    let file1 = dir.join("a.rs");
    let file2 = dir.join("b.rs");

    fs::write(
        &file1,
        r#"
pub trait Foo {
    fn bar(&self);
}
"#,
    )
    .unwrap();

    fs::write(
        &file2,
        r#"
pub trait Foo {
    fn baz(&self);
}
"#,
    )
    .unwrap();

    let dups = scan_directory_for_cross_file_duplicates(&dir).unwrap();
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0].symbol_kind, "trait");
    assert_eq!(dups[0].symbol_name, "Foo");
    assert_eq!(dups[0].file1, file1.to_string_lossy());
    assert_eq!(dups[0].line1, 2);
    assert_eq!(dups[0].file2, file2.to_string_lossy());
    assert_eq!(dups[0].line2, 2);
}

#[test]
fn test_different_directories_same_struct_not_flagged() {
    let temp = tempdir().unwrap();
    let dir1 = temp.path().join("mod1");
    let dir2 = temp.path().join("mod2");
    fs::create_dir_all(&dir1).unwrap();
    fs::create_dir_all(&dir2).unwrap();

    fs::write(
        dir1.join("a.rs"),
        r#"
pub struct Bar;
"#,
    )
    .unwrap();

    fs::write(
        dir2.join("b.rs"),
        r#"
pub struct Bar;
"#,
    )
    .unwrap();

    let roots = [dir1.as_path(), dir2.as_path()];
    let dups = scan_workspace_for_cross_file_duplicates(&roots).unwrap();
    assert!(
        dups.is_empty(),
        "Expected no duplicates across different directories, got: {:?}",
        dups
    );
}

#[test]
fn test_no_duplicates_succeeds() {
    let temp = tempdir().unwrap();
    let dir = temp.path().join("clean_module");
    fs::create_dir_all(&dir).unwrap();

    fs::write(
        dir.join("a.rs"),
        r#"
pub struct Alpha;
pub trait Beta {}
"#,
    )
    .unwrap();

    fs::write(
        dir.join("b.rs"),
        r#"
pub struct Gamma;
pub fn delta() {}
"#,
    )
    .unwrap();

    let dups = scan_directory_for_cross_file_duplicates(&dir).unwrap();
    assert!(
        dups.is_empty(),
        "Expected 0 duplicates in clean module, got: {:?}",
        dups
    );
}
