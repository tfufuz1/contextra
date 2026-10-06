//! Workspace Index for xtask tools.
//!
//! Known limitation: Macro-generated code (such as derive macros `#[derive(...)]` or custom `macro_rules!`)
//! is invisible to `syn` at the source text level.

use crate::harness::scan_common::{
    classify_file, resolve_external_test_modules, FileClass,
};
use crate::ws_cache;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use syn::visit::Visit;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub crate_name: String,
    pub class: FileClass,
    pub hash: String,
    pub line_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclInfo {
    pub name: String,
    pub kind: String, // "fn", "struct", "enum", "trait", "type", "const"
    pub crate_name: String,
    pub path: PathBuf,
    pub line: usize,
    pub in_macro_rules: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplInfo {
    pub trait_path: Option<String>,
    pub target_type: String,
    pub crate_name: String,
    pub path: PathBuf,
    pub file_class: FileClass,
    pub cfg_test: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassCounts {
    pub production: usize,
    pub test: usize,
    pub generated: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoDepInfo {
    pub name: String,
    pub section: String, // "normal", "dev", "build", "target"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceIndex {
    pub files: Vec<FileInfo>,
    pub decls: Vec<DeclInfo>,
    pub impls: Vec<ImplInfo>,
    pub ident_refs: HashMap<String, ClassCounts>,
    pub text_refs: HashMap<String, usize>,
    pub cargo_deps: Vec<CargoDepInfo>,
    pub parsed_files_count: usize,
}

impl WorkspaceIndex {
    pub fn build() -> Result<Self, String> {
        let root = crate::find_root_dir();
        Self::build_from_root(&root)
    }

    pub fn build_from_root(root: &Path) -> Result<Self, String> {
        let cache_dir = root.join("target").join("xtask-cache");
        fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create cache dir: {}", e))?;
        let cache_file = cache_dir.join("workspace-index.json");

        let mut old_cache: Option<WorkspaceIndexCacheData> = None;
        if cache_file.exists() {
            if let Ok(content) = fs::read_to_string(&cache_file) {
                old_cache = serde_json::from_str(&content).ok();
            }
        }

        let mut rs_files = Vec::new();
        let crates_dir = root.join("crates");
        if crates_dir.exists() {
            for entry in walkdir::WalkDir::new(&crates_dir)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    name != "target" && name != ".git"
                })
                .filter_map(|e| e.ok())
            {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rs") {
                    rs_files.push(p.to_path_buf());
                }
            }
        }
        rs_files.sort();

        // 1. Discover external test module references via syn
        let mut external_test_files = HashSet::new();
        for file_path in &rs_files {
            if let Ok(content) = ws_cache::read_cached(file_path) {
                if let Ok(file_ast) = syn::parse_file(&content) {
                    resolve_external_test_modules(file_path, &file_ast, &mut external_test_files);
                }
            }
        }

        let mut file_infos = Vec::new();
        let mut file_hashes = HashMap::new();
        let mut decls = Vec::new();
        let mut impls = Vec::new();
        let mut ident_refs: HashMap<String, ClassCounts> = HashMap::new();
        let mut parsed_files_count = 0;

        let mut cached_files_map: HashMap<PathBuf, FileAnalysisCache> = HashMap::new();
        if let Some(ref cache) = old_cache {
            for f in &cache.file_analyses {
                cached_files_map.insert(f.path.clone(), f.clone());
            }
        }

        let mut new_file_analyses = Vec::new();

        for file_path in &rs_files {
            let rel_path = file_path
                .strip_prefix(root)
                .unwrap_or(file_path)
                .to_path_buf();
            let content = match ws_cache::read_cached(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let hash = blake3::hash(content.as_bytes()).to_hex().to_string();
            file_hashes.insert(rel_path.clone(), hash.clone());

            let crate_name = extract_crate_name(&rel_path);
            let class = classify_file(file_path, &content, &external_test_files);
            let line_count = content.lines().count();

            file_infos.push(FileInfo {
                path: rel_path.clone(),
                crate_name: crate_name.clone(),
                class,
                hash: hash.clone(),
                line_count,
            });

            let analysis = if let Some(cached_item) = cached_files_map.get(&rel_path) {
                if cached_item.hash == hash {
                    cached_item.clone()
                } else {
                    parsed_files_count += 1;
                    analyze_file(&rel_path, &content, &crate_name, class, &hash)
                }
            } else {
                parsed_files_count += 1;
                analyze_file(&rel_path, &content, &crate_name, class, &hash)
            };

            decls.extend(analysis.decls.clone());
            impls.extend(analysis.impls.clone());

            for (id, counts) in &analysis.ident_refs {
                let entry = ident_refs.entry(id.clone()).or_default();
                entry.production += counts.production;
                entry.test += counts.test;
                entry.generated += counts.generated;
            }

            new_file_analyses.push(analysis);
        }

        // Non-Rust text refs
        let text_refs = scan_text_refs(root, &ident_refs);

        // Cargo dependencies via cargo metadata
        let cargo_deps = fetch_cargo_deps(root);

        let index = WorkspaceIndex {
            files: file_infos,
            decls,
            impls,
            ident_refs,
            text_refs,
            cargo_deps,
            parsed_files_count,
        };

        let cache_data = WorkspaceIndexCacheData {
            file_analyses: new_file_analyses,
        };
        if let Ok(json) = serde_json::to_string_pretty(&cache_data) {
            let _ = fs::write(&cache_file, json);
        }

        Ok(index)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileAnalysisCache {
    path: PathBuf,
    hash: String,
    decls: Vec<DeclInfo>,
    impls: Vec<ImplInfo>,
    ident_refs: HashMap<String, ClassCounts>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceIndexCacheData {
    file_analyses: Vec<FileAnalysisCache>,
}

fn extract_crate_name(rel_path: &Path) -> String {
    let mut comps = rel_path.components();
    if let Some(c1) = comps.next() {
        if c1.as_os_str() == "crates" {
            if let Some(c2) = comps.next() {
                return c2.as_os_str().to_string_lossy().to_string();
            }
        }
    }
    "".to_string()
}

fn analyze_file(
    rel_path: &Path,
    content: &str,
    crate_name: &str,
    class: FileClass,
    hash: &str,
) -> FileAnalysisCache {
    let mut decls = Vec::new();
    let mut impls = Vec::new();
    let mut ident_refs = HashMap::new();

    if let Ok(file_ast) = syn::parse_file(content) {
        let mut visitor = AstIndexVisitor {
            rel_path,
            crate_name,
            class,
            in_macro_rules: false,
            decls: &mut decls,
            impls: &mut impls,
            ident_refs: &mut ident_refs,
        };
        visitor.visit_file(&file_ast);
    }

    FileAnalysisCache {
        path: rel_path.to_path_buf(),
        hash: hash.to_string(),
        decls,
        impls,
        ident_refs,
    }
}

struct AstIndexVisitor<'a> {
    rel_path: &'a Path,
    crate_name: &'a str,
    class: FileClass,
    in_macro_rules: bool,
    decls: &'a mut Vec<DeclInfo>,
    impls: &'a mut Vec<ImplInfo>,
    ident_refs: &'a mut HashMap<String, ClassCounts>,
}

impl<'a> AstIndexVisitor<'a> {
    fn record_ident(&mut self, ident: &syn::Ident) {
        let s = ident.to_string();
        let entry = self.ident_refs.entry(s).or_default();
        match self.class {
            FileClass::Production => entry.production += 1,
            FileClass::Test => entry.test += 1,
            FileClass::Generated => entry.generated += 1,
        }
    }

    fn line_of_span(&self, span: proc_macro2::Span) -> usize {
        span.start().line
    }
}

impl<'a, 'ast> Visit<'ast> for AstIndexVisitor<'a> {
    fn visit_ident(&mut self, i: &'ast syn::Ident) {
        self.record_ident(i);
        syn::visit::visit_ident(self, i);
    }

    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        if matches!(i.vis, syn::Visibility::Public(_)) {
            self.decls.push(DeclInfo {
                name: i.sig.ident.to_string(),
                kind: "fn".to_string(),
                crate_name: self.crate_name.to_string(),
                path: self.rel_path.to_path_buf(),
                line: self.line_of_span(i.sig.ident.span()),
                in_macro_rules: self.in_macro_rules,
            });
        }
        syn::visit::visit_item_fn(self, i);
    }

    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        if matches!(i.vis, syn::Visibility::Public(_)) {
            self.decls.push(DeclInfo {
                name: i.ident.to_string(),
                kind: "struct".to_string(),
                crate_name: self.crate_name.to_string(),
                path: self.rel_path.to_path_buf(),
                line: self.line_of_span(i.ident.span()),
                in_macro_rules: self.in_macro_rules,
            });
        }
        syn::visit::visit_item_struct(self, i);
    }

    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        if matches!(i.vis, syn::Visibility::Public(_)) {
            self.decls.push(DeclInfo {
                name: i.ident.to_string(),
                kind: "enum".to_string(),
                crate_name: self.crate_name.to_string(),
                path: self.rel_path.to_path_buf(),
                line: self.line_of_span(i.ident.span()),
                in_macro_rules: self.in_macro_rules,
            });
        }
        syn::visit::visit_item_enum(self, i);
    }

    fn visit_item_trait(&mut self, i: &'ast syn::ItemTrait) {
        if matches!(i.vis, syn::Visibility::Public(_)) {
            self.decls.push(DeclInfo {
                name: i.ident.to_string(),
                kind: "trait".to_string(),
                crate_name: self.crate_name.to_string(),
                path: self.rel_path.to_path_buf(),
                line: self.line_of_span(i.ident.span()),
                in_macro_rules: self.in_macro_rules,
            });
        }
        syn::visit::visit_item_trait(self, i);
    }

    fn visit_item_type(&mut self, i: &'ast syn::ItemType) {
        if matches!(i.vis, syn::Visibility::Public(_)) {
            self.decls.push(DeclInfo {
                name: i.ident.to_string(),
                kind: "type".to_string(),
                crate_name: self.crate_name.to_string(),
                path: self.rel_path.to_path_buf(),
                line: self.line_of_span(i.ident.span()),
                in_macro_rules: self.in_macro_rules,
            });
        }
        syn::visit::visit_item_type(self, i);
    }

    fn visit_item_const(&mut self, i: &'ast syn::ItemConst) {
        if matches!(i.vis, syn::Visibility::Public(_)) {
            self.decls.push(DeclInfo {
                name: i.ident.to_string(),
                kind: "const".to_string(),
                crate_name: self.crate_name.to_string(),
                path: self.rel_path.to_path_buf(),
                line: self.line_of_span(i.ident.span()),
                in_macro_rules: self.in_macro_rules,
            });
        }
        syn::visit::visit_item_const(self, i);
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        let trait_path = i.trait_.as_ref().map(|(_, path, _)| {
            path.segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
        });

        let target_type = match &*i.self_ty {
            syn::Type::Path(p) => p
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default(),
            _ => quote::quote!(#i.self_ty).to_string(),
        };

        let cfg_test = crate::harness::scan_common::has_cfg_test_attr(&i.attrs);

        self.impls.push(ImplInfo {
            trait_path,
            target_type,
            crate_name: self.crate_name.to_string(),
            path: self.rel_path.to_path_buf(),
            file_class: self.class,
            cfg_test,
        });

        syn::visit::visit_item_impl(self, i);
    }

    fn visit_item_macro(&mut self, i: &'ast syn::ItemMacro) {
        let is_macro_rules = i.mac.path.is_ident("macro_rules");
        let prev = self.in_macro_rules;
        if is_macro_rules {
            self.in_macro_rules = true;
        }
        syn::visit::visit_item_macro(self, i);
        self.in_macro_rules = prev;
    }
}

fn scan_text_refs(root: &Path, ident_refs: &HashMap<String, ClassCounts>) -> HashMap<String, usize> {
    let mut text_refs = HashMap::new();
    let crates_dir = root.join("crates");
    if !crates_dir.exists() {
        return text_refs;
    }

    let idents: HashSet<&str> = ident_refs.keys().map(|s| s.as_str()).collect();
    if idents.is_empty() {
        return text_refs;
    }

    for entry in walkdir::WalkDir::new(&crates_dir)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target" && name != ".git" && name != "docs"
        })
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.is_file() {
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or_default();
            if ext == "py" || ext == "pyi" || ext == "toml" {
                if let Ok(content) = fs::read_to_string(p) {
                    for ident in &idents {
                        let pattern = format!(r"\b{}\b", regex::escape(ident));
                        if let Ok(re) = Regex::new(&pattern) {
                            let matches = re.find_iter(&content).count();
                            if matches > 0 {
                                *text_refs.entry(ident.to_string()).or_default() += matches;
                            }
                        }
                    }
                }
            }
        }
    }

    text_refs
}

fn fetch_cargo_deps(root: &Path) -> Vec<CargoDepInfo> {
    let mut deps = Vec::new();
    let output = Command::new("cargo")
        .current_dir(root)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                if let Some(packages) = val.get("packages").and_then(|p| p.as_array()) {
                    for pkg in packages {
                        if let Some(pkg_deps) = pkg.get("dependencies").and_then(|d| d.as_array()) {
                            for dep in pkg_deps {
                                let name = dep
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or_default()
                                    .to_string();
                                let kind = dep.get("kind").and_then(|k| k.as_str());
                                let target = dep.get("target").and_then(|t| t.as_str());

                                let section = if kind == Some("dev") {
                                    "dev".to_string()
                                } else if kind == Some("build") {
                                    "build".to_string()
                                } else if target.is_some() {
                                    "target".to_string()
                                } else {
                                    "normal".to_string()
                                };

                                deps.push(CargoDepInfo { name, section });
                            }
                        }
                    }
                }
            }
        }
    }

    deps
}
