//! Harness Modul zur Erzeugung des Mapping-Reports für die Entfernung von `contextra-core` (generate-core-mapping).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use syn::{ItemUse, UseTree, ItemStruct, ItemEnum, ItemTrait, ItemType, ItemConst, ItemFn, Visibility};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MappingEntry {
    pub symbol: String,
    pub original_crate: String,
    pub original_module: String,
    pub core_reexport_path: String,
    pub suggested_target_crate: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoreMappingReport {
    pub generated_at_sha: String,
    pub total_symbols: usize,
    pub open_count: usize,
    pub entries: Vec<MappingEntry>,
}

struct CoreUseVisitor {
    use_paths: Vec<String>,
}

impl CoreUseVisitor {
    fn new() -> Self {
        Self {
            use_paths: Vec::new(),
        }
    }

    fn collect_tree(prefix: &str, tree: &UseTree, out: &mut Vec<String>) {
        match tree {
            UseTree::Path(p) => {
                let new_prefix = if prefix.is_empty() {
                    p.ident.to_string()
                } else {
                    format!("{}::{}", prefix, p.ident)
                };
                Self::collect_tree(&new_prefix, &p.tree, out);
            }
            UseTree::Name(n) => {
                let path = if prefix.is_empty() {
                    n.ident.to_string()
                } else {
                    format!("{}::{}", prefix, n.ident)
                };
                out.push(path);
            }
            UseTree::Rename(r) => {
                let path = if prefix.is_empty() {
                    format!("{} as {}", r.ident, r.rename)
                } else {
                    format!("{}::{} as {}", prefix, r.ident, r.rename)
                };
                out.push(path);
            }
            UseTree::Glob(_) => {
                let path = format!("{}::*", prefix);
                out.push(path);
            }
            UseTree::Group(g) => {
                for item in &g.items {
                    Self::collect_tree(prefix, item, out);
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for CoreUseVisitor {
    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        let mut paths = Vec::new();
        Self::collect_tree("", &node.tree, &mut paths);
        self.use_paths.extend(paths);
        syn::visit::visit_item_use(self, node);
    }
}

/// Visitor to collect all public symbols in a crate directory.
struct CratePubSymbolsVisitor {
    pub_symbols: BTreeMap<String, String>, // symbol -> module_path
    current_mod: String,
}

impl<'ast> Visit<'ast> for CratePubSymbolsVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if matches!(node.vis, Visibility::Public(_)) {
            self.pub_symbols.insert(node.ident.to_string(), self.current_mod.clone());
        }
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        if matches!(node.vis, Visibility::Public(_)) {
            self.pub_symbols.insert(node.ident.to_string(), self.current_mod.clone());
        }
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        if matches!(node.vis, Visibility::Public(_)) {
            self.pub_symbols.insert(node.ident.to_string(), self.current_mod.clone());
        }
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast ItemType) {
        if matches!(node.vis, Visibility::Public(_)) {
            self.pub_symbols.insert(node.ident.to_string(), self.current_mod.clone());
        }
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_const(&mut self, node: &'ast ItemConst) {
        if matches!(node.vis, Visibility::Public(_)) {
            self.pub_symbols.insert(node.ident.to_string(), self.current_mod.clone());
        }
        syn::visit::visit_item_const(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if matches!(node.vis, Visibility::Public(_)) {
            self.pub_symbols.insert(node.sig.ident.to_string(), self.current_mod.clone());
        }
        syn::visit::visit_item_fn(self, node);
    }
}

fn scan_crate_pub_symbols(crate_dir: &Path, crate_name: &str) -> BTreeMap<String, String> {
    let mut symbols = BTreeMap::new();
    let src_dir = crate_dir.join("src");
    if !src_dir.exists() {
        return symbols;
    }

    let walker = walkdir::WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok());
    for entry in walker {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "rs") {
            let relative = entry.path().strip_prefix(&src_dir).unwrap_or(entry.path());
            let mod_parts: Vec<String> = relative
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .map(|s| s.trim_end_matches(".rs").to_string())
                .filter(|s| s != "mod" && s != "lib")
                .collect();

            let mod_path = if mod_parts.is_empty() {
                crate_name.to_string()
            } else {
                format!("{}::{}", crate_name, mod_parts.join("::"))
            };

            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(syntax) = syn::parse_file(&content) {
                    let mut visitor = CratePubSymbolsVisitor {
                        pub_symbols: BTreeMap::new(),
                        current_mod: mod_path,
                    };
                    visitor.visit_file(&syntax);
                    for (sym, origin) in visitor.pub_symbols {
                        symbols.insert(sym, origin);
                    }
                }
            }
        }
    }

    symbols
}

pub fn run_generate_core_mapping(args: &[String]) -> i32 {
    let repo_root = Path::new(".");
    let core_lib_path = repo_root.join("crates/contextra-core/src/lib.rs");

    let core_lib_content = match fs::read_to_string(&core_lib_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[error] generate-core-mapping: Konnte {:?} nicht lesen: {}", core_lib_path, e);
            return 2;
        }
    };

