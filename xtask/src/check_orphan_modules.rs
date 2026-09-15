//! Gate: Prüft, ob jede `.rs`-Datei unter `src/` eines Crates von einer `mod`-Deklaration
//! in der Modul-Hierarchie erreicht wird. Verhindert unkompilierte Waisendateien ("Orphan Modules").

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Findet alle Crate-Roots unter `crates/` und `xtask/`.
fn find_crate_roots(root: &Path) -> Vec<PathBuf> {
    let mut crate_roots = Vec::new();

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .chain(WalkDir::new(root.join("xtask")))
        .chain(WalkDir::new(root.join("benchmarks")))
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.is_file() && p.file_name().and_then(|s| s.to_str()) == Some("Cargo.toml") {
            if let Some(parent) = p.parent() {
                if parent.join("src").exists() {
                    crate_roots.push(parent.to_path_buf());
                }
            }
        }
    }

    crate_roots.sort();
    crate_roots.dedup();
    crate_roots
}

#[derive(Debug, Clone)]
struct ModDeclOccurrence {
    file_path: PathBuf,
    line_num: usize,
    mod_name: String,
}

/// Extrahiert alle Moduldeklarationen (`mod foo;` oder `pub mod foo;`) mit Zeilennummern.
fn extract_mod_declarations(file_path: &Path) -> Vec<ModDeclOccurrence> {
    let mut decls = Vec::new();
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return decls,
    };

    let re = match regex::Regex::new(r"^\s*(pub(\([^)]+\))?\s+)?mod\s+([a-zA-Z0-9_]+)\s*;") {
        Ok(r) => r,
        Err(_) => return decls,
    };

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        if let Some(caps) = re.captures(line) {
            decls.push(ModDeclOccurrence {
                file_path: file_path.to_path_buf(),
                line_num: line_idx + 1,
                mod_name: caps[3].to_string(),
            });
        }
    }

    decls
}

/// Prüft ein einzelnes Crate auf Erreichbarkeit aller `.rs`-Dateien unter `src/`.
/// Jede Datei muss von genau 1 `mod`-Deklaration erreicht werden (0 = Waise, 2+ = Mehrfachdeklaration).
pub fn check_crate_orphan_modules(crate_root: &Path) -> Vec<String> {
    let src_dir = crate_root.join("src");
    if !src_dir.exists() {
        return Vec::new();
    }

    let mut all_rs_files = Vec::new();
    for entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rs") {
            all_rs_files.push(p.to_path_buf());
        }
    }

    let mut all_declarations = Vec::new();
    for file in &all_rs_files {
        let decls = extract_mod_declarations(file);
        all_declarations.extend(decls);
    }

    let mut violations = Vec::new();

    for file in &all_rs_files {
        let file_name = match file.file_name().and_then(|s| s.to_str()) {
            Some(name) => name,
            None => continue,
        };

        // Root entry points and binaries are never orphan modules.
        if file_name == "lib.rs" || file_name == "main.rs" {
            continue;
        }

        let rel_to_src = match file.strip_prefix(&src_dir) {
            Ok(rel) => rel,
            Err(_) => continue,
        };

        // Binary targets in src/bin/ are entry points
        if rel_to_src.starts_with("bin") {
            continue;
        }

        let parent_dir = match file.parent() {
            Some(dir) => dir,
            None => continue,
        };

        let mut declaring_occurrences = Vec::new();

        if file_name == "mod.rs" {
            let mod_dir_name = match parent_dir.file_name().and_then(|s| s.to_str()) {
                Some(name) => name,
                None => continue,
            };

            let grand_parent = match parent_dir.parent() {
                Some(dir) => dir,
                None => continue,
            };

            for decl in &all_declarations {
                if decl.mod_name == mod_dir_name {
                    let decl_parent = decl.file_path.parent();
                    let is_grand_parent_file = decl_parent == Some(grand_parent);
                    let is_parent_sibling_file =
                        decl.file_path == grand_parent.join(format!("{}.rs", mod_dir_name));
                    if is_grand_parent_file || is_parent_sibling_file {
                        declaring_occurrences.push(decl.clone());
                    }
                }
            }
        } else {
            let mod_name = match file.file_stem().and_then(|s| s.to_str()) {
                Some(stem) => stem,
                None => continue,
            };

            let parent_dir_name = parent_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let parent_dir_sibling_file = parent_dir
                .parent()
                .map(|p| p.join(format!("{}.rs", parent_dir_name)));

            for decl in &all_declarations {
                if decl.mod_name == mod_name {
                    let is_sibling_in_parent_dir =
                        decl.file_path.parent() == Some(parent_dir) && decl.file_path != *file;
                    let is_parent_dir_sibling =
                        parent_dir_sibling_file.as_ref() == Some(&decl.file_path);
                    if is_sibling_in_parent_dir || is_parent_dir_sibling {
                        declaring_occurrences.push(decl.clone());
                    }
                }
            }
        }

        if declaring_occurrences.is_empty() {
            violations.push(format!(
                "{} (Waise: 0 mod-Deklarationen in Modulhierarchie)",
                file.display()
            ));
        } else if declaring_occurrences.len() > 1 {
            let decl_strs: Vec<String> = declaring_occurrences
                .iter()
                .map(|d| format!("{}:{}", d.file_path.display(), d.line_num))
                .collect();
            violations.push(format!(
                "{} (Mehrfachdeklaration: in {} Stellen deklariert: {})",
                file.display(),
                declaring_occurrences.len(),
                decl_strs.join(", ")
            ));
        }
    }

    violations.sort();
    violations
}

