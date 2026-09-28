//! Symbol Exists Gate: Löste Pfade wie `contextra_store::wal::WalWriter` auf und verifiziert deren Existenz.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use syn::{Item, UseTree, Visibility};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SymbolExistsFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolExistsGateResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<SymbolExistsFinding>,
}

#[derive(Debug, Clone)]
pub struct SymbolExistsItemInfo {
    pub kind: String,
    pub name: String,
    pub visibility: String,
    pub file: String,
    pub line: usize,
    pub signature: String,
    pub parent_trait: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SymbolExistsCapabilitiesManifest {
    #[serde(default)]
    pub crates: HashMap<String, SymbolExistsCrateInfo>,
}

#[derive(Debug, Deserialize)]
pub struct SymbolExistsCrateInfo {
    pub path: String,
}

pub fn symbol_exists_lev_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    let mut matrix = vec![vec![0; v2.len() + 1]; v1.len() + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(v1.len() + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(v2.len() + 1) {
        *cell = j;
    }

    for i in 1..=v1.len() {
        for j in 1..=v2.len() {
            let cost = if v1[i - 1] == v2[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }
    matrix[v1.len()][v2.len()]
}

pub fn symbol_exists_normalize_crate_name(name: &str) -> String {
    name.replace('-', "_")
}

pub fn symbol_exists_find_crate_path(root: &Path, crate_name: &str) -> Option<PathBuf> {
    let caps_path = root.join("capabilities.toml");
    let target = symbol_exists_normalize_crate_name(crate_name);

    if let Ok(content) = fs::read_to_string(&caps_path) {
        if let Ok(manifest) = toml::from_str::<SymbolExistsCapabilitiesManifest>(&content) {
            for (c_name, info) in manifest.crates {
                if symbol_exists_normalize_crate_name(&c_name) == target {
                    return Some(root.join(info.path));
                }
            }
        }
    }

    // Fallback: check crates/<name> or crates/<normalized_name>
    let p1 = root.join("crates").join(crate_name);
    if p1.exists() {
        return Some(p1);
    }
    let p2 = root.join("crates").join(&target);
    if p2.exists() {
        return Some(p2);
    }
    None
}

pub fn run_symbol_exists(args: &[String]) -> i32 {
    let mut root_dir = symbol_exists_default_root();
    let mut json_output = false;
    let mut symbol_arg: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                if i + 1 < args.len() {
                    root_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--json" => {
                json_output = true;
            }
            _ => {
                if !args[i].starts_with('-') && symbol_arg.is_none() {
                    symbol_arg = Some(args[i].clone());
                }
            }
        }
        i += 1;
    }

    let raw_symbol = match symbol_arg {
        Some(s) => s,
        None => {
            let res = SymbolExistsGateResult {
                gate: "symbol-exists".to_string(),
                status: "error".to_string(),
                summary: "Kein Symbol angegeben. Nutzung: symbol-exists <crate::pfad::Item> [--json]".to_string(),
                findings: vec![],
            };
            return symbol_exists_emit(res, json_output, 2);
        }
    };

    let segments: Vec<&str> = raw_symbol.split("::").collect();
    if segments.len() < 2 {
        let res = SymbolExistsGateResult {
            gate: "symbol-exists".to_string(),
            status: "error".to_string(),
            summary: format!("Ungültiger Symbolpfad '{}'. Erwartet: <crate::pfad::Item>", raw_symbol),
            findings: vec![],
        };
        return symbol_exists_emit(res, json_output, 2);
    }

    let crate_name = segments[0];
    let crate_dir = match symbol_exists_find_crate_path(&root_dir, crate_name) {
        Some(p) => p,
        None => {
            let res = SymbolExistsGateResult {
                gate: "symbol-exists".to_string(),
                status: "error".to_string(),
                summary: format!("Crate '{}' konnte in capabilities.toml/crates/ nicht gefunden werden", crate_name),
                findings: vec![],
            };
            return symbol_exists_emit(res, json_output, 2);
        }
    };

    let mut finder = SymbolExistsCrateFinder::new(crate_dir);
    finder.scan_crate();

    let target_symbol = segments[1..].join("::");
    if let Some(info) = finder.lookup_symbol(&target_symbol) {
        let summary = if let Some(ref tr) = info.parent_trait {
            format!(
                "Symbol '{}' (Kind: {}, Name: {}, Sichtbarkeit: {}) in Trait '{}' gefunden: {} in {}:{}",
                raw_symbol, info.kind, info.name, info.visibility, tr, info.signature, info.file, info.line
            )
        } else {
            format!(
                "Symbol '{}' (Kind: {}, Name: {}, Sichtbarkeit: {}) gefunden: {} in {}:{}",
                raw_symbol, info.kind, info.name, info.visibility, info.signature, info.file, info.line
            )
        };
        let res = SymbolExistsGateResult {
            gate: "symbol-exists".to_string(),
            status: "pass".to_string(),
            summary,
            findings: vec![],
        };
        symbol_exists_emit(res, json_output, 0)
    } else {
        let suggestions = finder.get_similar_candidates(&target_symbol, 5);
        let msg = if suggestions.is_empty() {
            format!("Symbol '{}' wurde in Crate '{}' nicht gefunden.", raw_symbol, crate_name)
        } else {
            format!(
                "Symbol '{}' wurde in Crate '{}' nicht gefunden. Ähnliche Symbole: {}",
                raw_symbol,
                crate_name,
                suggestions.join(", ")
            )
        };

        let res = SymbolExistsGateResult {
            gate: "symbol-exists".to_string(),
            status: "fail".to_string(),
            summary: msg.clone(),
            findings: vec![SymbolExistsFinding {
                id: "SE-001".to_string(),
                severity: "error".to_string(),
                file: "".to_string(),
                line: 0,
                message: msg,
                fix: "Symbol-Name oder Modulpfad prüfen".to_string(),
            }],
        };
        symbol_exists_emit(res, json_output, 1)
    }
}

pub struct SymbolExistsCrateFinder {
    pub crate_dir: PathBuf,
    pub items: HashMap<String, SymbolExistsItemInfo>,
    pub reexports: HashMap<String, String>, // alias -> target path
    pub glob_reexports: Vec<(String, String)>, // (mod_path, target_prefix)
    pub visited_files: HashSet<PathBuf>,
}

impl SymbolExistsCrateFinder {
    pub fn new(crate_dir: PathBuf) -> Self {
        Self {
            crate_dir,
            items: HashMap::new(),
            reexports: HashMap::new(),
            glob_reexports: Vec::new(),
            visited_files: HashSet::new(),
        }
    }

    pub fn scan_crate(&mut self) {
        let src_dir = self.crate_dir.join("src");
        let lib_rs = src_dir.join("lib.rs");
        let main_rs = src_dir.join("main.rs");

        if lib_rs.exists() {
            self.parse_module_file(&lib_rs, "");
        } else if main_rs.exists() {
            self.parse_module_file(&main_rs, "");
        }

        // Trace re-exports up to fixpoint (max 10 passes)
        for _pass in 0..10 {
            let mut added_any = false;

            // Expand glob re-exports
            let mut new_glob_items = HashMap::new();
            for (mod_path, target_prefix) in &self.glob_reexports {
                let prefix_pattern = if target_prefix.is_empty() {
                    String::new()
                } else {
                    format!("{}::", target_prefix)
                };

                for (key, item) in &self.items {
                    let matches = if prefix_pattern.is_empty() {
                        !key.contains("::")
                    } else if key.starts_with(&prefix_pattern) {
                        let rel = &key[prefix_pattern.len()..];
                        !rel.contains("::")
                    } else {
                        false
                    };

                    if matches {
                        let rel = if prefix_pattern.is_empty() {
                            key.as_str()
                        } else {
                            &key[prefix_pattern.len()..]
                        };
                        let alias_full = join_mod_path(mod_path, rel);
                        if !self.items.contains_key(&alias_full) && !new_glob_items.contains_key(&alias_full) {
                            new_glob_items.insert(alias_full, item.clone());
                        }
                    }
                }
            }

            if !new_glob_items.is_empty() {
                added_any = true;
                self.items.extend(new_glob_items);
            }

            // Expand named re-exports
            let mut new_items = HashMap::new();
            for (alias, target) in &self.reexports {
                if !self.items.contains_key(alias) {
                    if let Some(item) = self.items.get(target).cloned() {
                        new_items.insert(alias.clone(), item);
                    }
                }
            }

            if !new_items.is_empty() {
                added_any = true;
                self.items.extend(new_items);
            }

            if !added_any {
                break;
            }
        }
    }

    pub fn parse_module_file(&mut self, file_path: &Path, mod_path: &str) {
        let canon = file_path.canonicalize().unwrap_or_else(|_| file_path.to_path_buf());
        if self.visited_files.contains(&canon) {
            return;
        }
        self.visited_files.insert(canon);

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let ast = match syn::parse_file(&content) {
            Ok(a) => a,
            Err(_) => return,
        };

        let rel_file = file_path
            .strip_prefix(&self.crate_dir)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        for item in &ast.items {
            self.register_item(item, file_path, &rel_file, mod_path);
        }
    }

    fn register_item(&mut self, item: &Item, file_path: &Path, rel_file: &str, mod_path: &str) {
        match item {
            Item::Struct(s) => {
                let name = s.ident.to_string();
                let full = join_mod_path(mod_path, &name);
                let info = SymbolExistsItemInfo {
                    kind: "struct".to_string(),
                    name: name.clone(),
                    visibility: format_vis(&s.vis),
                    file: rel_file.to_string(),
                    line: 1,
                    signature: format!("pub struct {}", name),
                    parent_trait: None,
                };
                self.items.insert(full, info);
            }
            Item::Enum(e) => {
                let name = e.ident.to_string();
                let full = join_mod_path(mod_path, &name);
                let info = SymbolExistsItemInfo {
                    kind: "enum".to_string(),
                    name: name.clone(),
                    visibility: format_vis(&e.vis),
                    file: rel_file.to_string(),
                    line: 1,
                    signature: format!("pub enum {}", name),
                    parent_trait: None,
                };
                self.items.insert(full, info);
            }
            Item::Trait(t) => {
                let name = t.ident.to_string();
                let full = join_mod_path(mod_path, &name);
                let info = SymbolExistsItemInfo {
                    kind: "trait".to_string(),
                    name: name.clone(),
                    visibility: format_vis(&t.vis),
                    file: rel_file.to_string(),
                    line: 1,
                    signature: format!("pub trait {}", name),
                    parent_trait: None,
                };
                self.items.insert(full.clone(), info);

                // Register trait methods
                for trait_item in &t.items {
                    if let syn::TraitItem::Fn(tf) = trait_item {
                        let m_name = tf.sig.ident.to_string();
                        let m_full = format!("{}::{}", full, m_name);
                        let m_info = SymbolExistsItemInfo {
                            kind: "fn".to_string(),
                            name: m_name.clone(),
                            visibility: "pub".to_string(),
                            file: rel_file.to_string(),
                            line: 1,
                            signature: quote::quote!(#tf).to_string(),
                            parent_trait: Some(name.clone()),
                        };
                        self.items.insert(m_full, m_info);
                    }
                }
            }
            Item::Fn(f) => {
                let name = f.sig.ident.to_string();
                let full = join_mod_path(mod_path, &name);
                let info = SymbolExistsItemInfo {
                    kind: "fn".to_string(),
                    name: name.clone(),
                    visibility: format_vis(&f.vis),
                    file: rel_file.to_string(),
                    line: 1,
                    signature: quote::quote!(#f).to_string(),
                    parent_trait: None,
                };
                self.items.insert(full, info);
            }
            Item::Type(ty) => {
                let name = ty.ident.to_string();
                let full = join_mod_path(mod_path, &name);
                let info = SymbolExistsItemInfo {
                    kind: "type".to_string(),
                    name: name.clone(),
                    visibility: format_vis(&ty.vis),
                    file: rel_file.to_string(),
                    line: 1,
                    signature: quote::quote!(#ty).to_string(),
                    parent_trait: None,
                };
                self.items.insert(full, info);
            }
            Item::Const(c) => {
                let name = c.ident.to_string();
                let full = join_mod_path(mod_path, &name);
                let info = SymbolExistsItemInfo {
                    kind: "const".to_string(),
                    name: name.clone(),
                    visibility: format_vis(&c.vis),
                    file: rel_file.to_string(),
                    line: 1,
                    signature: quote::quote!(#c).to_string(),
                    parent_trait: None,
                };
                self.items.insert(full, info);
            }
            Item::Static(st) => {
                let name = st.ident.to_string();
                let full = join_mod_path(mod_path, &name);
                let info = SymbolExistsItemInfo {
                    kind: "static".to_string(),
                    name: name.clone(),
                    visibility: format_vis(&st.vis),
                    file: rel_file.to_string(),
                    line: 1,
                    signature: quote::quote!(#st).to_string(),
                    parent_trait: None,
                };
                self.items.insert(full, info);
            }
            Item::Impl(imp) => {
                let self_ty = &imp.self_ty;
                let type_name = quote::quote!(#self_ty).to_string();
                let clean_type = type_name.split('<').next().unwrap_or(&type_name).trim().to_string();

                for impl_item in &imp.items {
                    if let syn::ImplItem::Fn(im_fn) = impl_item {
                        let method_name = im_fn.sig.ident.to_string();
                        let method_full = join_mod_path(mod_path, &format!("{}::{}", clean_type, method_name));
                        let info = SymbolExistsItemInfo {
                            kind: "fn".to_string(),
                            name: method_name,
                            visibility: format_vis(&im_fn.vis),
                            file: rel_file.to_string(),
                            line: 1,
                            signature: quote::quote!(#im_fn).to_string(),
                            parent_trait: imp.trait_.as_ref().map(|(_, path, _)| quote::quote!(#path).to_string()),
                        };
                        self.items.insert(method_full, info);
                    }
                }
            }
            Item::Mod(m) => {
                let mod_name = m.ident.to_string();
                let new_mod_path = join_mod_path(mod_path, &mod_name);

                let info = SymbolExistsItemInfo {
                    kind: "mod".to_string(),
                    name: mod_name.clone(),
                    visibility: format_vis(&m.vis),
                    file: rel_file.to_string(),
                    line: 1,
                    signature: format!("mod {}", mod_name),
                    parent_trait: None,
                };
                self.items.insert(new_mod_path.clone(), info);

                if let Some((_, ref items)) = m.content {
                    for sub_item in items {
                        self.register_item(sub_item, file_path, rel_file, &new_mod_path);
                    }
                } else {
                    // Inline mod without body => resolve external file
                    let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let file_stem = file_path.file_stem().and_then(|n| n.to_str()).unwrap_or("");

                    let parent_dir = if file_name == "mod.rs" || file_name == "lib.rs" || file_name == "main.rs" {
                        file_path.parent().unwrap_or(Path::new(".")).to_path_buf()
                    } else {
                        file_path.parent().unwrap_or(Path::new(".")).join(file_stem)
                    };

                    let sub_f1 = parent_dir.join(format!("{}.rs", mod_name));
                    let sub_f2 = parent_dir.join(&mod_name).join("mod.rs");

                    if sub_f1.exists() {
                        self.parse_module_file(&sub_f1, &new_mod_path);
                    } else if sub_f2.exists() {
                        self.parse_module_file(&sub_f2, &new_mod_path);
                    }
                }
            }
            Item::Use(u) => {
                self.register_use_tree(&u.tree, mod_path, mod_path);
            }
            _ => {}
        }
    }

    fn register_use_tree(&mut self, tree: &UseTree, mod_path: &str, prefix: &str) {
        match tree {
            UseTree::Path(p) => {
                let segment = p.ident.to_string();
                let new_prefix = join_mod_path(prefix, &segment);
                self.register_use_tree(&p.tree, mod_path, &new_prefix);
            }
            UseTree::Name(n) => {
                let name = n.ident.to_string();
                let alias_full = join_mod_path(mod_path, &name);
                let target_full = join_mod_path(prefix, &name);
                self.reexports.insert(alias_full, target_full);
            }
            UseTree::Rename(r) => {
                let alias = r.rename.to_string();
                let orig = r.ident.to_string();
                let alias_full = join_mod_path(mod_path, &alias);
                let target_full = join_mod_path(prefix, &orig);
                self.reexports.insert(alias_full, target_full);
            }
            UseTree::Group(g) => {
                for item in &g.items {
                    self.register_use_tree(item, mod_path, prefix);
                }
            }
            UseTree::Glob(_) => {
                self.glob_reexports.push((mod_path.to_string(), prefix.to_string()));
            }
        }
    }

    pub fn lookup_symbol(&self, symbol_path: &str) -> Option<&SymbolExistsItemInfo> {
        if let Some(item) = self.items.get(symbol_path) {
            return Some(item);
        }
        None
    }

    pub fn get_similar_candidates(&self, target_symbol: &str, limit: usize) -> Vec<String> {
        let mut candidates: Vec<(&String, usize)> = self
            .items
            .keys()
            .map(|k| (k, symbol_exists_lev_distance(k, target_symbol)))
            .collect();

        candidates.sort_by(|a, b| a.1.cmp(&b.1));
        candidates
            .into_iter()
            .take(limit)
            .map(|(k, _)| k.clone())
            .collect()
    }
}

fn join_mod_path(mod_path: &str, name: &str) -> String {
    if mod_path.is_empty() {
        name.to_string()
    } else {
        format!("{}::{}", mod_path, name)
    }
}

fn format_vis(vis: &Visibility) -> String {
    match vis {
        Visibility::Public(_) => "pub".to_string(),
        Visibility::Restricted(_) => "pub(crate)".to_string(),
        Visibility::Inherited => "private".to_string(),
    }
}

fn symbol_exists_emit(res: SymbolExistsGateResult, json_output: bool, exit_code: i32) -> i32 {
    if json_output {
        println!("{}", serde_json::to_string(&res).unwrap_or_default());
    } else {
        println!("=== Gate symbol-exists: {} ===", res.status);
        println!("{}", res.summary);
        for f in &res.findings {
            println!(
                "[{}] {}: {}\n  Fix: {}",
                f.severity.to_uppercase(),
                f.file,
                f.message,
                f.fix
            );
        }
    }
    exit_code
}

fn symbol_exists_default_root() -> PathBuf {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return PathBuf::from(path);
        }
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
