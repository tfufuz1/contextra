//! Harness Modul: findet öffentliche Symbole ohne jede Referenz außerhalb ihrer eigenen Deklaration (orphan-symbols).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Item, Visibility};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrphanSymbolFinding {
    pub severity: String,
    pub krate: String,
    pub kind: String,
    pub name: String,
    pub file: String,
    pub line: usize,
    pub is_boundary_crate: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrphanSymbolsResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<OrphanSymbolFinding>,
}

#[derive(Debug)]
struct DeclItem {
    name: String,
    kind: String,
    krate: String,
    file: String,
    line: usize,
}

#[derive(Default)]
struct IdentCounter {
    counts: HashMap<String, usize>,
}

impl<'ast> Visit<'ast> for IdentCounter {
    fn visit_ident(&mut self, i: &'ast syn::Ident) {
        let name = i.to_string();
        *self.counts.entry(name).or_insert(0) += 1;
        syn::visit::visit_ident(self, i);
    }

    fn visit_macro(&mut self, i: &'ast syn::Macro) {
        self.count_tokens(&i.tokens);
        syn::visit::visit_macro(self, i);
    }
}

impl IdentCounter {
    fn count_tokens(&mut self, tokens: &proc_macro2::TokenStream) {
        for tt in tokens.clone() {
            match tt {
                proc_macro2::TokenTree::Group(g) => self.count_tokens(&g.stream()),
                proc_macro2::TokenTree::Ident(id) => {
                    *self.counts.entry(id.to_string()).or_insert(0) += 1;
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct CapabilitiesToml {
    #[serde(default)]
    crates: HashMap<String, CrateEntryToml>,
}

#[derive(Debug, serde::Deserialize)]
struct CrateEntryToml {
    ring: Option<String>,
}

fn load_boundary_crates(root: &Path) -> HashSet<String> {
    let mut boundary = HashSet::new();
    let cap_path = root.join("capabilities.toml");
    if let Ok(content) = fs::read_to_string(&cap_path) {
        if let Ok(parsed) = toml::from_str::<CapabilitiesToml>(&content) {
            for (c_name, info) in parsed.crates {
                if let Some(r) = info.ring {
                    if r.contains('4') {
                        boundary.insert(c_name.clone());
                        boundary.insert(c_name.replace('-', "_"));
                    }
                }
            }
        }
    }
    boundary
}

fn orphan_symbols_default_root() -> PathBuf {
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return PathBuf::from(path_str);
            }
        }
    }
    PathBuf::from(".")
}

fn is_kind_allowed(kind: &str, filter: Option<&HashSet<String>>) -> bool {
    match filter {
        Some(set) => set.contains(kind),
        None => true,
    }
}

fn collect_pub_decls(
    items: &[Item],
    krate: &str,
    rel_file: &str,
    kinds_filter: Option<&HashSet<String>>,
    decls: &mut Vec<DeclItem>,
) {
    for item in items {
        match item {
            Item::Fn(f) => {
                if matches!(f.vis, Visibility::Public(_)) && is_kind_allowed("fn", kinds_filter) {
                    decls.push(DeclItem {
                        name: f.sig.ident.to_string(),
                        kind: "fn".to_string(),
                        krate: krate.to_string(),
                        file: rel_file.to_string(),
                        line: f.span().start().line,
                    });
                }
            }
            Item::Struct(s) => {
                if matches!(s.vis, Visibility::Public(_)) && is_kind_allowed("struct", kinds_filter) {
                    decls.push(DeclItem {
                        name: s.ident.to_string(),
                        kind: "struct".to_string(),
                        krate: krate.to_string(),
                        file: rel_file.to_string(),
                        line: s.span().start().line,
                    });
                }
            }
            Item::Enum(e) => {
                if matches!(e.vis, Visibility::Public(_)) && is_kind_allowed("enum", kinds_filter) {
                    decls.push(DeclItem {
                        name: e.ident.to_string(),
                        kind: "enum".to_string(),
                        krate: krate.to_string(),
                        file: rel_file.to_string(),
                        line: e.span().start().line,
                    });
                }
            }
            Item::Trait(t) => {
                if matches!(t.vis, Visibility::Public(_)) && is_kind_allowed("trait", kinds_filter) {
                    decls.push(DeclItem {
                        name: t.ident.to_string(),
                        kind: "trait".to_string(),
                        krate: krate.to_string(),
                        file: rel_file.to_string(),
                        line: t.span().start().line,
                    });
                }
            }
            Item::Type(ty) => {
                if matches!(ty.vis, Visibility::Public(_)) && is_kind_allowed("type", kinds_filter) {
                    decls.push(DeclItem {
                        name: ty.ident.to_string(),
                        kind: "type".to_string(),
                        krate: krate.to_string(),
                        file: rel_file.to_string(),
                        line: ty.span().start().line,
                    });
                }
            }
            Item::Const(c) => {
                if matches!(c.vis, Visibility::Public(_)) && is_kind_allowed("const", kinds_filter) {
                    decls.push(DeclItem {
                        name: c.ident.to_string(),
                        kind: "const".to_string(),
                        krate: krate.to_string(),
                        file: rel_file.to_string(),
                        line: c.span().start().line,
                    });
                }
            }
            Item::Mod(m) => {
                if let Some((_, ref sub_items)) = m.content {
                    collect_pub_decls(sub_items, krate, rel_file, kinds_filter, decls);
                }
            }
            _ => {}
        }
    }
}

pub fn run_orphan_symbols(args: &[String]) -> i32 {
    let mut root_dir = orphan_symbols_default_root();
    let mut use_json = false;
    let mut kinds_filter: Option<HashSet<String>> = None;
    let mut exclude_crates: HashSet<String> = HashSet::new();

    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                if idx + 1 < args.len() {
                    root_dir = PathBuf::from(&args[idx + 1]);
                    idx += 1;
                }
            }
            "--json" => {
                use_json = true;
            }
            "--kind" => {
                if idx + 1 < args.len() {
                    let kinds: HashSet<String> = args[idx + 1]
                        .split(',')
                        .map(|s| s.trim().to_lowercase())
                        .filter(|s| !s.is_empty())
                        .collect();
                    kinds_filter = Some(kinds);
                    idx += 1;
                }
            }
            "--exclude-crate" => {
                if idx + 1 < args.len() {
                    for c in args[idx + 1].split(',') {
                        let trimmed = c.trim();
                        if !trimmed.is_empty() {
                            exclude_crates.insert(trimmed.to_string());
                            exclude_crates.insert(trimmed.replace('-', "_"));
                        }
                    }
                    idx += 1;
                }
            }
            _ => {}
        }
        idx += 1;
    }

    if !root_dir.exists() {
        eprintln!(
            "Fehler: Wurzelverzeichnis '{}' existiert nicht.",
            root_dir.display()
        );
        return 1;
    }

    let crates_dir = root_dir.join("crates");
    if !crates_dir.exists() {
        eprintln!(
            "Fehler: Crates-Verzeichnis '{}' existiert nicht.",
            crates_dir.display()
        );
        return 1;
    }

    let boundary_crates = load_boundary_crates(&root_dir);

    let mut counter = IdentCounter::default();
    let mut decls = Vec::new();

    let walker = walkdir::WalkDir::new(&crates_dir);
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let rel = match path.strip_prefix(&root_dir) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let parts: Vec<&str> = rel_str.split('/').collect();

        if parts.len() < 4 || parts[0] != "crates" || parts[2] != "src" {
            continue;
        }

        let krate = parts[1].to_string();

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(err) => {
                eprintln!(
                    "Warnung: Fehler beim Lesen von '{}': {}",
                    path.display(),
                    err
                );
                continue;
            }
        };

        let ast = match syn::parse_file(&content) {
            Ok(a) => a,
            Err(err) => {
                eprintln!(
                    "Warnung: Parsing fehlgeschlagen für '{}': {}",
                    path.display(),
                    err
                );
                continue;
            }
        };

        // Pass 1: Global Ident count
        counter.visit_file(&ast);

        // Pass 2: Collect top-level pub decls if crate is not excluded
        if !exclude_crates.contains(&krate) && !exclude_crates.contains(&krate.replace('-', "_")) {
            collect_pub_decls(&ast.items, &krate, &rel_str, kinds_filter.as_ref(), &mut decls);
        }
    }

    let mut findings = Vec::new();
    let mut hoch_count = 0;
    let mut mittel_count = 0;

    for decl in decls {
        let global_count = counter.counts.get(&decl.name).copied().unwrap_or(0);
        let severity = match global_count {
            1 => "hoch",
            2 => "mittel",
            _ => continue,
        };

        if severity == "hoch" {
            hoch_count += 1;
        } else {
            mittel_count += 1;
        }

        let is_boundary_crate = boundary_crates.contains(&decl.krate)
            || boundary_crates.contains(&decl.krate.replace('-', "_"));

        findings.push(OrphanSymbolFinding {
            severity: severity.to_string(),
            krate: decl.krate,
            kind: decl.kind,
            name: decl.name,
            file: decl.file,
            line: decl.line,
            is_boundary_crate,
        });
    }

    // Sort findings deterministically: (severity: "hoch" < "mittel", krate, file, line, name)
    findings.sort_by(|a, b| {
        a.severity
            .cmp(&b.severity)
            .then_with(|| a.krate.cmp(&b.krate))
            .then_with(|| a.file.cmp(&b.file))
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.name.cmp(&b.name))
    });

    let has_fail = findings
        .iter()
        .any(|f| f.severity == "hoch" && !f.is_boundary_crate);
    let status = if has_fail { "fail" } else { "pass" };
    let summary = format!(
        "Gefundene Orphan-Symbole: {} (hoch: {}, mittel: {})",
        findings.len(),
        hoch_count,
        mittel_count
    );

    let result = OrphanSymbolsResult {
        gate: "orphan-symbols".to_string(),
        status: status.to_string(),
        summary,
        findings,
    };

    if use_json {
        println!("{}", serde_json::to_string(&result).unwrap_or_default());
    } else {
        println!("=== Gate orphan-symbols: {} ===", result.status);
        println!("{}", result.summary);
        println!();

        let hoch_findings: Vec<_> = result.findings.iter().filter(|f| f.severity == "hoch").collect();
        let mittel_findings: Vec<_> = result.findings.iter().filter(|f| f.severity == "mittel").collect();

        if !hoch_findings.is_empty() {
            println!("### Severity: hoch");
            println!("| Crate | Kind | Name | Datei | Zeile | Grenz-Crate |");
            println!("| :--- | :--- | :--- | :--- | :--- | :--- |");
            for f in &hoch_findings {
                let boundary_str = if f.is_boundary_crate { "ja" } else { "nein" };
                println!(
                    "| `{}` | `{}` | `{}` | `{}` | {} | {} |",
                    f.krate, f.kind, f.name, f.file, f.line, boundary_str
                );
            }
            println!();
        }

        if !mittel_findings.is_empty() {
            println!("### Severity: mittel");
            println!("| Crate | Kind | Name | Datei | Zeile | Grenz-Crate |");
            println!("| :--- | :--- | :--- | :--- | :--- | :--- |");
            for f in &mittel_findings {
                let boundary_str = if f.is_boundary_crate { "ja" } else { "nein" };
                println!(
                    "| `{}` | `{}` | `{}` | `{}` | {} | {} |",
                    f.krate, f.kind, f.name, f.file, f.line, boundary_str
                );
            }
            println!();
        }

        if result.findings.is_empty() {
            println!("Keine Orphan-Symbole gefunden.");
        }
    }

    if status == "pass" {
        0
    } else {
        2
    }
}
