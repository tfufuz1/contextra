//! Build script for `xtask` crate.
//!
//! Scans `src/harness/*.rs` (excluding `mod.rs`), validates stems, summaries, and entry points,
//! and generates dispatch logic into `OUT_DIR/harness_generated.rs`.

#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Subcommands explicitly allowed to exist both as a harness module and in `cli/mod.rs`.
const ALLOWED_COLLISIONS: &[&str] = &[
    // gc-floor-single-source is registered both in harness/ and cli/mod.rs as an alias.
    "gc-floor-single-source",
];

fn main() {
    println!("cargo:rerun-if-changed=src/harness");
    println!("cargo:rerun-if-changed=src/cli/mod.rs");

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

    modules.sort_by(|a, b| a.0.cmp(&b.0));

    // Check for collisions with cli/mod.rs subcommands
    let cli_mod_path = Path::new(&manifest_dir).join("src").join("cli").join("mod.rs");
    if cli_mod_path.exists() {
        let cli_content = fs::read_to_string(&cli_mod_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", cli_mod_path.display(), e));
        let cli_cmds = extract_cli_subcommands(&cli_content);
        if cli_cmds.is_empty() {
            panic!(
                "BUILD ERROR: Extracted 0 subcommands from {}. Check parser logic.",
                cli_mod_path.display()
            );
        }

        for (stem, _) in &modules {
            let cmd_name = stem.replace('_', "-");
            if cli_cmds.contains(&cmd_name) && !ALLOWED_COLLISIONS.contains(&cmd_name.as_str()) {
                panic!(
                    "BUILD ERROR: Harness command '{}' collides with existing subcommand in cli/mod.rs",
                    cmd_name
                );
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

/// Helper function to strip line comments (`//...`) and block comments (`/*...*/`) from source text.
fn strip_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_block_comment = false;

    while let Some(c) = chars.next() {
        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if c == '/' {
            if chars.peek() == Some(&'*') {
                chars.next();
                in_block_comment = true;
                continue;
            } else if chars.peek() == Some(&'/') {
                // Skip line comment until newline
                while let Some(&next) = chars.peek() {
                    if next == '\n' {
                        break;
                    }
                    chars.next();
                }
                continue;
            }
        }

        output.push(c);
    }

    output
}

pub fn extract_cli_subcommands(cli_mod_content: &str) -> HashSet<String> {
    let mut cmds = HashSet::new();
    let clean = strip_comments(cli_mod_content);

    let table_str = if let Some((_, after)) = clean.split_once("COMMAND_DISPATCH_TABLE") {
        after
    } else {
        return cmds;
    };

    // Parse entries of form ("command-name", ...)
    let bytes = table_str.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let start = i + 1;
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            if i < bytes.len() {
                let candidate = &table_str[start..i];
                // Verify this string is a tuple key by checking if followed by optional whitespace and comma
                let rest = table_str[i + 1..].trim_start();
                if rest.starts_with(',')
                    && !candidate.is_empty()
                    && candidate
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
                {
                    cmds.insert(candidate.to_string());
                }
            }
        }
        i += 1;
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
        code.push_str(&format!(
            "#[path = \"{}\"]\npub mod {};\n\n",
            path_str, stem
        ));
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
