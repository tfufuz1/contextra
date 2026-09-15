//! Gate: Prüft, ob jede `.rs`-Datei unter `src/` eines Crates von einer `mod`-Deklaration
//! in der Modul-Hierarchie erreicht wird. Verhindert unkompilierte Waisendateien ("Orphan Modules").

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Findet alle Crate-Roots unter `crates/` und `xtask/`.
fn find_crate_roots(root: &Path) -> Vec<PathBuf> {
    let mut crate_roots = Vec::new();

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .chain(WalkDir::new(root.join("xtask")))
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

/// Extrahiert alle Modulnamen (`mod foo;` oder `pub mod foo;`), die in einem Rust-Sourcefile deklariert sind.
fn extract_mod_declarations(file_path: &Path) -> HashSet<String> {
    let mut mods = HashSet::new();
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return mods,
    };

    let re = regex::Regex::new(r"^\s*(pub(\([^)]+\))?\s+)?mod\s+([a-zA-Z0-9_]+)\s*;").unwrap();

    for line in content.lines() {
        if let Some(caps) = re.captures(line) {
            mods.insert(caps[3].to_string());
        }
    }

    mods
}

/// Prüft ein einzelnes Crate auf unverdrahtete `.rs`-Waisendateien unter `src/`.
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

    let mut declared_mods_by_file: std::collections::HashMap<PathBuf, HashSet<String>> =
        std::collections::HashMap::new();

    for file in &all_rs_files {
        let declared = extract_mod_declarations(file);
        declared_mods_by_file.insert(file.clone(), declared);
    }

    let mut orphaned = Vec::new();

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

        let mod_name = match file.file_stem().and_then(|s| s.to_str()) {
            Some(stem) => stem,
            None => continue,
        };

        // Handle `mod.rs` (it's declared by parent dir's parent file or parent `mod.rs`)
        if file_name == "mod.rs" {
            let mod_dir_name = match parent_dir.file_name().and_then(|s| s.to_str()) {
                Some(name) => name,
                None => continue,
            };

            let grand_parent = match parent_dir.parent() {
                Some(dir) => dir,
                None => continue,
            };

            let parent_mod_rs = grand_parent.join("mod.rs");
            let parent_lib_rs = grand_parent.join("lib.rs");
            let parent_main_rs = grand_parent.join("main.rs");
            let parent_sibling_rs = grand_parent.join(format!("{}.rs", mod_dir_name));

            let mut is_declared = false;
            for candidate in &[
                parent_mod_rs,
                parent_lib_rs,
                parent_main_rs,
                parent_sibling_rs,
            ] {
                if candidate.exists() {
                    if let Some(mods) = declared_mods_by_file.get(candidate) {
                        if mods.contains(mod_dir_name) {
                            is_declared = true;
                            break;
                        }
                    }
                }
            }

            if !is_declared {
                orphaned.push(file.to_string_lossy().to_string());
            }

            continue;
        }

        // Standard `.rs` file (e.g., `graph_index.rs` in `src/traits/` or `budget.rs` in `src/types/`)
        // Candidate decl files: `parent_dir/mod.rs`, `parent_dir/lib.rs`, `parent_dir/main.rs`, or `parent_dir.rs` in parent_dir's parent
        let sibling_mod_rs = parent_dir.join("mod.rs");
        let sibling_lib_rs = parent_dir.join("lib.rs");
        let sibling_main_rs = parent_dir.join("main.rs");

        let parent_dir_name = parent_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let parent_dir_file = parent_dir
            .parent()
            .map(|p| p.join(format!("{}.rs", parent_dir_name)));

        let mut is_declared = false;

        let candidates = vec![
            Some(sibling_mod_rs),
            Some(sibling_lib_rs),
            Some(sibling_main_rs),
            parent_dir_file,
        ];

        for candidate in candidates.into_iter().flatten() {
            if candidate.exists() {
                if let Some(mods) = declared_mods_by_file.get(&candidate) {
                    if mods.contains(mod_name) {
                        is_declared = true;
                        break;
                    }
                }
            }
        }

        // Also check if any other `.rs` file in parent_dir declares this mod
        if !is_declared {
            for (declaring_file, mods) in &declared_mods_by_file {
                if declaring_file.parent() == Some(parent_dir) && mods.contains(mod_name) {
                    is_declared = true;
                    break;
                }
            }
        }

        if !is_declared {
            orphaned.push(file.to_string_lossy().to_string());
        }
    }

    orphaned.sort();
    orphaned
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
        assert!(orphans[0].ends_with("orphan.rs"));
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
