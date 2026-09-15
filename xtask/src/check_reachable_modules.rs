//! Gate: Verifiziert, dass jede `.rs`-Datei unter `crates/*/src/`, `benchmarks/*/src/` und `xtask/src/`
//! von mindestens einer `mod`-Deklaration (oder als Crate-Root / Binary-Target) erreichbar ist.

use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn run_check_reachable_modules() -> bool {
    println!("=== Gate: check-reachable-modules ===");

    let workspace_root = std::env::var("CARGO_WORKSPACE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    let crate_dirs = ["crates", "benchmarks", "xtask"];
    let mut unreachable_files: Vec<String> = Vec::new();

    for top_dir_name in crate_dirs {
        let top_path = workspace_root.join(top_dir_name);
        if top_dir_name == "xtask" {
            check_src_dir(&top_path.join("src"), &mut unreachable_files);
            continue;
        }

        if !top_path.is_dir() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&top_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let src_dir = path.join("src");
                    if src_dir.is_dir() {
                        check_src_dir(&src_dir, &mut unreachable_files);
                    }
                }
            }
        }
    }

    if unreachable_files.is_empty() {
        println!("✅ Alle .rs-Dateien sind über mod-Deklarationen oder Crate-Roots erreichbar.");
        true
    } else {
        eprintln!(
            "❌ Unerreichbare (Waisen-) .rs-Dateien ohne `mod`-Deklaration gefunden ({}):",
            unreachable_files.len()
        );
        for file in &unreachable_files {
            eprintln!("   {}", file);
        }
        false
    }
}

fn check_src_dir(src_dir: &Path, unreachable: &mut Vec<String>) {
    for entry in WalkDir::new(src_dir).into_iter().flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let rel = match path.strip_prefix(src_dir) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let components: Vec<&str> = rel
            .components()
            .map(|c| c.as_os_str().to_str().unwrap_or_default())
            .collect();

        if components.is_empty() {
            continue;
        }

        // 1. Crate-Roots (src/lib.rs, src/main.rs) sind automatisch erreichbar
        if rel == Path::new("lib.rs") || rel == Path::new("main.rs") {
            continue;
        }

        // 2. Executable Target Roots (src/bin/*.rs, src/bin/*/main.rs)
        if components[0] == "bin" {
            if components.len() == 2 {
                // src/bin/tool.rs
                continue;
            }
            if components.len() == 3 && components[2] == "main.rs" {
                // src/bin/tool/main.rs
                continue;
            }
        }

        // 3. Bestimme Modulnamen & Kandidaten für Elterndateien mit `mod <name>;`
        let file_name = components.last().copied().unwrap_or_default();
        let mod_name: &str;
        let candidate_parents: Vec<PathBuf>;

        if file_name == "mod.rs" {
            if components.len() == 2 {
                // z.B. src/foo/mod.rs -> Modulname "foo", Elterndatei lib.rs oder main.rs
                mod_name = components[0];
                candidate_parents = vec![src_dir.join("lib.rs"), src_dir.join("main.rs")];
            } else {
                // z.B. src/a/b/mod.rs -> Modulname "b", Elterndatei src/a.rs oder src/a/mod.rs
                mod_name = components[components.len() - 2];
                let parent_dir = src_dir.join(PathBuf::from_iter(&components[..components.len() - 2]));
                candidate_parents = vec![
                    parent_dir.with_extension("rs"),
                    parent_dir.join("mod.rs"),
                ];
            }
        } else {
            mod_name = file_name.strip_suffix(".rs").unwrap_or(file_name);
            if components.len() == 1 {
                // z.B. src/foo.rs -> Modulname "foo", Elterndatei lib.rs oder main.rs
                candidate_parents = vec![src_dir.join("lib.rs"), src_dir.join("main.rs")];
            } else {
                // z.B. src/a/b.rs -> Modulname "b", Elterndatei src/a.rs oder src/a/mod.rs
                let parent_dir = src_dir.join(PathBuf::from_iter(&components[..components.len() - 1]));
                candidate_parents = vec![
                    parent_dir.with_extension("rs"),
                    parent_dir.join("mod.rs"),
                ];
            }
        }

        // Prüfe ob mindestens eine Elterndatei `mod <mod_name>;` enthält
        let re_pattern = format!(
            r"(?m)^\s*(pub\s+)?(pub\((crate|super|self)\)\s+)?mod\s+{}\s*;",
            regex::escape(mod_name)
        );
        let re = match Regex::new(&re_pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let mut is_reachable = false;
        for parent_path in &candidate_parents {
            if parent_path.is_file() {
                if let Ok(content) = std::fs::read_to_string(parent_path) {
                    if re.is_match(&content) {
                        is_reachable = true;
                        break;
                    }
                }
            }
        }

        if !is_reachable {
            unreachable.push(path.display().to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_reachable_modules_workspace_clean() {
        assert!(run_check_reachable_modules());
    }
}
