#[path = "../src/check_ring_layering.rs"]
mod check_ring_layering;

#[path = "../src/harness/unwired_ports.rs"]
mod unwired_ports;

use std::fs;
use tempfile::TempDir;

fn setup_test_workspace() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    let root = dir.path();

    fs::write(
        root.join("capabilities.toml"),
        r#"
[crates.ring0_crate]
path = "crates/ring0_crate"

[crates.ring3_comp_root]
path = "crates/ring3_comp_root"
"#,
    )
    .unwrap();

    let ring0_dir = root.join("crates/ring0_crate");
    let ring0_src = ring0_dir.join("src");
    fs::create_dir_all(&ring0_src).unwrap();
    fs::write(
        ring0_dir.join("Cargo.toml"),
        r#"
[package]
name = "ring0_crate"
version = "0.1.0"
edition = "2021"

[package.metadata.contextra]
ring = "0"
"#,
    )
    .unwrap();

    fs::write(
        ring0_src.join("lib.rs"),
        r#"
pub trait UnwiredTrait {
    fn do_something(&self);
}

pub trait SoloTrait {
    fn solo_method(&self);
}

pub struct SoloType;

impl SoloTrait for SoloType {
    fn solo_method(&self) {}
}

pub trait OkTrait {
    fn ok_method(&self);
}

pub struct OkType;

impl OkTrait for OkType {
    fn ok_method(&self) {}
}
"#,
    )
    .unwrap();

    let ring3_dir = root.join("crates/ring3_comp_root");
    let ring3_src = ring3_dir.join("src");
    fs::create_dir_all(&ring3_src).unwrap();
    fs::write(
        ring3_dir.join("Cargo.toml"),
        r#"
[package]
name = "ring3_comp_root"
version = "0.1.0"
edition = "2021"

[package.metadata.contextra]
ring = "3"
"#,
    )
    .unwrap();

    fs::write(
        ring3_src.join("lib.rs"),
        r#"
// Composition Root references OkType
pub fn init() {
    let _x = OkType;
}
"#,
    )
    .unwrap();

    dir
}

#[test]
fn test_unwired_ports_unwired_classification() {
    let dir = setup_test_workspace();
    let root_str = dir.path().to_string_lossy().to_string();

    let args = vec![
        "--root".to_string(),
        root_str,
        "--json".to_string(),
        "--only-unwired".to_string(),
    ];

    let code = unwired_ports::run_unwired_ports(&args);
    assert_eq!(code, 2);
}

#[test]
fn test_unwired_ports_solo_and_ok_classifications() {
    let dir = setup_test_workspace();
    let root_str = dir.path().to_string_lossy().to_string();

    // Fix unwired trait in lib.rs to allow pass status or inspect json
    let ring0_lib = dir.path().join("crates/ring0_crate/src/lib.rs");
    fs::write(
        &ring0_lib,
        r#"
pub trait SoloTrait {
    fn solo_method(&self);
}

pub struct SoloType;

impl SoloTrait for SoloType {
    fn solo_method(&self) {}
}

pub trait OkTrait {
    fn ok_method(&self);
}

pub struct OkType;

impl OkTrait for OkType {
    fn ok_method(&self) {}
}
"#,
    )
    .unwrap();

    let args = vec!["--root".to_string(), root_str, "--json".to_string()];

    let code = unwired_ports::run_unwired_ports(&args);
    assert_eq!(code, 0);
}