/// Hauptfunktion für xtask: scannt den Workspace auf Waisendateien.
pub fn run_check_orphan_modules(root: &Path) -> Result<Vec<String>, String> {
    let crate_roots = find_crate_roots(root);
    let mut all_orphaned = Vec::new();

    for crate_root in crate_roots {
        let orphans = check_crate_orphan_modules(&crate_root);
        all_orphaned.extend(orphans);
    }

    Ok(all_orphaned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_orphan_module_detected() {
        let dir = tempdir().unwrap();
        let crate_dir = dir.path().join("my_crate");
        let src_dir = crate_dir.join("src");
        let sub_dir = src_dir.join("sub");
        fs::create_dir_all(&sub_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"my_crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(src_dir.join("lib.rs"), "pub mod sub;\n").unwrap();
        // sub/mod.rs does NOT declare `pub mod orphan;`
        fs::write(sub_dir.join("mod.rs"), "pub mod valid;\n").unwrap();
        fs::write(sub_dir.join("valid.rs"), "// valid module\n").unwrap();
        fs::write(sub_dir.join("orphan.rs"), "// orphan module\n").unwrap();

        let orphans = check_crate_orphan_modules(&crate_dir);
        assert_eq!(orphans.len(), 1);
        assert!(orphans[0].contains("orphan.rs"));
        assert!(orphans[0].contains("Waise"));
    }

    #[test]
    fn test_z10_reference_case_four_unwired_files_detected() {
        let dir = tempdir().unwrap();
        let crate_dir = dir.path().join("memfuse-core");
        let src_dir = crate_dir.join("src");
        let traits_dir = src_dir.join("traits");
        fs::create_dir_all(&traits_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"memfuse-core\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(src_dir.join("lib.rs"), "pub mod traits;\n").unwrap();
        // traits/mod.rs does NOT declare any of the 4 trait files
        fs::write(
            traits_dir.join("mod.rs"),
            "// traits entry point without mod decls\n",
        )
        .unwrap();
        fs::write(
            traits_dir.join("graph_index.rs"),
            "pub trait GraphIndex {}\n",
        )
        .unwrap();
        fs::write(traits_dir.join("text_index.rs"), "pub trait TextIndex {}\n").unwrap();
        fs::write(
            traits_dir.join("vector_index.rs"),
            "pub trait VectorIndex {}\n",
        )
        .unwrap();
        fs::write(
            traits_dir.join("checkpoint.rs"),
            "pub trait Checkpoint {}\n",
        )
        .unwrap();

        let violations = check_crate_orphan_modules(&crate_dir);
        assert_eq!(
            violations.len(),
            4,
            "Expected 4 orphan violations for unwired trait files, got: {:?}",
            violations
        );
        assert!(violations.iter().any(|v| v.contains("graph_index.rs")));
        assert!(violations.iter().any(|v| v.contains("text_index.rs")));
        assert!(violations.iter().any(|v| v.contains("vector_index.rs")));
        assert!(violations.iter().any(|v| v.contains("checkpoint.rs")));
    }

    #[test]
    fn test_duplicate_mod_declarations_detected() {
        let dir = tempdir().unwrap();
        let crate_dir = dir.path().join("my_crate");
        let src_dir = crate_dir.join("src");
        let sub_dir = src_dir.join("sub");
        fs::create_dir_all(&sub_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"my_crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(src_dir.join("lib.rs"), "pub mod sub;\n").unwrap();
        // sub/mod.rs duplicate declaration of mod child;
        fs::write(sub_dir.join("mod.rs"), "pub mod child;\npub mod child;\n").unwrap();
        fs::write(sub_dir.join("child.rs"), "// child module\n").unwrap();

        let violations = check_crate_orphan_modules(&crate_dir);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("child.rs"));
        assert!(violations[0].contains("Mehrfachdeklaration"));
    }

    #[test]
    fn test_bin_targets_are_not_flagged_as_orphans() {
        let dir = tempdir().unwrap();
        let crate_dir = dir.path().join("my_crate");
        let src_dir = crate_dir.join("src");
        let bin_dir = src_dir.join("bin");
        let tool_dir = bin_dir.join("tool");
        fs::create_dir_all(&tool_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"my_crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(src_dir.join("lib.rs"), "// lib root\n").unwrap();
        fs::write(bin_dir.join("cli.rs"), "fn main() {}\n").unwrap();
        fs::write(tool_dir.join("main.rs"), "fn main() {}\n").unwrap();

        let violations = check_crate_orphan_modules(&crate_dir);
        assert!(
            violations.is_empty(),
            "Expected no orphan violations for bin targets, got: {:?}",
            violations
        );
    }

    #[test]
    fn test_valid_modules_no_orphans() {
        let dir = tempdir().unwrap();
        let crate_dir = dir.path().join("my_crate");
        let src_dir = crate_dir.join("src");
        let sub_dir = src_dir.join("sub");
        fs::create_dir_all(&sub_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"my_crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(src_dir.join("lib.rs"), "pub mod sub;\n").unwrap();
        fs::write(sub_dir.join("mod.rs"), "pub mod first;\npub mod second;\n").unwrap();
        fs::write(sub_dir.join("first.rs"), "// first\n").unwrap();
        fs::write(sub_dir.join("second.rs"), "// second\n").unwrap();

        let orphans = check_crate_orphan_modules(&crate_dir);
        assert!(
            orphans.is_empty(),
            "Expected no orphans, got: {:?}",
            orphans
        );
    }
}
