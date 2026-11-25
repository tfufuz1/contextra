#[path = "../src/harness/symbol_exists.rs"]
mod symbol_exists;

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn setup_temp_workspace() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    let p = dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(p)
        .output()
        .expect("git init failed");

    fs::write(
        p.join("capabilities.toml"),
        r#"
[crates.crate_a]
path = "crates/crate-a"

[crates.crate_b]
path = "crates/crate-b"
"#,
    )
    .unwrap();

    let crate_a = p.join("crates/crate-a/src");
    fs::create_dir_all(&crate_a).unwrap();
    fs::write(
        crate_a.join("lib.rs"),
        r#"
pub mod submod;
pub use submod::MyStruct;

pub trait MyTrait {
    fn trait_method(&self);
}
"#,
    )
    .unwrap();

    fs::write(
        crate_a.join("submod.rs"),
        r#"
pub struct MyStruct {
    pub field: u32,
}

impl MyStruct {
    pub fn new() -> Self {
        Self { field: 0 }
    }
}
"#,
    )
    .unwrap();

    dir
}

#[test]
fn test_symbol_exists_found_struct_and_method() {
    let dir = setup_temp_workspace();
    let p = dir.path();

    let root_arg = format!("{}", p.display());
    let args1 = vec![
        "--root".to_string(),
        root_arg.clone(),
        "crate_a::submod::MyStruct".to_string(),
    ];
    let code1 = symbol_exists::run_symbol_exists(&args1);
    assert_eq!(code1, 0);

    let args2 = vec![
        "--root".to_string(),
        root_arg,
        "crate_a::submod::MyStruct::new".to_string(),
    ];
    let code2 = symbol_exists::run_symbol_exists(&args2);
    assert_eq!(code2, 0);
}

#[test]
fn test_symbol_exists_reexport() {
    let dir = setup_temp_workspace();
    let p = dir.path();

    let root_arg = format!("{}", p.display());
    let args = vec![
        "--root".to_string(),
        root_arg,
        "crate_a::MyStruct".to_string(),
    ];
    let code = symbol_exists::run_symbol_exists(&args);
    assert_eq!(code, 0);
}

#[test]
fn test_symbol_exists_not_found_with_suggestions() {
    let dir = setup_temp_workspace();
    let p = dir.path();

    let root_arg = format!("{}", p.display());
    let args = vec![
        "--root".to_string(),
        root_arg,
        "crate_a::submod::MyStrukt".to_string(),
    ];
    let code = symbol_exists::run_symbol_exists(&args);
    assert_eq!(code, 1);
}
