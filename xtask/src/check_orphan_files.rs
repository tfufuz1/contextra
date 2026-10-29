//! Gate: check-orphan-files
//! Scans all workspace crates to ensure every `.rs` file under `src/` (and helper modules in `tests/`)
//! is reachable from module declarations (`mod foo;` or `#[path = "..."]`) starting from target roots
//! (`lib.rs`, `main.rs`, `bin/*`, `tests/*`, `benches/*`, `examples/*`, `build.rs`).
//!
//! Unreachable files are compared against `governance/orphan-allowlist.toml`.
//! If any unallowlisted orphan file is found or any allowlist entry is expired, the check fails (exit code != 0).

use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct AllowlistFile {
    #[serde(default)]
    pub orphan: Vec<AllowlistEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AllowlistEntry {
    pub path: String,
    pub reason: String,
    pub owner: String,
    pub expires: String, // YYYY-MM-DD
}

#[derive(Debug, Clone)]
pub struct ModDecl {
    pub declaring_file: PathBuf,
    pub mod_name: String,
    pub custom_path: Option<String>,
    pub inline_path_prefix: PathBuf,
}

struct ModVisitor {
    decls: Vec<ModDecl>,
    declaring_file: PathBuf,
    inline_stack: Vec<String>,
}

impl<'ast> Visit<'ast> for ModVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let mod_name = node.ident.to_string();
        let mut custom_path = None;

        for attr in &node.attrs {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if meta.path.is_ident("path") {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &meta.value
                    {
                        custom_path = Some(s.value());
                    }
                }
            } else if attr.path().is_ident("path") {
                if let Ok(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                })) = attr.parse_args()
                {
                    custom_path = Some(s.value());
                }
            }
        }

        let prefix = self
            .inline_stack
            .iter()
            .fold(PathBuf::new(), |acc, p| acc.join(p));

        if node.content.is_none() {
            // Out-of-line module: `mod foo;`
            self.decls.push(ModDecl {
                declaring_file: self.declaring_file.clone(),
                mod_name,
                custom_path,
                inline_path_prefix: prefix,
            });
        } else {
            // Inline module: `mod foo { ... }`
            if let Some(cp) = custom_path {
                self.decls.push(ModDecl {
                    declaring_file: self.declaring_file.clone(),
                    mod_name: mod_name.clone(),
                    custom_path: Some(cp),
                    inline_path_prefix: prefix.clone(),
                });
            }

            self.inline_stack.push(mod_name);
            syn::visit::visit_item_mod(self, node);
            self.inline_stack.pop();
        }
    }
}

/// Parses a Rust file using `syn` and extracts module declarations.
pub fn extract_mod_declarations(file_path: &Path) -> Vec<ModDecl> {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let syntax = match syn::parse_file(&content) {
        Ok(ast) => ast,
        Err(_) => return Vec::new(),
    };

    let mut visitor = ModVisitor {
        decls: Vec::new(),
        declaring_file: file_path.to_path_buf(),
        inline_stack: Vec::new(),
    };
    visitor.visit_file(&syntax);
    visitor.decls
}

/// Resolves the filesystem target for a `mod foo;` declaration.
pub fn resolve_mod_file(
    decl: &ModDecl,
    src_dir: &Path,
    tests_dir: Option<&Path>,
) -> Option<PathBuf> {
    let parent_file_dir = decl.declaring_file.parent()?;
    let base_parent = parent_file_dir.join(&decl.inline_path_prefix);

    if let Some(ref custom) = decl.custom_path {
        let p = parent_file_dir.join(custom);
        if p.exists() {
            return Some(p.canonicalize().unwrap_or(p));
        }
        let p_base = base_parent.join(custom);
        if p_base.exists() {
            return Some(p_base.canonicalize().unwrap_or(p_base));
        }
        return None;
    }

    let is_root_or_mod_rs =
        if let Some(stem) = decl.declaring_file.file_stem().and_then(|s| s.to_str()) {
            stem == "lib"
                || stem == "main"
                || stem == "mod"
                || decl.declaring_file.parent() == Some(src_dir)
                || tests_dir.map_or(false, |td| decl.declaring_file.parent() == Some(td))
        } else {
            false
        };

    let base_dir = if decl.declaring_file.file_name().and_then(|s| s.to_str()) == Some("mod.rs")
        || decl.declaring_file == src_dir.join("lib.rs")
        || decl.declaring_file == src_dir.join("main.rs")
    {
        base_parent.clone()
    } else {
        if decl.inline_path_prefix.components().next().is_some() {
            base_parent.clone()
        } else {
            let stem = decl.declaring_file.file_stem()?.to_str()?;
            parent_file_dir.join(stem)
        }
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
        let candidate3 = base_parent.join(format!("{}.rs", decl.mod_name));
        if candidate3.is_file() {
            return Some(candidate3.canonicalize().unwrap_or(candidate3));
        }
        let candidate4 = base_parent.join(&decl.mod_name).join("mod.rs");
        if candidate4.is_file() {
            return Some(candidate4.canonicalize().unwrap_or(candidate4));
        }
    }

    None
}

