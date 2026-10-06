//! Harness Modul: findet pub traits ohne Implementierung oder ohne Verdrahtung in einem Composition-Root-Crate (unwired-ports).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use syn::visit::Visit;
use syn::{ItemEnum, ItemImpl, ItemStruct, ItemTrait, ItemType, ItemUnion, Visibility};

use crate::check_ring_layering::{get_ring_map_from_metadata_json, get_workspace_ring_map, Ring};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnwiredPortsFinding {
    pub severity: String,
    pub trait_name: String,
    pub implementing_types: Vec<String>,
    pub defined_in: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnwiredPortsGateResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<UnwiredPortsFinding>,
}

#[derive(Debug, Clone)]
struct TraitDecl {
    trait_name: String,
    #[allow(dead_code)]
    crate_name: String,
    file_path: String,
}

#[derive(Debug, Clone)]
struct ImplDecl {
    trait_name: String,
    type_name: String,
    #[allow(dead_code)]
    crate_name: String,
    file_path: String,
}

#[derive(Debug, Clone)]
struct TypeDecl {
    type_name: String,
    file_path: String,
}

struct CodeVisitor<'a> {
    crate_name: &'a str,
    file_path: &'a str,
    traits: Vec<TraitDecl>,
    impls: Vec<ImplDecl>,
    type_defs: Vec<TypeDecl>,
}

impl<'a> CodeVisitor<'a> {
    fn new(crate_name: &'a str, file_path: &'a str) -> Self {
        Self {
            crate_name,
            file_path,
            traits: Vec::new(),
            impls: Vec::new(),
            type_defs: Vec::new(),
        }
    }
}

impl<'ast, 'a> Visit<'ast> for CodeVisitor<'a> {
    fn visit_item_trait(&mut self, i: &'ast ItemTrait) {
        if matches!(i.vis, Visibility::Public(_)) {
            self.traits.push(TraitDecl {
                trait_name: i.ident.to_string(),
                crate_name: self.crate_name.to_string(),
                file_path: self.file_path.to_string(),
            });
        }
        syn::visit::visit_item_trait(self, i);
    }

    fn visit_item_impl(&mut self, i: &'ast ItemImpl) {
        if let Some((_, path, _)) = &i.trait_ {
            if let Some(trait_seg) = path.segments.last() {
                let trait_name = trait_seg.ident.to_string();
                if let Some(type_name) = extract_type_name(&i.self_ty) {
                    self.impls.push(ImplDecl {
                        trait_name,
                        type_name,
                        crate_name: self.crate_name.to_string(),
                        file_path: self.file_path.to_string(),
                    });
                }
            }
        }
        syn::visit::visit_item_impl(self, i);
    }

    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        self.type_defs.push(TypeDecl {
            type_name: i.ident.to_string(),
            file_path: self.file_path.to_string(),
        });
        syn::visit::visit_item_struct(self, i);
    }

    fn visit_item_enum(&mut self, i: &'ast ItemEnum) {
        self.type_defs.push(TypeDecl {
            type_name: i.ident.to_string(),
            file_path: self.file_path.to_string(),
        });
        syn::visit::visit_item_enum(self, i);
    }

    fn visit_item_type(&mut self, i: &'ast ItemType) {
        self.type_defs.push(TypeDecl {
            type_name: i.ident.to_string(),
            file_path: self.file_path.to_string(),
        });
        syn::visit::visit_item_type(self, i);
    }

    fn visit_item_union(&mut self, i: &'ast ItemUnion) {
        self.type_defs.push(TypeDecl {
            type_name: i.ident.to_string(),
            file_path: self.file_path.to_string(),
        });
        syn::visit::visit_item_union(self, i);
    }
}

