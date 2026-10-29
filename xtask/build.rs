//! Build script for `xtask` crate.
//!
//! Scans `src/gates/<cluster>/*.rs` and generates module imports and registry dispatch logic.

use std::fs;
use std::path::{Path, PathBuf};

// Build script needs Cargo-injected build environment variables (OUT_DIR, CARGO_MANIFEST_DIR).
// RING-LAYER-EXCEPTION: ADR-098 / P28 exception for build.rs reading Cargo build environment variables.
#[allow(clippy::disallowed_methods)]
fn main() {
    println!("cargo:rerun-if-changed=src/gates");
    println!("cargo:rerun-if-changed=src/harness");
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    println!("cargo:rerun-if-env-changed=CARGO_MANIFEST_DIR");

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

    // Harness generation
    let harness_dir = Path::new(&manifest_dir).join("src").join("harness");
    let mut modules = Vec::new();

    if harness_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&harness_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
                    let stem = match path.file_stem().and_then(|s| s.to_str()) {
                        Some(s) => s.to_string(),
                        None => continue,
                    };
                    if stem == "mod" {
                        continue;
                    }
                    let content = fs::read_to_string(&path).unwrap_or_default();
                    let first_line = content.lines().next().unwrap_or("");
                    let summary = first_line
                        .strip_prefix("//!")
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    modules.push((stem, summary));
                }
            }
        }
    }

    modules.sort_by(|a, b| a.0.cmp(&b.0));

    let mut harness_code = String::new();
    harness_code.push_str("// Auto-generated harness dispatch code. DO NOT EDIT.\n\n");

    for (stem, _) in &modules {
        let harness_path = Path::new(&manifest_dir)
            .join("src")
            .join("harness")
            .join(format!("{}.rs", stem));
        let path_str = match harness_path.to_str() {
            Some(s) => s.replace('\\', "/"),
            None => continue,
        };
        harness_code.push_str(&format!("#[path = \"{}\"]\nmod {};\n\n", path_str, stem));
    }

    harness_code.push_str("pub fn dispatch(cmd: &str, _args: &[String]) -> Option<i32> {\n");
    harness_code.push_str("    match cmd {\n");
    for (stem, _) in &modules {
        let cmd_name = stem.replace('_', "-");
        harness_code.push_str(&format!(
            "        \"{}\" => Some({}::run_{}(_args)),\n",
            cmd_name, stem, stem
        ));
    }
    harness_code.push_str("        _ => None,\n");
    harness_code.push_str("    }\n");
    harness_code.push_str("}\n\n");

    harness_code
        .push_str("pub fn registered_commands() -> &'static [(&'static str, &'static str)] {\n");
    harness_code.push_str("    &[\n");
    for (stem, summary) in &modules {
        let cmd_name = stem.replace('_', "-");
        let escaped_summary = summary.replace('\\', "\\\\").replace('"', "\\\"");
        harness_code.push_str(&format!(
            "        (\"{}\", \"{}\"),\n",
            cmd_name, escaped_summary
        ));
    }
    harness_code.push_str("    ]\n");
    harness_code.push_str("}\n");

    let harness_dest = out_dir.join("harness_generated.rs");
    let _ = fs::write(&harness_dest, harness_code);
}
