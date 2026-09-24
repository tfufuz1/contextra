//! Gate: check-module-reachability
//! Prüft, ob jede `.rs`-Datei unter `src/` eines Workspace-Members von mindestens
//! einer `mod`-Deklaration in der Modul-Hierarchie erreicht wird.
//! Ausnahmen (Wurzeln, Binaries, Übergangsliste) werden gesondert behandelt.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Übergangsliste für unverdrahtete Dateien (z. B. unfertige Traits/Module in Phase 0R).
/// Diese Funde erzeugen eine WARN-Ausgabe statt eines Fehlers (Exit 0).
pub const KNOWN_TRANSITION_UNREACHABLE: &[&str] = &[
    "crates/contextra-core/src/traits/graph_index.rs",
    "crates/contextra-core/src/traits/vector_index.rs",
    "crates/contextra-core/src/traits/text_index.rs",
    "crates/contextra-core/src/traits/lifecycle.rs",
    "crates/contextra-db/src/decay_controller.rs",
    "crates/contextra-db/src/filter.rs",
    "crates/contextra-db/src/fusion.rs",
    "crates/contextra-db/src/homeostat.rs",
    "crates/contextra-db/src/maintenance_config.rs",
    "crates/contextra-db/src/maintenance_scheduler.rs",
    "crates/contextra-db/src/multistep.rs",
    "crates/contextra-db/src/pid_latency_controller.rs",
    "crates/contextra-db/src/transaction.rs",
    "crates/contextra-db/src/volatile_vault.rs",
    "benchmarks/contextra-bench/src/bin/compare_baseline.rs",
    "crates/contextra-mcp/src/bin/contextra-mcp-server.rs",
];

#[derive(Debug, Clone)]
struct ModDecl {
    declaring_file: PathBuf,
    line_num: usize,
    mod_name: String,
    custom_path: Option<String>,
}