fn extract_type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(tp) => tp.path.segments.last().map(|s| s.ident.to_string()),
        syn::Type::Paren(p) => extract_type_name(&p.elem),
        syn::Type::Reference(r) => extract_type_name(&r.elem),
        syn::Type::Group(g) => extract_type_name(&g.elem),
        syn::Type::TraitObject(to) => {
            for bound in &to.bounds {
                if let syn::TypeParamBound::Trait(tb) = bound {
                    if let Some(seg) = tb.path.segments.last() {
                        return Some(seg.ident.to_string());
                    }
                }
            }
            None
        }
        _ => None,
    }
}

#[derive(Debug)]
struct WorkspaceCrateInfo {
    name: String,
    ring: Ring,
    crate_dir: PathBuf,
}

fn find_workspace_crates(
    root_dir: &Path,
    ring_map: &HashMap<String, Ring>,
) -> Vec<WorkspaceCrateInfo> {
    let mut crates = Vec::new();

    let capabilities_path = root_dir.join("capabilities.toml");
    let mut caps_crates: HashMap<String, PathBuf> = HashMap::new();
    if capabilities_path.exists() {
        if let Ok(content) = fs::read_to_string(&capabilities_path) {
            if let Ok(val) = content.parse::<toml::Value>() {
                if let Some(crates_tbl) = val.get("crates").and_then(|c| c.as_table()) {
                    for (c_name, c_val) in crates_tbl {
                        if let Some(path_str) = c_val.get("path").and_then(|p| p.as_str()) {
                            caps_crates.insert(c_name.clone(), root_dir.join(path_str));
                        }
                    }
                }
            }
        }
    }

    for (c_name, ring) in ring_map {
        let crate_dir = if let Some(p) = caps_crates.get(c_name.as_str()) {
            p.clone()
        } else {
            let p1 = root_dir.join("crates").join(c_name);
            let p2 = root_dir.join("crates").join(c_name.replace('_', "-"));
            let p3 = root_dir.join(c_name);
            if p1.exists() {
                p1
            } else if p2.exists() {
                p2
            } else if p3.exists() {
                p3
            } else {
                continue;
            }
        };

        crates.push(WorkspaceCrateInfo {
            name: c_name.clone(),
            ring: *ring,
            crate_dir,
        });
    }

    // Also scan crates/ directory for any crates not in caps_crates/ring_map
    let crates_dir = root_dir.join("crates");
    if crates_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&crates_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && p.join("Cargo.toml").exists() {
                    let folder_name = p.file_name().unwrap_or_default().to_string_lossy();
                    if !crates.iter().any(|c| c.crate_dir == p) {
                        let ring = ring_map
                            .get(folder_name.as_ref())
                            .copied()
                            .unwrap_or(Ring::Ring0);
                        crates.push(WorkspaceCrateInfo {
                            name: folder_name.to_string(),
                            ring,
                            crate_dir: p,
                        });
                    }
                }
            }
        }
    }

    crates
}

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.exists() {
        return files;
    }
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path.to_path_buf());
        }
    }
    files
}