pub struct OrphanCheckResult {
    pub errors: Vec<String>,
    pub orphans_found: Vec<String>,
}

pub fn load_allowlist(allowlist_path: &Path) -> Result<AllowlistFile, String> {
    if !allowlist_path.exists() {
        return Ok(AllowlistFile { orphan: Vec::new() });
    }

    let content = fs::read_to_string(allowlist_path)
        .map_err(|e| format!("Failed to read allowlist {:?}: {}", allowlist_path, e))?;

    let allowlist: AllowlistFile = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse allowlist {:?}: {}", allowlist_path, e))?;

    for entry in &allowlist.orphan {
        if entry.path.trim().is_empty() {
            return Err("Allowlist entry missing required 'path' field".to_string());
        }
        if entry.reason.trim().is_empty() {
            return Err(format!("Allowlist entry '{}' missing 'reason'", entry.path));
        }
        if entry.owner.trim().is_empty() {
            return Err(format!("Allowlist entry '{}' missing 'owner'", entry.path));
        }
        if entry.expires.trim().is_empty() {
            return Err(format!(
                "Allowlist entry '{}' missing 'expires'",
                entry.path
            ));
        }
    }

    Ok(allowlist)
}

/// Checks if an entry's expiration date (YYYY-MM-DD) is in the past relative to today (UTC).
pub fn is_entry_expired(expires: &str, today: &str) -> bool {
    expires.trim() < today.trim()
}