/// Baut alle Kommentare (`//...` und `/* ... */`) sowie String/Char-Literale aus dem Rust-Code um,
/// um Fehlalarme durch `mod foo;` in Kommentaren/Strings zu vermeiden.
fn strip_comments_and_strings(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;

    let mut in_line_comment = false;
    let mut in_block_comment = 0;
    let mut in_string = false;
    let mut in_char = false;

    while i < len {
        let c = chars[i];
        let next = if i + 1 < len {
            Some(chars[i + 1])
        } else {
            None
        };

        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                result.push('\n');
            } else {
                result.push(' ');
            }
            i += 1;
            continue;
        }

        if in_block_comment > 0 {
            if c == '/' && next == Some('*') {
                in_block_comment += 1;
                result.push(' ');
                result.push(' ');
                i += 2;
            } else if c == '*' && next == Some('/') {
                in_block_comment -= 1;
                result.push(' ');
                result.push(' ');
                i += 2;
            } else {
                if c == '\n' {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
                i += 1;
            }
            continue;
        }

        if in_string {
            if c == '\\' {
                result.push(' ');
                result.push(' ');
                i += 2;
            } else if c == '"' {
                in_string = false;
                result.push(' ');
                i += 1;
            } else {
                if c == '\n' {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
                i += 1;
            }
            continue;
        }

        if in_char {
            if c == '\\' {
                result.push(' ');
                result.push(' ');
                i += 2;
            } else if c == '\'' {
                in_char = false;
                result.push(' ');
                i += 1;
            } else {
                if c == '\n' {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
                i += 1;
            }
            continue;
        }

        if c == '/' && next == Some('/') {
            in_line_comment = true;
            result.push(' ');
            result.push(' ');
            i += 2;
            continue;
        }

        if c == '/' && next == Some('*') {
            in_block_comment = 1;
            result.push(' ');
            result.push(' ');
            i += 2;
            continue;
        }

        if c == '"' {
            in_string = true;
            result.push(' ');
            i += 1;
            continue;
        }

        if c == '\'' {
            let is_char_lit = if i + 2 < len && chars[i + 2] == '\'' && chars[i + 1] != '\\' {
                true
            } else if i + 3 < len && chars[i + 1] == '\\' && chars[i + 3] == '\'' {
                true
            } else if i + 4 < len && chars[i + 1] == '\\' && chars[i + 2] == 'x' && chars[i + 4] == '\'' {
                true
            } else if i + 3 < len && chars[i + 1] == '\\' && chars[i + 2] == 'u' {
                let mut found_end = false;
                for j in (i + 3)..len.min(i + 12) {
                    if chars[j] == '\'' {
                        found_end = true;
                        break;
                    }
                }
                found_end
            } else {
                false
            };

            if is_char_lit {
                in_char = true;
                result.push(' ');
                i += 1;
                continue;
            } else {
                result.push(c);
                i += 1;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

/// Extrahiert `mod foo;` und `#[path = "..."] mod foo;` aus einer Datei.
fn extract_mod_declarations(file_path: &Path) -> Vec<ModDecl> {
    let mut decls = Vec::new();
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return decls,
    };

    let stripped = strip_comments_and_strings(&content);
    let lines: Vec<&str> = stripped.lines().collect();

    let mod_re = match regex::Regex::new(r"^\s*(pub(\([^)]+\))?\s+)?mod\s+([a-zA-Z0-9_]+)\s*;") {
        Ok(r) => r,
        Err(_) => return decls,
    };

    let path_re = match regex::Regex::new(r#"^\s*#\s*\[\s*path\s*=\s*"([^"]+)"\s*\]"#) {
        Ok(r) => r,
        Err(_) => return decls,
    };

    let mut pending_path: Option<String> = None;

    for (line_idx, line) in lines.iter().enumerate() {
        if let Some(caps) = path_re.captures(line) {
            pending_path = Some(caps[1].to_string());
            continue;
        }

        if let Some(caps) = mod_re.captures(line) {
            decls.push(ModDecl {
                declaring_file: file_path.to_path_buf(),
                line_num: line_idx + 1,
                mod_name: caps[3].to_string(),
                custom_path: pending_path.take(),
            });
        } else {
            let line_trimmed = line.trim();
            if !line_trimmed.is_empty() && !line_trimmed.starts_with('#') {
                pending_path = None;
            }
        }
    }

    decls
}

/// Löst die Ziel-Datei für eine Moduldeklaration auf.
fn resolve_mod_file(decl: &ModDecl, src_dir: &Path) -> Option<PathBuf> {
    let parent_dir = decl.declaring_file.parent()?;

    if let Some(ref custom) = decl.custom_path {
        let p = parent_dir.join(custom);
        if p.exists() {
            return Some(p.canonicalize().unwrap_or(p));
        }
        return None;
    }

    let is_root_or_mod_rs =
        if let Some(stem) = decl.declaring_file.file_stem().and_then(|s| s.to_str()) {
            stem == "lib"
                || stem == "main"
                || stem == "mod"
                || decl.declaring_file.parent() == Some(src_dir)
        } else {
            false
        };

    let base_dir = if decl.declaring_file.file_name().and_then(|s| s.to_str()) == Some("mod.rs")
        || decl.declaring_file == src_dir.join("lib.rs")
        || decl.declaring_file == src_dir.join("main.rs")
    {
        parent_dir.to_path_buf()
    } else {
        let stem = decl.declaring_file.file_stem()?.to_str()?;
        parent_dir.join(stem)
    };

    let candidate1 = base_dir.join(format!("{}.rs", decl.mod_name));
    if candidate1.is_file() {
        return Some(candidate1.canonicalize().unwrap_or(candidate1));
    }

    let candidate2 = base_dir.join(&decl.mod_name).join("mod.rs");
    if candidate2.is_file() {
        return Some(candidate2.canonicalize().unwrap_or(candidate2));
    }

    if is_root_or_mod_rs {
        let candidate3 = parent_dir.join(format!("{}.rs", decl.mod_name));
        if candidate3.is_file() {
            return Some(candidate3.canonicalize().unwrap_or(candidate3));
        }
        let candidate4 = parent_dir.join(&decl.mod_name).join("mod.rs");
        if candidate4.is_file() {
            return Some(candidate4.canonicalize().unwrap_or(candidate4));
        }
    }

    None
}

/// Sucht nach bekannten Crate-Roots unter `crates/`, `xtask/`, `benchmarks/`.
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

pub struct ModuleReachabilityResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Scannt ein Crate auf Erreichbarkeit aller `.rs`-Dateien unter `src/`.
pub fn check_crate_reachability(crate_root: &Path, repo_root: &Path) -> ModuleReachabilityResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let raw_src_dir = crate_root.join("src");
    if !raw_src_dir.exists() {
        return ModuleReachabilityResult { errors, warnings };
    }
    let src_dir = raw_src_dir.canonicalize().unwrap_or(raw_src_dir);

    let mut all_rs_files = Vec::new();
    for entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rs") {
            let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
            all_rs_files.push(canon);
        }
    }

    let mut all_declarations = Vec::new();
    for file in &all_rs_files {
        let decls = extract_mod_declarations(file);
        all_declarations.extend(decls);
    }

    let mut target_decl_map: HashMap<PathBuf, Vec<ModDecl>> = HashMap::new();
    for decl in &all_declarations {
        if let Some(target) = resolve_mod_file(decl, &src_dir) {
            target_decl_map
                .entry(target)
                .or_default()
                .push(decl.clone());
        }
    }

    let repo_root_canon = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());

    for file in &all_rs_files {
        let rel_to_repo = file
            .strip_prefix(&repo_root_canon)
            .or_else(|_| file.strip_prefix(repo_root))
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/");

        let file_name = match file.file_name().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if file_name == "lib.rs" || file_name == "main.rs" {
            continue;
        }

        if let Ok(rel_to_src) = file.strip_prefix(&src_dir) {
            if rel_to_src.starts_with("bin") {
                continue;
            }
        }

        let decls = target_decl_map.get(file);
        let decl_count = decls.map(|d| d.len()).unwrap_or(0);

        let is_known_transition = KNOWN_TRANSITION_UNREACHABLE
            .iter()
            .any(|&t| t == rel_to_repo);

        if decl_count == 0 {
            let msg = format!("{} (Unerreichbar: 0 mod-Deklarationen)", rel_to_repo);
            if is_known_transition {
                warnings.push(format!("TRANSITION: {}", msg));
            } else {
                errors.push(msg);
            }
        } else if decl_count > 1 {
            let decl_strs: Vec<String> = decls
                .unwrap()
                .iter()
                .map(|d| {
                    let d_rel = d
                        .declaring_file
                        .strip_prefix(&repo_root_canon)
                        .or_else(|_| d.declaring_file.strip_prefix(repo_root))
                        .unwrap_or(&d.declaring_file)
                        .to_string_lossy()
                        .replace('\\', "/");
                    format!("{}:{}", d_rel, d.line_num)
                })
                .collect();
            let msg = format!(
                "{} (Mehrfachdeklaration in {} Stellen: {})",
                rel_to_repo,
                decl_count,
                decl_strs.join(", ")
            );
            if is_known_transition {
                warnings.push(format!("TRANSITION: {}", msg));
            } else {
                errors.push(msg);
            }
        }
    }

    errors.sort();
    warnings.sort();
    ModuleReachabilityResult { errors, warnings }
}