    let syn_file = match syn::parse_file(&core_lib_content) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[error] generate-core-mapping: Konnte AST von {:?} nicht parsen: {}", core_lib_path, e);
            return 2;
        }
    };

    let mut visitor = CoreUseVisitor::new();
    visitor.visit_file(&syn_file);

    // Dynamically scan public symbols of origin crates
    let types_symbols = scan_crate_pub_symbols(&repo_root.join("crates/contextra-types"), "contextra_types");
    let ports_symbols = scan_crate_pub_symbols(&repo_root.join("crates/contextra-ports"), "contextra_ports");
    let wire_symbols = scan_crate_pub_symbols(&repo_root.join("crates/contextra-wire"), "contextra_wire");
    let mvcc_symbols = scan_crate_pub_symbols(&repo_root.join("crates/contextra-mvcc"), "contextra_mvcc");

    // Resolve re-exports into discrete symbols and mapping entries
    let mut symbol_entries: BTreeMap<String, MappingEntry> = BTreeMap::new();

    for path in visitor.use_paths {
        if path.ends_with("::*") {
            let base_mod = path.trim_end_matches("::*");
            if base_mod == "contextra_ports" {
                // Wildcard re-export of contextra_ports!
                // Include all symbols defined in ports or inherited from types
                for (sym, origin_mod) in &ports_symbols {
                    let reexport_path = format!("contextra_core::traits::{}", sym);
                    let (target_crate, status) = classify_target_crate(sym, "contextra_ports", origin_mod, &types_symbols);
                    let entry = MappingEntry {
                        symbol: sym.clone(),
                        original_crate: if types_symbols.contains_key(sym) { "contextra_types".to_string() } else { "contextra_ports".to_string() },
                        original_module: if let Some(t_origin) = types_symbols.get(sym) { t_origin.clone() } else { origin_mod.clone() },
                        core_reexport_path: reexport_path,
                        suggested_target_crate: target_crate,
                        status,
                    };
                    symbol_entries.insert(sym.clone(), entry);
                }
                // Also add types inherited by contextra_ports re-exporting contextra_types
                for (sym, origin_mod) in &types_symbols {
                    if !symbol_entries.contains_key(sym) {
                        let (target_crate, status) = classify_target_crate(sym, "contextra_types", origin_mod, &types_symbols);
                        let entry = MappingEntry {
                            symbol: sym.clone(),
                            original_crate: "contextra_types".to_string(),
                            original_module: origin_mod.clone(),
                            core_reexport_path: format!("contextra_core::traits::{}", sym),
                            suggested_target_crate: target_crate,
                            status,
                        };
                        symbol_entries.insert(sym.clone(), entry);
                    }
                }
            } else if base_mod == "contextra_wire" {
                for (sym, origin_mod) in &wire_symbols {
                    let (target_crate, status) = classify_target_crate(sym, "contextra_wire", origin_mod, &types_symbols);
                    let entry = MappingEntry {
                        symbol: sym.clone(),
                        original_crate: "contextra_wire".to_string(),
                        original_module: origin_mod.clone(),
                        core_reexport_path: format!("contextra_core::ipc::{}", sym),
                        suggested_target_crate: target_crate,
                        status,
                    };
                    symbol_entries.insert(sym.clone(), entry);
                }
            } else {
                let sym = format!("{}::*", base_mod);
                let entry = MappingEntry {
                    symbol: sym.clone(),
                    original_crate: base_mod.split("::").next().unwrap_or("").to_string(),
                    original_module: base_mod.to_string(),
                    core_reexport_path: format!("contextra_core::{}", path),
                    suggested_target_crate: "OFFEN".to_string(),
                    status: "OFFEN".to_string(),
                };
                symbol_entries.insert(sym, entry);
            }
        } else {
            // Direct re-export, e.g. contextra_types::error::{ContextraError, Result} or contextra_mvcc::tx_buffer::{IndexOp, TxBuffer}
            let parts: Vec<&str> = path.split("::").collect();
            if parts.len() >= 2 {
                let sym_name = parts.last().cloned().unwrap_or_default();
                let origin_crate = parts[0];
                let origin_mod = parts[..parts.len() - 1].join("::");

                let (target_crate, status) = classify_target_crate(sym_name, origin_crate, &origin_mod, &types_symbols);
                let entry = MappingEntry {
                    symbol: sym_name.to_string(),
                    original_crate: origin_crate.to_string(),
                    original_module: origin_mod,
                    core_reexport_path: format!("contextra_core::{}", path),
                    suggested_target_crate: target_crate,
                    status,
                };
                symbol_entries.insert(sym_name.to_string(), entry);
            }
        }
    }

    let entries: Vec<MappingEntry> = symbol_entries.into_values().collect();
    let open_count = entries.iter().filter(|e| e.status == "OFFEN").count();

    let git_head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "HEAD".to_string());

    let report = CoreMappingReport {
        generated_at_sha: git_head,
        total_symbols: entries.len(),
        open_count,
        entries: entries.clone(),
    };

    let json_out = match serde_json::to_string_pretty(&report) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("[error] generate-core-mapping: JSON-Serialisierung fehlgeschlagen: {}", e);
            return 2;
        }
    };

    let mut md_out = String::new();
    md_out.push_str("# Contextra Core Re-Export Mapping Report\n\n");
    md_out.push_str(&format!("**Commit SHA:** `{}`\n", report.generated_at_sha));
    md_out.push_str(&format!("**Gesamtanzahl erfasster Symbole:** {}\n", report.total_symbols));
    md_out.push_str(&format!("**Anzahl offener Symbole (`OFFEN`):** {}\n\n", report.open_count));

    if report.open_count > 0 {
        md_out.push_str("## Offene Rückfragen (`OFFEN`)\n\n");
        md_out.push_str("| Symbol | Ursprungscrate | Ursprungsmodul | Core Re-Export Pfad | Vorschlag Zielcrate |\n");
        md_out.push_str("|---|---|---|---|---|\n");
        for entry in entries.iter().filter(|e| e.status == "OFFEN") {
            md_out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | `{}` |\n",
                entry.symbol, entry.original_crate, entry.original_module, entry.core_reexport_path, entry.suggested_target_crate
            ));
        }
        md_out.push_str("\n");
    }

    md_out.push_str("## Vollständige Symbol-Zuordnung\n\n");
    md_out.push_str("| Symbol | Ursprungscrate | Ursprungsmodul | Core Re-Export Pfad | Empfohlenes Zielcrate | Status |\n");
    md_out.push_str("|---|---|---|---|---|---|\n");
    for entry in &entries {
        md_out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | {} |\n",
            entry.symbol, entry.original_crate, entry.original_module, entry.core_reexport_path, entry.suggested_target_crate, entry.status
        ));
    }

    let json_path = repo_root.join("docs/refactor/core-mapping.json");
    let md_path = repo_root.join("docs/refactor/core-mapping.md");

    if let Err(e) = fs::write(&json_path, &json_out) {
        eprintln!("[error] generate-core-mapping: Konnte {:?} nicht schreiben: {}", json_path, e);
        return 2;
    }

    if let Err(e) = fs::write(&md_path, &md_out) {
        eprintln!("[error] generate-core-mapping: Konnte {:?} nicht schreiben: {}", md_path, e);
        return 2;
    }

    // Write audit log
    let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let audit_dir = repo_root.join(format!("logs/audits/J-14_{}", date_str));
    let _ = fs::create_dir_all(&audit_dir);
    let audit_log = audit_dir.join("generate-core-mapping.log");
    let log_content = format!(
        "generate-core-mapping report generated.\nTotal symbols: {}\nOFFEN count: {}\nCommit: {}\nFiles written:\n - {}\n - {}\n",
        report.total_symbols, report.open_count, report.generated_at_sha, json_path.display(), md_path.display()
    );
    let _ = fs::write(&audit_log, log_content);

    println!(
        "[ok] generate-core-mapping: {} Symbole erfasst ({} OFFEN). Berichte unter docs/refactor/ geschrieben.",
        report.total_symbols, report.open_count
    );

    0
}

