//! Harness-Modul: findet öffentliche Symbole, die workspace-weit nicht verwendet werden (orphan-symbols).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use syn::{Item, Visibility};

use crate::harness::scan_common::{classify_file, resolve_external_test_modules, FileClass};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SymbolFinding {
    pub kind: String,
    pub symbol: String,
    pub krate: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummaryData {
    pub orphan_count: usize,
    pub nur_tests_count: usize,
    pub produktiv_count: usize,
    pub verdacht_count: usize,
    pub excluded_boundary_count: usize,
    pub excluded_crates: Vec<String>,
    pub kinds_filter: Vec<String>,
    pub disclaimer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrphanSymbolsResult {
    pub gate: String,
    pub status: String,
    pub summary: SummaryData,
    pub orphan: Vec<SymbolFinding>,
    pub nur_tests: Vec<SymbolFinding>,
    pub verdacht: Vec<SymbolFinding>,
    pub note: String,
}

#[derive(Debug, Clone)]
struct DeclItem {
    name: String,
    kind: String,
    krate: String,
    file: String,
    line: usize,
}

struct DeclVisitor<'a> {
    krate: &'a str,
    rel_file: &'a str,
    kinds_filter: &'a HashSet<String>,
    in_macro_rules: bool,
    decls: Vec<DeclItem>,
}

impl<'a> DeclVisitor<'a> {
    fn new(krate: &'a str, rel_file: &'a str, kinds_filter: &'a HashSet<String>) -> Self {
        Self {
            krate,
            rel_file,
            kinds_filter,
            in_macro_rules: false,
            decls: Vec::new(),
        }
    }

    fn is_vis_public_or_restricted(vis: &Visibility) -> bool {
        matches!(vis, Visibility::Public(_) | Visibility::Restricted(_))
    }

    fn collect_items(&mut self, items: &[Item]) {
        for item in items {
            if self.in_macro_rules {
                continue;
            }
            match item {
                Item::Fn(f) => {
                    if Self::is_vis_public_or_restricted(&f.vis) && self.kinds_filter.contains("fn")
                    {
                        self.decls.push(DeclItem {
                            name: f.sig.ident.to_string(),
                            kind: "fn".to_string(),
                            krate: self.krate.to_string(),
                            file: self.rel_file.to_string(),
                            line: f.sig.ident.span().start().line,
                        });
                    }
                }
                Item::Struct(s) => {
                    if Self::is_vis_public_or_restricted(&s.vis)
                        && self.kinds_filter.contains("struct")
                    {
                        self.decls.push(DeclItem {
                            name: s.ident.to_string(),
                            kind: "struct".to_string(),
                            krate: self.krate.to_string(),
                            file: self.rel_file.to_string(),
                            line: s.ident.span().start().line,
                        });
                    }
                }
                Item::Enum(e) => {
                    if Self::is_vis_public_or_restricted(&e.vis)
                        && self.kinds_filter.contains("enum")
                    {
                        self.decls.push(DeclItem {
                            name: e.ident.to_string(),
                            kind: "enum".to_string(),
                            krate: self.krate.to_string(),
                            file: self.rel_file.to_string(),
                            line: e.ident.span().start().line,
                        });
                    }
                }
                Item::Trait(t) => {
                    if Self::is_vis_public_or_restricted(&t.vis)
                        && self.kinds_filter.contains("trait")
                    {
                        self.decls.push(DeclItem {
                            name: t.ident.to_string(),
                            kind: "trait".to_string(),
                            krate: self.krate.to_string(),
                            file: self.rel_file.to_string(),
                            line: t.ident.span().start().line,
                        });
                    }
                }
                Item::Type(ty) => {
                    if Self::is_vis_public_or_restricted(&ty.vis)
                        && self.kinds_filter.contains("type")
                    {
                        self.decls.push(DeclItem {
                            name: ty.ident.to_string(),
                            kind: "type".to_string(),
                            krate: self.krate.to_string(),
                            file: self.rel_file.to_string(),
                            line: ty.ident.span().start().line,
                        });
                    }
                }
                Item::Const(c) => {
                    if Self::is_vis_public_or_restricted(&c.vis)
                        && self.kinds_filter.contains("const")
                    {
                        self.decls.push(DeclItem {
                            name: c.ident.to_string(),
                            kind: "const".to_string(),
                            krate: self.krate.to_string(),
                            file: self.rel_file.to_string(),
                            line: c.ident.span().start().line,
                        });
                    }
                }
                Item::Mod(m) => {
                    if let Some((_, ref sub_items)) = m.content {
                        self.collect_items(sub_items);
                    }
                }
                Item::Macro(m) => {
                    if m.mac.path.is_ident("macro_rules") {
                        let prev = self.in_macro_rules;
                        self.in_macro_rules = true;
                        // Skip macro_rules items
                        self.in_macro_rules = prev;
                    }
                }
                _ => {}
            }
        }
    }
}

struct IdentVisitor {
    file_class: FileClass,
    prod_counts: HashMap<String, usize>,
    test_counts: HashMap<String, usize>,
}

impl IdentVisitor {
    fn new() -> Self {
        Self {
            file_class: FileClass::Production,
            prod_counts: HashMap::new(),
            test_counts: HashMap::new(),
        }
    }

    fn record_ident(&mut self, name: &str) {
        match self.file_class {
            FileClass::Production => {
                *self.prod_counts.entry(name.to_string()).or_insert(0) += 1;
            }
            FileClass::Test => {
                *self.test_counts.entry(name.to_string()).or_insert(0) += 1;
            }
            FileClass::Generated => {}
        }
    }

    fn count_tokens(&mut self, tokens: &proc_macro2::TokenStream) {
        for tt in tokens.clone() {
            match tt {
                proc_macro2::TokenTree::Group(g) => self.count_tokens(&g.stream()),
                proc_macro2::TokenTree::Ident(id) => {
                    self.record_ident(&id.to_string());
                }
                _ => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for IdentVisitor {
    fn visit_ident(&mut self, i: &'ast syn::Ident) {
        self.record_ident(&i.to_string());
        syn::visit::visit_ident(self, i);
    }

    fn visit_macro(&mut self, i: &'ast syn::Macro) {
        self.count_tokens(&i.tokens);
        syn::visit::visit_macro(self, i);
    }
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

fn scan_text_references(
    crates_dir: &Path,
    symbol_names: &HashSet<String>,
) -> HashMap<String, usize> {
    let mut text_counts = HashMap::new();
    if !crates_dir.exists() || symbol_names.is_empty() {
        return text_counts;
    }

    let walker = walkdir::WalkDir::new(crates_dir);
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if ext != "py" && ext != "pyi" && ext != "toml" {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for name in symbol_names {
            let pattern = format!(r"\b{}\b", regex::escape(name));
            if let Ok(re) = regex::Regex::new(&pattern) {
                let matches = re.find_iter(&content).count();
                if matches > 0 {
                    *text_counts.entry(name.clone()).or_insert(0) += matches;
                }
            }
        }
    }

    text_counts
}

pub fn run_orphan_symbols(args: &[String]) -> i32 {
    let mut root_dir = orphan_symbols_default_root();
    let mut use_json = false;
    let mut out_file_path: Option<PathBuf> = None;
    let mut kinds_filter: HashSet<String> = HashSet::from([
        "trait".to_string(),
        "struct".to_string(),
        "enum".to_string(),
        "fn".to_string(),
        "type".to_string(),
        "const".to_string(),
    ]);
    let mut exclude_crates: HashSet<String> = HashSet::from([
        "contextra".to_string(),
        "contextra-mcp".to_string(),
        "contextra-py".to_string(),
    ]);

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
            "--out-file" => {
                if idx + 1 < args.len() {
                    out_file_path = Some(PathBuf::from(&args[idx + 1]));
                    idx += 1;
                }
            }
            "--kind" => {
                if idx + 1 < args.len() {
                    kinds_filter = args[idx + 1]
                        .split(',')
                        .map(|s| s.trim().to_lowercase())
                        .filter(|s| !s.is_empty())
                        .collect();
                    idx += 1;
                }
            }
            "--exclude-crate" => {
                if idx + 1 < args.len() {
                    exclude_crates = HashSet::new();
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

    // Pass 1: Collect all .rs files under crates/
    let mut rs_files = Vec::new();
    let walker = walkdir::WalkDir::new(&crates_dir);
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            rs_files.push(path.to_path_buf());
        }
    }

    // Pass 2: Discover external test module references
    let mut external_test_files = HashSet::new();
    for file_path in &rs_files {
        if let Ok(content) = fs::read_to_string(file_path) {
            if let Ok(file_ast) = syn::parse_file(&content) {
                resolve_external_test_modules(file_path, &file_ast, &mut external_test_files);
            }
        }
    }

    // Pass 3: Classify files, collect declarations and ident occurrences
    let mut ident_visitor = IdentVisitor::new();
    let mut declarations = Vec::new();

    for file_path in &rs_files {
        let rel = match file_path.strip_prefix(&root_dir) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let parts: Vec<&str> = rel_str.split('/').collect();

        if parts.len() < 4 || parts[0] != "crates" || parts[2] != "src" {
            continue;
        }

        let krate = parts[1].to_string();

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(err) => {
                eprintln!(
                    "Warnung: Fehler beim Lesen von '{}': {}",
                    file_path.display(),
                    err
                );
                continue;
            }
        };

        let class = classify_file(file_path, &content, &external_test_files);

        let ast = match syn::parse_file(&content) {
            Ok(a) => a,
            Err(err) => {
                eprintln!(
                    "Warnung: Parsing fehlgeschlagen für '{}': {}",
                    file_path.display(),
                    err
                );
                continue;
            }
        };

        // Record ident references for usage counting
        ident_visitor.file_class = class;
        ident_visitor.visit_file(&ast);

        // Collect public declarations from production files
        if class == FileClass::Production {
            let mut decl_visitor = DeclVisitor::new(&krate, &rel_str, &kinds_filter);
            decl_visitor.collect_items(&ast.items);
            declarations.extend(decl_visitor.decls);
        }
    }

    // Pass 4: Collect text references for non-Rust files in crates/
    let unique_symbol_names: HashSet<String> =
        declarations.iter().map(|d| d.name.clone()).collect();
    let text_counts = scan_text_references(&crates_dir, &unique_symbol_names);

    // Pass 5: Evaluate declarations and classify
    let mut orphan_findings = Vec::new();
    let mut nur_tests_findings = Vec::new();
    let mut verdacht_findings = Vec::new();

    let mut produktiv_count = 0;
    let mut excluded_boundary_count = 0;
    let mut has_non_excluded_orphan = false;

    for decl in declarations {
        let prod_hits = ident_visitor
            .prod_counts
            .get(&decl.name)
            .copied()
            .unwrap_or(0);
        let test_hits = ident_visitor
            .test_counts
            .get(&decl.name)
            .copied()
            .unwrap_or(0);
        let text_hits = text_counts.get(&decl.name).copied().unwrap_or(0);

        let prod_usages = if prod_hits > 0 { prod_hits - 1 } else { 0 };
        let test_usages = test_hits;
        let text_usages = text_hits;
        let total_hits = prod_hits + test_hits + text_hits;

        let is_orphan = prod_usages == 0 && test_usages == 0 && text_usages == 0;
        let is_nur_tests = test_usages > 0 && prod_usages == 0 && text_usages == 0;
        let is_produktiv = prod_usages > 0 || text_usages > 0;
        let is_verdacht = total_hits == 2;

        let is_excluded_crate = exclude_crates.contains(&decl.krate)
            || exclude_crates.contains(&decl.krate.replace('-', "_"));

        let finding = SymbolFinding {
            kind: decl.kind,
            symbol: decl.name,
            krate: decl.krate,
            file: decl.file,
            line: decl.line,
        };

        if is_orphan {
            if is_excluded_crate {
                excluded_boundary_count += 1;
            } else {
                has_non_excluded_orphan = true;
                orphan_findings.push(finding.clone());
            }
        } else if is_nur_tests {
            nur_tests_findings.push(finding.clone());
        } else if is_produktiv {
            produktiv_count += 1;
        }

        if is_verdacht {
            verdacht_findings.push(finding);
        }
    }

    // Sort findings deterministically: (kind, symbol, krate, file, line)
    let sort_fn = |a: &SymbolFinding, b: &SymbolFinding| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| a.symbol.cmp(&b.symbol))
            .then_with(|| a.krate.cmp(&b.krate))
            .then_with(|| a.file.cmp(&b.file))
            .then_with(|| a.line.cmp(&b.line))
    };

    orphan_findings.sort_by(sort_fn);
    nur_tests_findings.sort_by(sort_fn);
    verdacht_findings.sort_by(sort_fn);

    let status = if has_non_excluded_orphan {
        "fail"
    } else {
        "pass"
    };

    let mut sorted_excluded_crates: Vec<String> = exclude_crates.into_iter().collect();
    sorted_excluded_crates.sort();

    let mut sorted_kinds_filter: Vec<String> = kinds_filter.into_iter().collect();
    sorted_kinds_filter.sort();

    let summary = SummaryData {
        orphan_count: orphan_findings.len(),
        nur_tests_count: nur_tests_findings.len(),
        produktiv_count,
        verdacht_count: verdacht_findings.len(),
        excluded_boundary_count,
        excluded_crates: sorted_excluded_crates,
        kinds_filter: sorted_kinds_filter,
        disclaimer: "Hinweis: Die Zahlen können gegen einen anderen Commit abweichen.".to_string(),
    };

    let note_text = "Ein Fund heißt nicht automatisch \"löschen\". Mögliche Ursachen: geplante Verdrahtung fehlt, bewusste öffentliche API, Refactoring hat den letzten Aufrufer entfernt.".to_string();

    let result = OrphanSymbolsResult {
        gate: "orphan-symbols".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        orphan: orphan_findings,
        nur_tests: nur_tests_findings,
        verdacht: verdacht_findings,
        note: note_text.clone(),
    };

    // Render Markdown Report
    let mut md = String::new();
    md.push_str("# Report: Orphan-Symbols Scanner\n\n");
    md.push_str("## Zusammenfassung\n\n");
    md.push_str(&format!(
        "- **Status:** `{}`\n",
        result.status.to_uppercase()
    ));
    md.push_str(&format!(
        "- **Exkludierte Grenz-Crates (`--exclude-crate`):** {}\n",
        summary.excluded_crates.join(", ")
    ));
    md.push_str(&format!(
        "- **Analysierte Symbol-Arten (`--kind`):** {}\n",
        summary.kinds_filter.join(", ")
    ));
    md.push_str("- **Klassen-Zählung:**\n");
    md.push_str(&format!("  - **ORPHAN:** {}\n", summary.orphan_count));
    md.push_str(&format!("  - **NUR_TESTS:** {}\n", summary.nur_tests_count));
    md.push_str(&format!("  - **PRODUKTIV:** {}\n", summary.produktiv_count));
    md.push_str(&format!("  - **VERDACHT:** {}\n", summary.verdacht_count));
    md.push_str(&format!(
        "  - **Ausgeschlossene Grenz-Crate-Funde:** {}\n\n",
        summary.excluded_boundary_count
    ));
    md.push_str(&format!("*{}\n\n", summary.disclaimer));

    md.push_str("## ORPHAN\n\n");
    if result.orphan.is_empty() {
        md.push_str("Keine Funde in dieser Kategorie.\n\n");
    } else {
        md.push_str("| Art | Symbol | Crate | Datei | Zeile |\n");
        md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
        for f in &result.orphan {
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | {} |\n",
                f.kind, f.symbol, f.krate, f.file, f.line
            ));
        }
        md.push('\n');
    }

    md.push_str("## NUR_TESTS\n\n");
    if result.nur_tests.is_empty() {
        md.push_str("Keine Funde in dieser Kategorie.\n\n");
    } else {
        md.push_str("| Art | Symbol | Crate | Datei | Zeile |\n");
        md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
        for f in &result.nur_tests {
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | {} |\n",
                f.kind, f.symbol, f.krate, f.file, f.line
            ));
        }
        md.push('\n');
    }

    md.push_str("## VERDACHT\n\n");
    if result.verdacht.is_empty() {
        md.push_str("Keine Funde in dieser Kategorie.\n\n");
    } else {
        md.push_str("| Art | Symbol | Crate | Datei | Zeile |\n");
        md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
        for f in &result.verdacht {
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | {} |\n",
                f.kind, f.symbol, f.krate, f.file, f.line
            ));
        }
        md.push('\n');
    }

    md.push_str("## Hinweis\n\n");
    md.push_str(&format!("{}\n", note_text));

    if let Some(out_path) = out_file_path {
        if let Err(e) = fs::write(&out_path, &md) {
            eprintln!(
                "Fehler beim Schreiben des Reports in '{}': {}",
                out_path.display(),
                e
            );
            return 1;
        }
    }

    if use_json {
        match serde_json::to_string_pretty(&result) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Fehler bei JSON-Serialisierung: {}", e);
                return 1;
            }
        }
    } else {
        print!("{}", md);
    }

    if status == "pass" {
        0
    } else {
        2
    }
}
