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
    "crates/contextra-checkpoint/src/guard/tests.rs",
    "crates/contextra-checkpoint/src/meta/tests.rs",
    "crates/contextra-checkpoint/src/orphan/tests.rs",
    "crates/contextra-infer-ollama/src/query_rewriter.rs",
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
            } else if i + 4 < len
                && chars[i + 1] == '\\'
                && chars[i + 2] == 'x'
                && chars[i + 4] == '\''
            {
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

    let lines_raw: Vec<&str> = content.lines().collect();
    let stripped = strip_comments_and_strings(&content);
    let lines_stripped: Vec<&str> = stripped.lines().collect();

    let mod_re = match regex::Regex::new(r"^\s*(pub(\([^)]+\))?\s+)?mod\s+([a-zA-Z0-9_]+)\s*;") {
        Ok(r) => r,
        Err(_) => return decls,
    };

    let path_re = match regex::Regex::new(r#"^\s*#\s*\[\s*path\s*=\s*"([^"]+)"\s*\]"#) {
        Ok(r) => r,
        Err(_) => return decls,
    };

    let mut pending_path: Option<String> = None;

    for (line_idx, line_stripped) in lines_stripped.iter().enumerate() {
        let line_raw = lines_raw.get(line_idx).copied().unwrap_or(line_stripped);

        if let Some(caps) = path_re.captures(line_raw) {
            pending_path = Some(caps[1].to_string());
            continue;
        }

        if let Some(caps) = mod_re.captures(line_stripped) {
            decls.push(ModDecl {
                declaring_file: file_path.to_path_buf(),
                line_num: line_idx + 1,
                mod_name: caps[3].to_string(),
                custom_path: pending_path.take(),
            });
        } else {
            let line_trimmed = line_stripped.trim();
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

/// Prüft, ob ein Unterverzeichnis im übergeordneten Modul dekariert ist.
fn is_dir_declared(dir: &Path, src_dir: &Path, all_declarations: &[ModDecl]) -> bool {
    let dir_name = match dir.file_name().and_then(|s| s.to_str()) {
        Some(n) => n,
        None => return true,
    };

    let parent_dir = match dir.parent() {
        Some(p) => p,
        None => return true,
    };

    let parent_files = if parent_dir == src_dir {
        vec![
            src_dir.join("lib.rs"),
            src_dir.join("main.rs"),
            src_dir.join("mod.rs"),
        ]
    } else {
        let mut files = vec![parent_dir.join("mod.rs")];
        if let Some(parent_stem) = parent_dir.file_stem().and_then(|s| s.to_str()) {
            if let Some(grandparent) = parent_dir.parent() {
                files.push(grandparent.join(format!("{}.rs", parent_stem)));
            }
        }
        files
    };

    let canonical_dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());

    for decl in all_declarations {
        let declaring_file_canon = decl
            .declaring_file
            .canonicalize()
            .unwrap_or_else(|_| decl.declaring_file.clone());

        let is_parent_file = parent_files.iter().any(|pf| {
            pf.canonicalize()
                .map(|pfc| pfc == declaring_file_canon)
                .unwrap_or_else(|_| pf == &decl.declaring_file)
        });

        if is_parent_file {
            if decl.mod_name == dir_name {
                return true;
            }
            if let Some(target) = resolve_mod_file(decl, src_dir) {
                let target_canon = target.canonicalize().unwrap_or(target);
                if target_canon == canonical_dir || target_canon.starts_with(&canonical_dir) {
                    return true;
                }
            }
        }
    }

    false
}

/// Scannt ein Crate auf Erreichbarkeit aller `.rs`-Dateien und -Unterverzeichnisse unter `src/`.
pub fn check_crate_reachability(crate_root: &Path, repo_root: &Path) -> ModuleReachabilityResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let raw_src_dir = crate_root.join("src");
    if !raw_src_dir.exists() {
        return ModuleReachabilityResult { errors, warnings };
    }
    let src_dir = raw_src_dir.canonicalize().unwrap_or(raw_src_dir);

    let mut all_rs_files = Vec::new();
    let mut all_dirs = Vec::new();
    for entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rs") {
            let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
            all_rs_files.push(canon);
        } else if p.is_dir() && p != src_dir {
            let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
            all_dirs.push(canon);
        }
    }

    let mut all_declarations = Vec::new();
    for file in &all_rs_files {
        let decls = extract_mod_declarations(file);
        all_declarations.extend(decls);
    }

    let repo_root_canon = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());

    let mut orphan_dirs = std::collections::HashSet::new();

    for dir in &all_dirs {
        let rel_to_src = match dir.strip_prefix(&src_dir) {
            Ok(rel) => rel,
            Err(_) => continue,
        };

        if rel_to_src.starts_with("bin") {
            continue;
        }

        let contains_rs = WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| e.path().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("rs"));

        if !contains_rs {
            continue;
        }

        if !is_dir_declared(dir, &src_dir, &all_declarations) {
            orphan_dirs.insert(dir.clone());

            let rel_to_repo = dir
                .strip_prefix(&repo_root_canon)
                .or_else(|_| dir.strip_prefix(repo_root))
                .unwrap_or(dir)
                .to_string_lossy()
                .replace('\\', "/");

            let is_known_transition = KNOWN_TRANSITION_UNREACHABLE
                .iter()
                .any(|&t| t == rel_to_repo);

            let msg = format!("{} (Unerreichbar: 0 mod-Deklarationen)", rel_to_repo);
            if is_known_transition {
                warnings.push(format!("TRANSITION: {}", msg));
            } else {
                errors.push(msg);
            }
        }
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

        let is_known_transition = KNOWN_TRANSITION_UNREACHABLE
            .iter()
            .any(|&t| t == rel_to_repo);

        let is_harness_module = if let Ok(rel_to_src) = file.strip_prefix(&src_dir) {
            rel_to_src.starts_with("harness") && file_name != "mod.rs"
        } else {
            false
        };

        let raw_decls = target_decl_map.get(file);
        let mut decls_vec = raw_decls.cloned().unwrap_or_default();

        if is_harness_module && decls_vec.is_empty() {
            let harness_mod = src_dir.join("harness").join("mod.rs");
            if harness_mod.exists() {
                decls_vec.push(ModDecl {
                    declaring_file: harness_mod,
                    line_num: 1,
                    mod_name: file.file_stem().unwrap().to_string_lossy().to_string(),
                    custom_path: None,
                });
            }
        }

        let decl_count = decls_vec.len();

        if decl_count == 0 {
            let inside_orphan_dir = orphan_dirs.iter().any(|od| file.starts_with(od));
            if !inside_orphan_dir {
                let msg = format!("{} (Unerreichbar: 0 mod-Deklarationen)", rel_to_repo);
                if is_known_transition {
                    warnings.push(format!("TRANSITION: {}", msg));
                } else {
                    errors.push(msg);
                }
            }
        } else if decl_count > 1 {
            // Check if all declaring files are distinct root entry points (e.g. lib.rs vs main.rs)
            // with at most 1 declaration per root file.
            let mut is_valid_split_root = true;
            let mut seen_declaring_files = std::collections::HashSet::new();

            for d in &decls_vec {
                let d_file_name = d.declaring_file.file_name().and_then(|s| s.to_str());
                let is_root_decl_file = match d_file_name {
                    Some("lib.rs") | Some("main.rs") => true,
                    _ => {
                        if let Ok(rel) = d.declaring_file.strip_prefix(&src_dir) {
                            rel.starts_with("bin")
                        } else {
                            false
                        }
                    }
                };

                if !is_root_decl_file || !seen_declaring_files.insert(&d.declaring_file) {
                    is_valid_split_root = false;
                    break;
                }
            }

            if !is_valid_split_root {
                let decl_strs: Vec<String> = decls_vec
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
        let crate_dir = repo_root.join("crates/contextra-checkpoint");
        let src_dir = crate_dir.join("src");
        let guard_dir = src_dir.join("guard");
        fs::create_dir_all(&guard_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"contextra-checkpoint\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(src_dir.join("lib.rs"), "pub mod guard;\n").unwrap();
        fs::write(guard_dir.join("mod.rs"), "// guard mod\n").unwrap();
        fs::write(guard_dir.join("tests.rs"), "// guard tests\n").unwrap();

        let res = check_crate_reachability(&crate_dir, repo_root);
        assert!(
            res.errors.is_empty(),
            "Transition files must not produce hard errors"
        );
        assert_eq!(res.warnings.len(), 1);
        assert!(res.warnings[0].contains("tests.rs"));
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

    #[test]
    fn test_orphan_directory_detected() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let crate_dir = repo_root.join("crates/my_crate");
        let src_dir = crate_dir.join("src");
        let orphan_dir = src_dir.join("orphan_dir");
        fs::create_dir_all(&orphan_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"my_crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(src_dir.join("lib.rs"), "// no mod orphan_dir declaration\n").unwrap();
        fs::write(orphan_dir.join("mod.rs"), "pub mod helper;\n").unwrap();
        fs::write(orphan_dir.join("helper.rs"), "// helper\n").unwrap();

        let res = check_crate_reachability(&crate_dir, repo_root);
        assert!(!res.errors.is_empty());
        assert!(res.errors.iter().any(|e| e.contains("orphan_dir")));
    }

    #[test]
    fn test_valid_directory_detected() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let crate_dir = repo_root.join("crates/my_crate");
        let src_dir = crate_dir.join("src");
        let valid_dir = src_dir.join("valid_dir");
        fs::create_dir_all(&valid_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"my_crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(src_dir.join("lib.rs"), "pub mod valid_dir;\n").unwrap();
        fs::write(valid_dir.join("mod.rs"), "pub mod child;\n").unwrap();
        fs::write(valid_dir.join("child.rs"), "// child\n").unwrap();

        let res = check_crate_reachability(&crate_dir, repo_root);
        assert!(res.errors.is_empty());
    }

    #[test]
    fn test_cfg_gated_directory_module() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let crate_dir = repo_root.join("crates/my_crate");
        let src_dir = crate_dir.join("src");
        let cfg_dir = src_dir.join("cfg_dir");
        fs::create_dir_all(&cfg_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"my_crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(src_dir.join("lib.rs"), "#[cfg(feature = \"my_feat\")]\npub mod cfg_dir;\n").unwrap();
        fs::write(cfg_dir.join("mod.rs"), "// cfg mod\n").unwrap();

        let res = check_crate_reachability(&crate_dir, repo_root);
        assert!(res.errors.is_empty());
    }

    #[test]
    fn test_path_attribute_directory_module() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let crate_dir = repo_root.join("crates/my_crate");
        let src_dir = crate_dir.join("src");
        let custom_dir = src_dir.join("custom_path_dir");
        fs::create_dir_all(&custom_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"my_crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            "#[path = \"custom_path_dir/custom_mod.rs\"]\npub mod custom;\n",
        )
        .unwrap();
        fs::write(custom_dir.join("custom_mod.rs"), "// custom mod\n").unwrap();

        let res = check_crate_reachability(&crate_dir, repo_root);
        assert!(res.errors.is_empty(), "Errors found: {:?}", res.errors);
    }
}
