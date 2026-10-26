//! Build script for `xtask` crate.
//!
//! Scans `src/gates/<cluster>/*.rs` and generates module imports and registry dispatch logic.

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=src/gates");

    let out_dir = match std::env::var_os("OUT_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => return,
    };

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let gates_dir = Path::new(&manifest_dir).join("src").join("gates");

    let mut clusters = Vec::new();

    if gates_dir.is_dir() {
        if let Ok(cluster_entries) = fs::read_dir(&gates_dir) {
            for entry in cluster_entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let cluster_name = match path.file_name().and_then(|s| s.to_str()) {
                        Some(s) => s.to_string(),
                        None => continue,
                    };
                    clusters.push(cluster_name);
                }
            }
        }
    }

    clusters.sort();

    let mut generated_code = String::new();
    generated_code.push_str("// Auto-generated gate cluster modules. DO NOT EDIT.\n\n");

    for cluster in &clusters {
        generated_code.push_str(&format!("pub mod {};\n", cluster));
    }

    let dest_path = out_dir.join("gates_generated.rs");
    let _ = fs::write(&dest_path, generated_code);

    let harness_dest_path = out_dir.join("harness_generated.rs");
    let _ = fs::write(
        &harness_dest_path,
        r#"// Auto-generated harness stub.
pub fn dispatch(_cmd: &str, _args: &[String]) -> Option<i32> { None }
pub fn registered_commands() -> Vec<(&'static str, &'static str)> { Vec::new() }
"#,
    );
}