pub fn run_check_orphan_files(root: &Path) -> Result<OrphanCheckResult, String> {
    let allowlist_path = root.join("governance/orphan-allowlist.toml");
    let allowlist = load_allowlist(&allowlist_path)?;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let mut allowed_map: HashMap<String, AllowlistEntry> = HashMap::new();
    let mut errors = Vec::new();

    for entry in allowlist.orphan {
        let norm_path = entry.path.replace('\\', "/");
        if is_entry_expired(&entry.expires, &today) {
            errors.push(format!(
                "EXPIRED ALLOWLIST ENTRY: {} (expired on {}, owner: {})",
                norm_path, entry.expires, entry.owner
            ));
        }
        allowed_map.insert(norm_path, entry);
    }

    // Run cargo metadata to discover all workspace members and targets
    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.current_dir(root);
    let metadata = cmd
        .exec()
        .map_err(|e| format!("Failed to run cargo metadata: {}", e))?;

    let root_canon = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut orphans_found = Vec::new();

    for pkg in metadata.workspace_packages() {
        let manifest_path = PathBuf::from(&pkg.manifest_path);
        let pkg_dir = match manifest_path.parent() {
            Some(p) => p.to_path_buf(),
            None => continue,
        };

        let src_dir = pkg_dir.join("src");
        let tests_dir = pkg_dir.join("tests");

        // Target roots: lib, main, bin/*, tests/*, benches/*, examples/*, build.rs
        let mut target_roots = HashSet::new();
        for target in &pkg.targets {
            let src_path = PathBuf::from(&target.src_path);
            if src_path.exists() {
                if let Ok(canon) = src_path.canonicalize() {
                    target_roots.insert(canon);
                } else {
                    target_roots.insert(src_path);
                }
            }
        }

        // Collect all candidate .rs files in src/ and tests/
        let mut candidate_files = Vec::new();

        if src_dir.exists() {
            let canon_src = src_dir.canonicalize().unwrap_or_else(|_| src_dir.clone());
            for entry in WalkDir::new(&canon_src).into_iter().filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rs") {
                    let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
                    candidate_files.push((canon, canon_src.clone()));
                }
            }
        }

        if tests_dir.exists() {
            let canon_tests = tests_dir
                .canonicalize()
                .unwrap_or_else(|_| tests_dir.clone());
            for entry in WalkDir::new(&canon_tests)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rs") {
                    let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());

                    // Helper modules in tests/ are files in subdirectories (e.g. tests/common/mod.rs)
                    // Root files in tests/ are target roots themselves.
                    if !target_roots.contains(&canon) {
                        candidate_files.push((canon, canon_tests.clone()));
                    }
                }
            }
        }

        // Reachability graph traversal
        let mut queue: Vec<PathBuf> = target_roots.iter().cloned().collect();
        let mut visited: HashSet<PathBuf> = target_roots.clone();

        let tests_dir_opt = if tests_dir.exists() {
            Some(tests_dir.as_path())
        } else {
            None
        };

        while let Some(current_file) = queue.pop() {
            let decls = extract_mod_declarations(&current_file);
            for decl in decls {
                if let Some(target) = resolve_mod_file(&decl, &src_dir, tests_dir_opt) {
                    if visited.insert(target.clone()) {
                        queue.push(target);
                    }
                }
            }
        }

        // Identify unvisited candidate files
        for (file, _base_dir) in candidate_files {
            if !visited.contains(&file) {
                let rel_path = file
                    .strip_prefix(&root_canon)
                    .or_else(|_| file.strip_prefix(root))
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .replace('\\', "/");

                orphans_found.push(rel_path.clone());

                if !allowed_map.contains_key(&rel_path) {
                    errors.push(format!("ORPHAN FILE DETECTED: {}", rel_path));
                }
            }
        }
    }

    orphans_found.sort();
    orphans_found.dedup();
    errors.sort();

    Ok(OrphanCheckResult {
        errors,
        orphans_found,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_extract_mod_declarations_syn_comments_strings() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.rs");

        let content = r#"
            // mod ignored_line_comment;
            /*
               mod ignored_block_comment;
            */
            pub fn foo() {
                let s = "mod ignored_string;";
                let c = 'm';
            }
            #[cfg(target_os = "linux")]
            pub mod reachable_cfg;

            #[path = "custom_dir/custom.rs"]
            pub mod custom_mod;

            pub mod nested_inline {
                pub mod inner_out_of_line;
            }
        "#;

        fs::write(&file_path, content).unwrap();

        let decls = extract_mod_declarations(&file_path);
        let decl_names: Vec<&str> = decls.iter().map(|d| d.mod_name.as_str()).collect();

        assert!(!decl_names.contains(&"ignored_line_comment"));
        assert!(!decl_names.contains(&"ignored_block_comment"));
        assert!(!decl_names.contains(&"ignored_string"));

        assert!(decl_names.contains(&"reachable_cfg"));
        assert!(decl_names.contains(&"custom_mod"));

        let custom_decl = decls.iter().find(|d| d.mod_name == "custom_mod").unwrap();
        assert_eq!(
            custom_decl.custom_path.as_deref(),
            Some("custom_dir/custom.rs")
        );

        assert!(decl_names.contains(&"inner_out_of_line"));
    }

    #[test]
    fn test_orphan_check_fixture_workspace() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Create Cargo.toml
        fs::write(
            root.join("Cargo.toml"),
            r#"
            [workspace]
            members = ["crates/test_crate"]
            "#,
        )
        .unwrap();

        let crate_dir = root.join("crates/test_crate");
        let src_dir = crate_dir.join("src");
        let custom_dir = src_dir.join("custom_dir");
        let nested_dir = src_dir.join("nested_inline");
        fs::create_dir_all(&custom_dir).unwrap();
        fs::create_dir_all(&nested_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
            [package]
            name = "test_crate"
            version = "0.1.0"
            edition = "2021"
            "#,
        )
        .unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            r#"
            #[cfg(feature = "some_feat")]
            pub mod cfg_module;

            #[path = "custom_dir/custom.rs"]
            pub mod custom_path_module;

            pub mod nested_inline {
                pub mod inner;
            }
            "#,
        )
        .unwrap();

        fs::write(src_dir.join("cfg_module.rs"), "// cfg module").unwrap();
        fs::write(custom_dir.join("custom.rs"), "// custom path").unwrap();
        fs::write(nested_dir.join("inner.rs"), "// inner module").unwrap();
        fs::write(src_dir.join("unreachable_orphan.rs"), "// orphan file").unwrap();

        // Governance dir with allowlist
        let gov_dir = root.join("governance");
        fs::create_dir_all(&gov_dir).unwrap();

        fs::write(
            gov_dir.join("orphan-allowlist.toml"),
            r#"
            [[orphan]]
            path = "crates/test_crate/src/unreachable_orphan.rs"
            reason = "Test orphan allowed"
            owner = "MP-05"
            expires = "2029-12-31"
            "#,
        )
        .unwrap();

        let res = run_check_orphan_files(root).unwrap();
        assert!(
            res.errors.is_empty(),
            "Expected 0 errors when orphan is allowlisted, got: {:?}",
            res.errors
        );
        assert!(res
            .orphans_found
            .contains(&"crates/test_crate/src/unreachable_orphan.rs".to_string()));

        // Add an unallowlisted orphan file
        fs::write(src_dir.join("unallowed_orphan.rs"), "// unallowed").unwrap();
        let res2 = run_check_orphan_files(root).unwrap();
        assert_eq!(res2.errors.len(), 1);
        assert!(res2.errors[0].contains("unallowed_orphan.rs"));
    }

    #[test]
    fn test_expired_allowlist_entry_fails() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let gov_dir = root.join("governance");
        fs::create_dir_all(&gov_dir).unwrap();

        fs::write(
            gov_dir.join("orphan-allowlist.toml"),
            r#"
            [[orphan]]
            path = "crates/test_crate/src/expired_orphan.rs"
            reason = "Expired test orphan"
            owner = "MP-05"
            expires = "2020-01-01"
            "#,
        )
        .unwrap();

        let is_exp = is_entry_expired("2020-01-01", "2026-10-29");
        assert!(is_exp);

        let allowlist = load_allowlist(&gov_dir.join("orphan-allowlist.toml")).unwrap();
        assert_eq!(allowlist.orphan.len(), 1);
        assert_eq!(allowlist.orphan[0].expires, "2020-01-01");
    }
}
