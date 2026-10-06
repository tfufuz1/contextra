//! Harness Modul: findet verdächtige Stub-/Platzhalter-Methodenkörper in impl-Blöcken (stub-impls).

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityLevel {
    Niedrig = 1,
    Mittel = 2,
    Hoch = 3,
}

impl SeverityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            SeverityLevel::Hoch => "HOCH",
            SeverityLevel::Mittel => "MITTEL",
            SeverityLevel::Niedrig => "NIEDRIG",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StubFinding {
    pub severity: String,
    pub krate: String,
    pub type_name: String,
    pub trait_name: Option<String>,
    pub method: String,
    pub reason: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StubGateResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<StubFinding>,
}

struct MacroVisitor<'a> {
    found: Option<&'a str>,
}

impl<'a, 'ast> Visit<'ast> for MacroVisitor<'a> {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if self.found.is_some() {
            return;
        }
        if let Some(seg) = mac.path.segments.last() {
            let ident = seg.ident.to_string();
            if ident == "todo" {
                self.found = Some("todo!");
            } else if ident == "unimplemented" {
                self.found = Some("unimplemented!");
            } else if ident == "unreachable" {
                self.found = Some("unreachable!");
            }
        }
        syn::visit::visit_macro(self, mac);
    }
}

fn find_stub_macro(block: &syn::Block) -> Option<&'static str> {
    let mut visitor = MacroVisitor { found: None };
    visitor.visit_block(block);
    visitor.found
}

fn is_non_unit_return(output: &syn::ReturnType) -> bool {
    match output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => {
            if let syn::Type::Tuple(tup) = ty.as_ref() {
                !tup.elems.is_empty()
            } else {
                true
            }
        }
    }
}

fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("cfg") {
            let meta_str = quote::quote!(#attr).to_string().replace(' ', "");
            meta_str.contains("cfg(test)")
        } else {
            false
        }
    })
}

