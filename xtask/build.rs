//! Build script for `xtask` crate.
//!
//! Scans `src/harness/*.rs` (excluding `mod.rs`), validates stems, summaries, and entry points,
//! and generates dispatch logic into `OUT_DIR/harness_generated.rs`.

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=src/harness");

    let out_dir = match std::env::var_os("OUT_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => return,
    };

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let harness_dir = Path::new(&manifest_dir).join("src").join("harness");

    let mut modules = Vec::new();

    if harness_dir.is_dir() {
        let entries = match fs::read_dir(&harness_dir) {
            Ok(e) => e,
            Err(e) => {
                panic!(
                    "Failed to read harness directory {}: {}",
                    harness_dir.display(),
                    e
                );
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
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

    modules.sort_by(|a, b| a.0.cmp(&b.0));

    // Check for collisions with main.rs subcommands
    let main_rs_path = Path::new(&manifest_dir).join("src").join("main.rs");
    if main_rs_path.exists() {
        if let Ok(main_content) = fs::read_to_string(&main_rs_path) {
            let main_cmds = extract_main_subcommands(&main_content);
            for (stem, _) in &modules {
                let cmd_name = stem.replace('_', "-");
                if main_cmds.contains(&cmd_name) {
                    panic!(
                        "BUILD ERROR: Harness command '{}' collides with existing subcommand in main.rs",
                        cmd_name
                    );
                }
            }
        }
    }

    let generated_code = generate_harness_code(&manifest_dir, &modules);
    let dest_path = out_dir.join("harness_generated.rs");
    fs::write(&dest_path, generated_code).unwrap_or_else(|e| {
        panic!(
            "Failed to write generated harness code to {}: {}",
            dest_path.display(),
            e
        )
    });
}

fn validate_stem(stem: &str, path: &Path) {
    let valid = !stem.is_empty()
        && stem
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_lowercase())
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

pub fn extract_main_subcommands(main_rs_content: &str) -> std::collections::HashSet<String> {
    let mut cmds = std::collections::HashSet::new();
    for line in main_rs_content.lines() {
        if let Some((patterns, _)) = line.split_once("=>") {
            for match_str in patterns.split('|') {
                let trimmed = match_str.trim();
                if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 2 {
                    cmds.insert(trimmed[1..trimmed.len() - 1].to_string());
                }
            }
        }
    }
    cmds
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