fn classify_target_crate(symbol: &str, origin_crate: &str, _origin_module: &str, types_symbols: &BTreeMap<String, String>) -> (String, String) {
    // Spec §4.9 Punkt 2 mapping rules:
    // - DocId, TxId, ContextraError -> contextra_types
    // - TxBuffer -> contextra_mvcc
    // - WireFormat -> contextra_wire
    // - Budget types & Distance types -> contextra_ports
    if symbol == "ResourceBudget" || symbol == "ResourceTracker" || symbol == "TokenBudget" || symbol == "DistanceMetric" || symbol == "MAX_SEARCH_K" || symbol == "TOMBSTONE_BIT" {
        return ("contextra_ports".to_string(), "KLAR".to_string());
    }

    if symbol == "DocId" || symbol == "TxId" || symbol == "ContextraError" || symbol == "ContextraErrorDto" || symbol == "SeqBitTombstone" {
        return ("contextra_types".to_string(), "KLAR".to_string());
    }

    if symbol == "TxBuffer" || symbol == "IndexOp" || symbol == "SequenceLog" || symbol == "SnapshotRegistry" || symbol == "SnapshotGuard" || symbol == "SnapshotLease" || origin_crate == "contextra_mvcc" {
        return ("contextra_mvcc".to_string(), "KLAR".to_string());
    }

    if symbol == "WireFormat" || symbol == "WireWiring" || origin_crate == "contextra_wire" {
        return ("contextra_wire".to_string(), "KLAR".to_string());
    }

    if origin_crate == "contextra_types" || types_symbols.contains_key(symbol) {
        return ("contextra_types".to_string(), "KLAR".to_string());
    }

    if origin_crate == "contextra_ports" {
        return ("contextra_ports".to_string(), "KLAR".to_string());
    }

    ("OFFEN".to_string(), "OFFEN".to_string())
}