pub fn run_unwired_ports(args: &[String]) -> i32 {
    let mut root_arg: Option<String> = None;
    let mut json_output = false;
    let mut only_unwired = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                if i + 1 < args.len() {
                    root_arg = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Fehler: --root benötigt einen Pfad als Argument.");
                    return 1;
                }
            }
            "--json" => {
                json_output = true;
            }
            "--only-unwired" => {
                only_unwired = true;
            }
            arg if arg.starts_with('-') => {
                eprintln!("Unbekannte Option: {}", arg);
                return 1;
            }
            _ => {}
        }
        i += 1;
    }

    let root_dir = if let Some(ref r) = root_arg {
        PathBuf::from(r)
    } else {
        Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()
            .and_then(|out| {
                if out.status.success() {
                    String::from_utf8(out.stdout)
                        .ok()
                        .map(|s| PathBuf::from(s.trim()))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    };

    let ring_map = if let Some(ref r) = root_arg {
        let cargo_toml = PathBuf::from(r).join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(out) = Command::new("cargo")
                .args([
                    "metadata",
                    "--format-version",
                    "1",
                    "--no-deps",
                    "--manifest-path",
                ])
                .arg(&cargo_toml)
                .output()
            {
                if out.status.success() {
                    let json_str = String::from_utf8_lossy(&out.stdout);
                    get_ring_map_from_metadata_json(&json_str).ok()
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
    .unwrap_or_else(|| get_workspace_ring_map().unwrap_or_default());

    let workspace_crates = find_workspace_crates(&root_dir, &ring_map);

    let mut all_traits: Vec<TraitDecl> = Vec::new();
    let mut all_impls: Vec<ImplDecl> = Vec::new();
    let mut all_type_defs: Vec<TypeDecl> = Vec::new();

    // 1. & 2. Trait and Impl scanning over src/ directories of crates
    for crate_info in &workspace_crates {
        let src_dir = crate_info.crate_dir.join("src");
        let src_files = collect_rs_files(&src_dir);

        for file_path in src_files {
            let rel_file = file_path
                .strip_prefix(&root_dir)
                .unwrap_or(&file_path)
                .to_string_lossy()
                .to_string();

            if let Ok(content) = fs::read_to_string(&file_path) {
                if let Ok(ast) = syn::parse_file(&content) {
                    let mut visitor = CodeVisitor::new(&crate_info.name, &rel_file);
                    visitor.visit_file(&ast);
                    all_traits.extend(visitor.traits);
                    all_impls.extend(visitor.impls);
                    all_type_defs.extend(visitor.type_defs);
                }
            }
        }
    }

    // Identify Composition Root Crates (Ring 3 and Ring 4)
    let comp_root_crates: Vec<&WorkspaceCrateInfo> = workspace_crates
        .iter()
        .filter(|c| matches!(c.ring, Ring::Ring3 | Ring::Ring4))
        .collect();

    // Collect all composition root files (including tests/ and test/ and benches/)
    let mut comp_root_files: Vec<(String, String)> = Vec::new(); // (rel_file_path, content)
    for comp_crate in &comp_root_crates {
        let crate_files = collect_rs_files(&comp_crate.crate_dir);
        for file_path in crate_files {
            let rel_file = file_path
                .strip_prefix(&root_dir)
                .unwrap_or(&file_path)
                .to_string_lossy()
                .to_string();
            if let Ok(content) = fs::read_to_string(&file_path) {
                comp_root_files.push((rel_file, content));
            }
        }
    }

    // Group trait definitions by trait name
    let mut trait_map: HashMap<String, Vec<TraitDecl>> = HashMap::new();
    for t in all_traits {
        trait_map.entry(t.trait_name.clone()).or_default().push(t);
    }

    // Build map of type definitions to their declaration files
    let mut type_def_files_map: HashMap<String, HashSet<String>> = HashMap::new();
    for td in all_type_defs {
        type_def_files_map
            .entry(td.type_name)
            .or_default()
            .insert(td.file_path);
    }

    let mut findings: Vec<UnwiredPortsFinding> = Vec::new();

    let mut trait_names: Vec<String> = trait_map.keys().cloned().collect();
    trait_names.sort();

    for trait_name in trait_names {
        let trait_decls = &trait_map[&trait_name];
        let mut defined_in: Vec<String> = trait_decls.iter().map(|d| d.file_path.clone()).collect();
        defined_in.sort();
        defined_in.dedup();

        let matching_impls: Vec<&ImplDecl> = all_impls
            .iter()
            .filter(|imp| imp.trait_name == trait_name)
            .collect();

        if matching_impls.is_empty() {
            // 3. Trait without any impl => UNWIRED
            findings.push(UnwiredPortsFinding {
                severity: "UNWIRED".to_string(),
                trait_name: trait_name.clone(),
                implementing_types: Vec::new(),
                defined_in,
            });
        } else {
            // 4. Trait with at least one impl => check wiring in Composition Root crates
            let mut impl_types: Vec<String> =
                matching_impls.iter().map(|i| i.type_name.clone()).collect();
            impl_types.sort();
            impl_types.dedup();

            let mut excluded_files: HashSet<String> = HashSet::new();
            for def in &defined_in {
                excluded_files.insert(def.clone());
            }
            for imp in &matching_impls {
                excluded_files.insert(imp.file_path.clone());
            }
            for ty_name in &impl_types {
                if let Some(def_files) = type_def_files_map.get(ty_name) {
                    for f in def_files {
                        excluded_files.insert(f.clone());
                    }
                }
            }

            let mut is_wired = false;
            'check_wiring: for ty_name in &impl_types {
                let pattern = format!(r"\b{}\b", regex::escape(ty_name));
                if let Ok(re) = regex::Regex::new(&pattern) {
                    for (rel_file, content) in &comp_root_files {
                        if excluded_files.contains(rel_file) {
                            continue;
                        }
                        if re.is_match(content) {
                            is_wired = true;
                            break 'check_wiring;
                        }
                    }
                }
            }

            let severity = if is_wired { "OK" } else { "SOLO" };
            findings.push(UnwiredPortsFinding {
                severity: severity.to_string(),
                trait_name: trait_name.clone(),
                implementing_types: impl_types,
                defined_in,
            });
        }
    }

    let unwired_count = findings.iter().filter(|f| f.severity == "UNWIRED").count();
    let solo_count = findings.iter().filter(|f| f.severity == "SOLO").count();
    let ok_count = findings.iter().filter(|f| f.severity == "OK").count();

    let status = if unwired_count > 0 { "fail" } else { "pass" };
    let summary = format!(
        "Unwired ports audit: {} UNWIRED, {} SOLO, {} OK traits found.",
        unwired_count, solo_count, ok_count
    );

    if only_unwired {
        findings.retain(|f| f.severity == "UNWIRED");
    }

    let gate_result = UnwiredPortsGateResult {
        gate: "unwired-ports".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        findings: findings.clone(),
    };

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&gate_result).unwrap_or_default()
        );
    } else {
        println!("=== Gate unwired-ports: {} ===", status);
        println!("{}", summary);
        println!("\n> **Hinweis:** Die Typ-Suche basiert auf Wortgrenzen-Regex (`\\b<Typ>\\b`). Bei sehr kurzen oder generischen Typnamen kann diese Heuristik False Positives bezüglich der Verdrahtung erzeugen.\n");

        let unwired_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == "UNWIRED")
            .collect();
        println!("## UNWIRED (Traits ohne Implementierung)");
        if unwired_findings.is_empty() {
            println!("- (keine)");
        } else {
            for f in unwired_findings {
                println!(
                    "- `{}` (definiert in: {})",
                    f.trait_name,
                    f.defined_in.join(", ")
                );
            }
        }

        if !only_unwired {
            let solo_findings: Vec<_> = findings.iter().filter(|f| f.severity == "SOLO").collect();
            println!("\n## SOLO (Implementiert, aber in keinem Composition Root verdrahtet)");
            if solo_findings.is_empty() {
                println!("- (keine)");
            } else {
                for f in solo_findings {
                    println!(
                        "- `{}` -> Typen: `{}` (definiert in: {})",
                        f.trait_name,
                        f.implementing_types.join(", "),
                        f.defined_in.join(", ")
                    );
                }
            }

            let ok_findings: Vec<_> = findings.iter().filter(|f| f.severity == "OK").collect();
            println!("\n## OK (Ordnungsgemäß verdrahtet)");
            if ok_findings.is_empty() {
                println!("- (keine)");
            } else {
                for f in ok_findings {
                    println!(
                        "- `{}` -> Typen: `{}` (definiert in: {})",
                        f.trait_name,
                        f.implementing_types.join(", "),
                        f.defined_in.join(", ")
                    );
                }
            }
        }
    }

    if status == "fail" {
        2
    } else {
        0
    }
}