fn match_trivial_expr(stmt: &syn::Stmt) -> Option<(&'static str, &'static str)> {
    let expr_tokens = match stmt {
        syn::Stmt::Expr(expr, _) => quote::quote!(#expr),
        syn::Stmt::Macro(stmt_mac) => quote::quote!(#stmt_mac.mac),
        _ => return None,
    };
    let expr_str = expr_tokens.to_string();
    let norm = expr_str
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let norm = norm.trim_end_matches(';');

    let literals = [
        ("Ok(())", "Ok(())"),
        ("Ok(Default::default())", "Ok(Default::default())"),
        ("Default::default()", "Default::default()"),
        ("None", "None"),
        ("vec![]", "vec![]"),
        ("0", "0"),
        ("false", "false"),
        ("true", "true"),
        ("\"\"", "\"\""),
        ("Vec::new()", "Vec::new()"),
        ("HashMap::new()", "HashMap::new()"),
        ("Self::default()", "Self::default()"),
        ("()", "()"),
    ];

    for (display, lit) in literals {
        let lit_norm = lit
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        if norm == lit_norm {
            return Some((display, lit));
        }
    }
    None
}

struct ItemVisitor<'a> {
    file_path: &'a str,
    krate: &'a str,
    findings: Vec<StubFinding>,
    in_cfg_test: bool,
}

impl<'a, 'ast> Visit<'ast> for ItemVisitor<'a> {
    fn visit_item_mod(&mut self, item_mod: &'ast syn::ItemMod) {
        let prev = self.in_cfg_test;
        if is_cfg_test(&item_mod.attrs) {
            self.in_cfg_test = true;
        }
        if !self.in_cfg_test {
            syn::visit::visit_item_mod(self, item_mod);
        }
        self.in_cfg_test = prev;
    }

    fn visit_item_impl(&mut self, item_impl: &'ast syn::ItemImpl) {
        if self.in_cfg_test || is_cfg_test(&item_impl.attrs) {
            return;
        }

        let type_name = quote::quote!(#item_impl.self_ty)
            .to_string()
            .replace(' ', "");
        let trait_name = item_impl
            .trait_
            .as_ref()
            .map(|(_, path, _)| quote::quote!(#path).to_string().replace(' ', ""));

        for item in &item_impl.items {
            if let syn::ImplItem::Fn(method) = item {
                if is_cfg_test(&method.attrs) {
                    continue;
                }

                let method_name = method.sig.ident.to_string();
                let line = method.sig.ident.span().start().line;
                let block = &method.block;

                // Priority 1: HOCH
                if let Some(mac_name) = find_stub_macro(block) {
                    self.findings.push(StubFinding {
                        severity: SeverityLevel::Hoch.as_str().to_string(),
                        krate: self.krate.to_string(),
                        type_name: type_name.clone(),
                        trait_name: trait_name.clone(),
                        method: method_name,
                        reason: format!("enthält {} Macro", mac_name),
                        file: self.file_path.to_string(),
                        line,
                    });
                    continue;
                }

                if block.stmts.is_empty() && is_non_unit_return(&method.sig.output) {
                    self.findings.push(StubFinding {
                        severity: SeverityLevel::Hoch.as_str().to_string(),
                        krate: self.krate.to_string(),
                        type_name: type_name.clone(),
                        trait_name: trait_name.clone(),
                        method: method_name,
                        reason: "leerer Methodenkörper bei Nicht-Void-Rückgabetyp".to_string(),
                        file: self.file_path.to_string(),
                        line,
                    });
                    continue;
                }

                // Priority 2: MITTEL
                if block.stmts.len() == 1 {
                    if let Some((display_lit, _lit)) = match_trivial_expr(&block.stmts[0]) {
                        // EXCEPTION check:
                        let is_constructor_idiom = (method_name == "new"
                            || method_name == "default")
                            && (display_lit == "Self::default()"
                                || display_lit == "Default::default()");

                        if !is_constructor_idiom {
                            self.findings.push(StubFinding {
                                severity: SeverityLevel::Mittel.as_str().to_string(),
                                krate: self.krate.to_string(),
                                type_name: type_name.clone(),
                                trait_name: trait_name.clone(),
                                method: method_name,
                                reason: format!("Trivialer Rückgabe-Ausdruck ({})", display_lit),
                                file: self.file_path.to_string(),
                                line,
                            });
                            continue;
                        }
                    }
                }

                // Priority 3: NIEDRIG
                let body_str = quote::quote!(#block).to_string();
                if body_str.len() < 200 {
                    let re = Regex::new(r"(?i)\b(stub|placeholder|mock|fake|dummy)\b").unwrap();
                    if let Some(mat) = re.find(&body_str) {
                        self.findings.push(StubFinding {
                            severity: SeverityLevel::Niedrig.as_str().to_string(),
                            krate: self.krate.to_string(),
                            type_name: type_name.clone(),
                            trait_name: trait_name.clone(),
                            method: method_name,
                            reason: format!(
                                "Verdächtiger Bezeichner ({}) in Methodenkörper < 200 Zeichen",
                                mat.as_str()
                            ),
                            file: self.file_path.to_string(),
                            line,
                        });
                        continue;
                    }
                }
            }
        }

        syn::visit::visit_item_impl(self, item_impl);
    }
}

fn find_root_dir() -> PathBuf {
    if let Ok(cargo_manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(cargo_manifest);
        if path.file_name().and_then(|s| s.to_str()) == Some("xtask") {
            if let Some(parent) = path.parent() {
                return parent.to_path_buf();
            }
        }
        return path;
    }
    let mut curr = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if curr.join("Cargo.toml").exists() && curr.join("capabilities.toml").exists() {
            return curr;
        }
        if !curr.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

fn collect_rs_files(root: &Path, crate_filter: Option<&str>) -> Vec<(PathBuf, String, String)> {
    let mut result = Vec::new();
    let scan_dir = if root.join("crates").exists() {
        root.join("crates")
    } else {
        root.to_path_buf()
    };

    let walk = walkdir::WalkDir::new(&scan_dir)
        .into_iter()
        .filter_map(|e| e.ok());

    for entry in walk {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            if rel_path.contains("/tests/")
                || rel_path.contains("/benches/")
                || rel_path.starts_with("tests/")
                || rel_path.starts_with("benches/")
            {
                continue;
            }

            let krate = if rel_path.starts_with("crates/") {
                let parts: Vec<&str> = rel_path.split('/').collect();
                if parts.len() >= 2 {
                    parts[1].to_string()
                } else {
                    "unknown".to_string()
                }
            } else {
                let parts: Vec<&str> = rel_path.split('/').collect();
                parts.first().copied().unwrap_or("root").to_string()
            };

            if let Some(filter) = crate_filter {
                if krate != filter {
                    continue;
                }
            }

            result.push((path.to_path_buf(), rel_path, krate));
        }
    }
    result.sort_by(|a, b| a.1.cmp(&b.1));
    result
}

pub fn run_stub_impls(args: &[String]) -> i32 {
    run_stub_impls_with_writer(args, &mut std::io::stdout())
}

pub fn run_stub_impls_with_writer<W: Write>(args: &[String], writer: &mut W) -> i32 {
    let mut root_dir = find_root_dir();
    let mut json_output = false;
    let mut crate_filter: Option<String> = None;
    let mut min_severity = SeverityLevel::Niedrig;

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
            "--crate" => {
                if i + 1 < args.len() {
                    crate_filter = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--min-severity" => {
                if i + 1 < args.len() {
                    let sev_str = args[i + 1].to_lowercase();
                    min_severity = match sev_str.as_str() {
                        "hoch" => SeverityLevel::Hoch,
                        "mittel" => SeverityLevel::Mittel,
                        _ => SeverityLevel::Niedrig,
                    };
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let files = collect_rs_files(&root_dir, crate_filter.as_deref());
    let mut all_findings = Vec::new();

    for (abs_path, rel_path, krate) in files {
        let content = match fs::read_to_string(&abs_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let syntax = match syn::parse_file(&content) {
            Ok(syn_file) => syn_file,
            Err(_) => continue,
        };

        let mut visitor = ItemVisitor {
            file_path: &rel_path,
            krate: &krate,
            findings: Vec::new(),
            in_cfg_test: false,
        };
        visitor.visit_file(&syntax);

        all_findings.extend(visitor.findings);
    }

    let filtered_findings: Vec<StubFinding> = all_findings
        .into_iter()
        .filter(|f| {
            let sev_level = match f.severity.as_str() {
                "HOCH" => SeverityLevel::Hoch,
                "MITTEL" => SeverityLevel::Mittel,
                _ => SeverityLevel::Niedrig,
            };
            sev_level >= min_severity
        })
        .collect();

    let has_hoch = filtered_findings.iter().any(|f| f.severity == "HOCH");
    let status = if has_hoch { "fail" } else { "pass" };

    let summary = format!(
        "{} verdächtige Stub-/Platzhalter-Implementierung(en) gefunden.",
        filtered_findings.len()
    );

    let gate_res = StubGateResult {
        gate: "stub-impls".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        findings: filtered_findings.clone(),
    };

    if json_output {
        let json_str = serde_json::to_string_pretty(&gate_res).unwrap_or_default();
        let _ = writeln!(writer, "{}", json_str);
    } else {
        let _ = writeln!(
            writer,
            "### Stub Implementations Report ({})\n",
            status.to_uppercase()
        );
        if filtered_findings.is_empty() {
            let _ = writeln!(
                writer,
                "Keine verdächtigen Stub-/Platzhalter-Methodenkörper gefunden.\n"
            );
        } else {
            let _ = writeln!(
                writer,
                "| Severity | Crate | Type / Trait | Method | Reason | Location |"
            );
            let _ = writeln!(writer, "|---|---|---|---|---|---|");
            for f in &filtered_findings {
                let icon = match f.severity.as_str() {
                    "HOCH" => "🔴 HOCH",
                    "MITTEL" => "🟡 MITTEL",
                    _ => "🔵 NIEDRIG",
                };
                let type_trait = match &f.trait_name {
                    Some(tr) => format!("impl {} for {}", tr, f.type_name),
                    None => f.type_name.clone(),
                };
                let _ = writeln!(
                    writer,
                    "| {} | {} | {} | {} | {} | {}:{} |",
                    icon, f.krate, type_trait, f.method, f.reason, f.file, f.line
                );
            }
            let _ = writeln!(
                writer,
                "\n> **Hinweis:** Funde der Stufen MITTEL und NIEDRIG können legitime No-Ops oder vereinfachte Default-Implementierungen sein.\n"
            );
        }
    }

    if status == "fail" {
        1
    } else {
        0
    }
}
