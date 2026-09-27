use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::tempdir;

static TEST_LOCK: Mutex<()> = Mutex::new(());

pub fn find_root_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CONTEXTRA_ROOT_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut dir = PathBuf::from(manifest_dir);
        loop {
            let cargo_path = dir.join("Cargo.toml");
            if cargo_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                    if content.contains("[workspace]") && content.contains("members") {
                        return dir;
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
    }
    PathBuf::from(".")
}

#[path = "../src/crate_context.rs"]
mod crate_context;

use crate_context::{run_crate_context, CrateContextData, OutputFormat};

#[test]
fn test_run_crate_context_markdown_and_json() {
    let _guard = TEST_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let root = dir.path();
    std::env::set_var("CONTEXTRA_ROOT_DIR", root);

    // Create synthetic capabilities.toml
    let caps_toml = r#"[crates.contextra-test]
ring = 'Ring 0'
maturity = 'stable'
description = 'Test Crate'
"#;
    fs::write(root.join("capabilities.toml"), caps_toml).unwrap();

    // Create synthetic crate dir
    let crate_dir = root.join("crates/contextra-test");
    let src_dir = crate_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let cargo_toml = r#"[package]
name = "contextra-test"
version = "0.1.0"

[dependencies]
contextra-core = { path = "../contextra-core" }
contextra-types = { path = "../contextra-types" }
serde = "1"
"#;
    fs::write(crate_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let lib_rs = r#"// Line 1
// AI-TAG[v1][TEST] Test tag
pub fn test_fn() -> usize {
    42
}
"#;
    fs::write(src_dir.join("lib.rs"), lib_rs).unwrap();

    let agents_md = "# Agent Guidelines\nRule 1: Always verify\n";
    fs::write(crate_dir.join("AGENTS.md"), agents_md).unwrap();

    // Run Markdown format
    let md_out = root.join("out/context.md");
    let md_res =
        run_crate_context("contextra-test", OutputFormat::Markdown, Some(&md_out)).unwrap();
    assert!(md_res.contains("# Crate Context: contextra-test"));
    assert!(md_res.contains("- **Ring:** Ring 0"));
    assert!(md_res.contains("- **Maturity:** stable"));
    assert!(md_res.contains("contextra-core"));
    assert!(md_res.contains("contextra-types"));
    assert!(md_res.contains("AI-TAG[v1][TEST]"));
    assert!(md_out.is_file());

    // Run JSON format
    let json_out = root.join("out/context.json");
    let json_res =
        run_crate_context("contextra-test", OutputFormat::Json, Some(&json_out)).unwrap();
    assert!(json_out.is_file());

    let parsed: CrateContextData = serde_json::from_str(&json_res).unwrap();
    assert_eq!(parsed.crate_name, "contextra-test");
    assert_eq!(parsed.ring, "Ring 0");
    assert_eq!(parsed.maturity, "stable");
    assert_eq!(parsed.loc, 5);
    assert_eq!(
        parsed.declared_dependencies,
        vec!["contextra-core".to_string(), "contextra-types".to_string()]
    );
    assert_eq!(parsed.open_ai_tags.len(), 1);
    assert!(parsed
        .agents_md_excerpt
        .unwrap()
        .contains("Rule 1: Always verify"));

    std::env::remove_var("CONTEXTRA_ROOT_DIR");
}

#[test]
fn test_run_crate_context_nonexistent_crate_fails() {
    let _guard = TEST_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let root = dir.path();
    std::env::set_var("CONTEXTRA_ROOT_DIR", root);

    let res = run_crate_context("nonexistent-crate", OutputFormat::Markdown, None);
    assert!(res.is_err());

    std::env::remove_var("CONTEXTRA_ROOT_DIR");
}