/// Hauptfunktion für xtask: scannt den Workspace auf Erreichbarkeit.
pub fn run_check_module_reachability(root: &Path) -> Result<ModuleReachabilityResult, String> {
    let crate_roots = find_crate_roots(root);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for crate_root in crate_roots {
        let res = check_crate_reachability(&crate_root, root);
        errors.extend(res.errors);
        warnings.extend(res.warnings);
    }

    Ok(ModuleReachabilityResult { errors, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_unreachable_module_detected() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let crate_dir = repo_root.join("crates/my_crate");
        let src_dir = crate_dir.join("src");
        let sub_dir = src_dir.join("sub");
        fs::create_dir_all(&sub_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"my_crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(src_dir.join("lib.rs"), "pub mod sub;\n").unwrap();
        fs::write(sub_dir.join("mod.rs"), "pub mod valid;\n").unwrap();
        fs::write(sub_dir.join("valid.rs"), "// valid\n").unwrap();
        fs::write(sub_dir.join("orphan.rs"), "// orphan\n").unwrap();

        let res = check_crate_reachability(&crate_dir, repo_root);
        assert_eq!(res.errors.len(), 1);
        assert!(res.errors[0].contains("orphan.rs"));
        assert!(res.errors[0].contains("Unerreichbar"));
    }

    #[test]
    fn test_transition_unreachable_gives_warning() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let crate_dir = repo_root.join("crates/contextra-core");
        let src_dir = crate_dir.join("src");
        let traits_dir = src_dir.join("traits");
        fs::create_dir_all(&traits_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"contextra-core\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(src_dir.join("lib.rs"), "pub mod traits;\n").unwrap();
        fs::write(traits_dir.join("mod.rs"), "// traits mod\n").unwrap();
        fs::write(
            traits_dir.join("graph_index.rs"),
            "pub trait GraphIndex {}\n",
        )
        .unwrap();

        let res = check_crate_reachability(&crate_dir, repo_root);
        assert!(
            res.errors.is_empty(),
            "Transition files must not produce hard errors"
        );
        assert_eq!(res.warnings.len(), 1);
        assert!(res.warnings[0].contains("graph_index.rs"));
        assert!(res.warnings[0].contains("TRANSITION"));
    }

    #[test]
    fn test_duplicate_mod_declarations_detected() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let crate_dir = repo_root.join("crates/my_crate");
        let src_dir = crate_dir.join("src");
        let sub_dir = src_dir.join("sub");
        fs::create_dir_all(&sub_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"my_crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(src_dir.join("lib.rs"), "pub mod sub;\n").unwrap();
        fs::write(sub_dir.join("mod.rs"), "pub mod child;\npub mod child;\n").unwrap();
        fs::write(sub_dir.join("child.rs"), "// child\n").unwrap();

        let res = check_crate_reachability(&crate_dir, repo_root);
        assert_eq!(res.errors.len(), 1);
        assert!(res.errors[0].contains("child.rs"));
        assert!(res.errors[0].contains("Mehrfachdeklaration"));
    }
}
