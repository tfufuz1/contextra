//! Build script for `xtask` crate.
//!
//! Generates harness dispatch logic in `OUT_DIR/harness_generated.rs`
//! and gate cluster modules in `OUT_DIR/gates_generated.rs`.

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=src/harness");
    println!("cargo:rerun-if-changed=src/gates");

    let out_dir = match std::env::var_os("OUT_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => return,
    };

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    // 1. Generate harness_generated.rs
    generate_harness(&manifest_dir, &out_dir);

    // 2. Generate gates_generated.rs
    generate_gates(&manifest_dir, &out_dir);
}

fn generate_harness(manifest_dir: &str, out_dir: &Path) {
    let harness_dir = Path::new(manifest_dir).join("src").join("harness");
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

                    validate_stem(&stem, &path);

                    let content = fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

                    let summary = extract_summary(&content, &path);
                    validate_run_fn(&content, &stem, &path);

                    modules.push((stem, summary));
                }
            }
        }
    }

    modules.sort_by(|a, b| a.0.cmp(&b.0));

    let generated_code = generate_harness_code(manifest_dir, &modules);
    let dest_path = out_dir.join("harness_generated.rs");
    let _ = fs::write(&dest_path, generated_code);
}

fn validate_stem(stem: &str, path: &Path) {
    let valid = !stem.is_empty()
        && stem.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && stem
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

    if !valid {
        panic!(
            "BUILD ERROR: Invalid harness module filename stem '{}' at {}. Must match [a-z][a-z0-9_]*",
            stem,
            path.display()
        );
    }
}

fn extract_summary(content: &str, path: &Path) -> String {
    let first_line = content.lines().next().unwrap_or("");
    if let Some(summary) = first_line.strip_prefix("//!") {
        let trimmed = summary.trim();
        if trimmed.is_empty() {
            panic!(
                "BUILD ERROR: Harness module {} first line `//!` summary cannot be empty",
                path.display()
            );
        }
        trimmed.to_string()
    } else {
        panic!(
            "BUILD ERROR: Harness module {} must start with `//! <summary>` line",
            path.display()
        );
    }
}

fn validate_run_fn(content: &str, stem: &str, path: &Path) {
    let expected_fn = format!("pub fn run_{}", stem);
    if !content.contains(&expected_fn) {
        panic!(
            "BUILD ERROR: Harness module {} missing entry function `{}(args: &[String]) -> i32`",
            path.display(),
            expected_fn
        );
    }
}

fn generate_harness_code(manifest_dir: &str, modules: &[(String, String)]) -> String {
    let mut code = String::new();
    code.push_str("// Auto-generated harness dispatch code. DO NOT EDIT.\n\n");

    for (stem, _) in modules {
        let harness_path = Path::new(manifest_dir)
            .join("src")
            .join("harness")
            .join(format!("{}.rs", stem));
        let path_str = harness_path.to_str().unwrap().replace('\\', "/");
        code.push_str(&format!("#[path = \"{}\"]\nmod {};\n\n", path_str, stem));
    }

    code.push_str("pub fn dispatch(cmd: &str, _args: &[String]) -> Option<i32> {\n");
    code.push_str("    match cmd {\n");
    for (stem, _) in modules {
        let cmd_name = stem.replace('_', "-");
        code.push_str(&format!(
            "        \"{}\" => Some({}::run_{}(_args)),\n",
            cmd_name, stem, stem
        ));
    }
    code.push_str("        _ => None,\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    code.push_str("pub fn registered_commands() -> &'static [(&'static str, &'static str)] {\n");
    code.push_str("    &[\n");
    for (stem, summary) in modules {
        let cmd_name = stem.replace('_', "-");
        let escaped_summary = summary.replace('\\', "\\\\").replace('"', "\\\"");
        code.push_str(&format!(
            "        (\"{}\", \"{}\"),\n",
            cmd_name, escaped_summary
        ));
    }
    code.push_str("    ]\n");
    code.push_str("}\n");

    code
}

fn generate_gates(manifest_dir: &str, out_dir: &Path) {
    let gates_dir = Path::new(manifest_dir).join("src").join("gates");
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
}
