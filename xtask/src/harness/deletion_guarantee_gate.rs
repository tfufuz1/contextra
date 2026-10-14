//! Deletion Guarantee Quality Gate: Statische AST-Analyse von Löschgarantien und Crypto-Sicherheitsinvarianten.
// FILE-CONTEXT (STAND 2026-10-13, ZWECK, INVARIANTEN)
// ZWECK: Quality Gate for static analysis of deletion guarantees and crypto security invariants.
// INVARIANTEN: Zero-Panic, Fail-closed (Exit Code 2 bei Verstößen).

use crate::harness::scan_common::{classify_file, FileClass};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use syn::visit::Visit;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionGateFinding {
    pub rule: String,
    pub severity: String, // "error" | "warning"
    pub file_path: String,
    pub line_num: usize,
    pub description: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeletionGateReport {
    pub gate: String,
    pub status: String, // "pass" | "fail"
    pub summary: String,
    pub errors_count: usize,
    pub warnings_count: usize,
    pub findings: Vec<DeletionGateFinding>,
}

pub fn run_deletion_guarantee_gate(args: &[String]) -> i32 {
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut json = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                if i + 1 < args.len() {
                    root = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--json" => {
                json = true;
            }
            _ => {}
        }
        i += 1;
    }

    let mut findings = Vec::new();

    // Collect all .rs files
    let external_test_files = HashSet::new();
    let mut rs_files = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target" && name != ".git" && name != "node_modules"
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            rs_files.push(path.to_path_buf());
        }
    }

    // Map domain tags for Rule (f)
    let mut domain_tags: HashMap<String, Vec<(String, usize)>> = HashMap::new();

    for file_path in &rs_files {
        let rel_path = file_path
            .strip_prefix(&root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .replace('\\', "/");

        // Skip xtask itself from domain checks
        if rel_path.starts_with("xtask/") || rel_path.starts_with("xtask-heavy/") {
            continue;
        }

        let Ok(content) = fs::read_to_string(file_path) else {
            continue;
        };

        let class = classify_file(file_path, &content, &external_test_files);
        let is_prod = class == FileClass::Production;

        // Hint check: `proof: null` literal in code or json
        for (line_idx, line) in content.lines().enumerate() {
            if line.contains("proof: null") || line.contains("\"proof\": null") {
                findings.push(DeletionGateFinding {
                    rule: "hint_proof_null".to_string(),
                    severity: "warning".to_string(),
                    file_path: rel_path.clone(),
                    line_num: line_idx + 1,
                    description: "Literal 'proof: null' detected".to_string(),
                });
            }
        }

        let Ok(syntax) = syn::parse_file(&content) else {
            continue;
        };

        // AST Visitor
        let mut visitor = DeletionVisitor {
            file_path: &rel_path,
            is_prod,
            content: &content,
            findings: Vec::new(),
            domain_tags: &mut domain_tags,
        };
        visitor.visit_file(&syntax);

        findings.extend(visitor.findings);
    }

    // Rule (f): Duplicate Domain Tag check
    for (tag, locs) in domain_tags {
        if locs.len() > 1 {
            for (file_path, line_num) in &locs {
                findings.push(DeletionGateFinding {
                    rule: "f_duplicate_domain_tag".to_string(),
                    severity: "error".to_string(),
                    file_path: file_path.clone(),
                    line_num: *line_num,
                    description: format!(
                        "Domain-Tag '{}' ist doppelt vergeben (Gefunden in {} Stellen)",
                        tag,
                        locs.len()
                    ),
                });
            }
        }
    }

    let errors_count = findings.iter().filter(|f| f.severity == "error").count();
    let warnings_count = findings.iter().filter(|f| f.severity == "warning").count();
    let status = if errors_count > 0 { "fail" } else { "pass" };

    let summary = format!(
        "deletion-guarantee-gate: {} Fehler, {} Hinweise",
        errors_count, warnings_count
    );

    let report = DeletionGateReport {
        gate: "deletion-guarantee-gate".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        errors_count,
        warnings_count,
        findings,
    };

    if json {
        if let Ok(rendered) = serde_json::to_string_pretty(&report) {
            println!("{}", rendered);
        }
    } else {
        println!("{}", summary);
        for f in &report.findings {
            let prefix = if f.severity == "error" { "❌ ERROR" } else { "💡 HINT" };
            println!("{} [{}] {}:{} — {}", prefix, f.rule, f.file_path, f.line_num, f.description);
        }
    }

    if status == "fail" {
        2
    } else {
        0
    }
}

struct DeletionVisitor<'a> {
    file_path: &'a str,
    is_prod: bool,
    content: &'a str,
    findings: Vec<DeletionGateFinding>,
    domain_tags: &'a mut HashMap<String, Vec<(String, usize)>>,
}

impl<'a> Visit<'a> for DeletionVisitor<'a> {
    fn visit_item_fn(&mut self, i: &'a syn::ItemFn) {
        let fn_name = i.sig.ident.to_string();

        // (b) `Ok(None)` in `issue_*witness*`-Funktionen
        if fn_name.contains("issue_") && fn_name.contains("witness") {
            let mut ok_none_visitor = OkNoneVisitor {
                found_ok_none: false,
                line_num: 0,
            };
            ok_none_visitor.visit_block(&i.block);

            if ok_none_visitor.found_ok_none {
                let line_num = ok_none_visitor.line_num;
                self.findings.push(DeletionGateFinding {
                    rule: "b_ok_none_witness".to_string(),
                    severity: "error".to_string(),
                    file_path: self.file_path.to_string(),
                    line_num,
                    description: format!(
                        "Funktion '{}' gibt 'Ok(None)' zurück, was Zeugenüberprüfungen umgeht",
                        fn_name
                    ),
                });
            }
        }

        syn::visit::visit_item_fn(self, i);
    }

    fn visit_expr_call(&mut self, i: &'a syn::ExprCall) {
        // (a) `db_instance_id`-basierte Seed-Ableitung im Produktionscode
        if self.is_prod {
            let expr_str = quote::quote!(#i).to_string();
            if expr_str.contains("db_instance_id")
                && (expr_str.contains("seed") || expr_str.contains("derive") || expr_str.contains("Kdf") || expr_str.contains("KeyManager"))
            {
                let line_num = get_line_num(self.content, i);
                self.findings.push(DeletionGateFinding {
                    rule: "a_db_instance_id_seed".to_string(),
                    severity: "error".to_string(),
                    file_path: self.file_path.to_string(),
                    line_num,
                    description: "`db_instance_id`-basierte Seed-Ableitung im Produktionscode entdeckt".to_string(),
                });
            }
        }

        syn::visit::visit_expr_call(self, i);
    }

    fn visit_expr_method_call(&mut self, i: &'a syn::ExprMethodCall) {
        let method_name = i.method.to_string();

        // (c) `unwrap_or(0)` / `unwrap_or_default` in Dateien unter `deletion/` oder `deletion_proof/`
        let in_deletion_dir = self.file_path.contains("/deletion/")
            || self.file_path.contains("/deletion_proof/")
            || self.file_path.contains("deletion_proof");

        if in_deletion_dir && self.is_prod {
            if method_name == "unwrap_or_default" {
                let line_num = get_line_num(self.content, i);
                self.findings.push(DeletionGateFinding {
                    rule: "c_unwrap_or_in_deletion".to_string(),
                    severity: "error".to_string(),
                    file_path: self.file_path.to_string(),
                    line_num,
                    description: "Verwendung von 'unwrap_or_default()' in Löschmodul entdeckt".to_string(),
                });
            } else if method_name == "unwrap_or" {
                let expr_str = quote::quote!(#i).to_string();
                if expr_str.contains("unwrap_or (0)") || expr_str.contains("unwrap_or(0)") {
                    let line_num = get_line_num(self.content, i);
                    self.findings.push(DeletionGateFinding {
                        rule: "c_unwrap_or_in_deletion".to_string(),
                        severity: "error".to_string(),
                        file_path: self.file_path.to_string(),
                        line_num,
                        description: "Verwendung von 'unwrap_or(0)' in Löschmodul entdeckt".to_string(),
                    });
                }
            }
        }

        // (d) `.verify(` auf Ed25519-Schlüsseln statt `verify_strict` in `deletion_proof/`
        if self.file_path.contains("deletion_proof") && method_name == "verify" {
            let expr_str = quote::quote!(#i).to_string();
            if expr_str.contains("Ed25519") || expr_str.contains("ed25519") || expr_str.contains("VerifyingKey") || expr_str.contains("PublicKey") {
                let line_num = get_line_num(self.content, i);
                self.findings.push(DeletionGateFinding {
                    rule: "d_ed25519_verify_not_strict".to_string(),
                    severity: "error".to_string(),
                    file_path: self.file_path.to_string(),
                    line_num,
                    description: "Verwendung von '.verify(' auf Ed25519-Schlüssel anstelle von 'verify_strict(' in deletion_proof".to_string(),
                });
            }
        }

        syn::visit::visit_expr_method_call(self, i);
    }

    fn visit_local(&mut self, i: &'a syn::Local) {
        // (e) `let _ =` auf Ergebnisse von `revoke_group`/`record_phase`
        let local_str = quote::quote!(#i).to_string();
        if local_str.starts_with("let _ =") || local_str.starts_with("let _ :") {
            if local_str.contains("revoke_group") || local_str.contains("record_phase") {
                let line_num = get_line_num(self.content, i);
                self.findings.push(DeletionGateFinding {
                    rule: "e_ignored_security_result".to_string(),
                    severity: "error".to_string(),
                    file_path: self.file_path.to_string(),
                    line_num,
                    description: "Ergebnis von 'revoke_group' oder 'record_phase' mit 'let _ =' ignoriert".to_string(),
                });
            }
        }

        syn::visit::visit_local(self, i);
    }

    fn visit_item_const(&mut self, i: &'a syn::ItemConst) {
        // (f) Domain tag duplication tracking
        let const_name = i.ident.to_string();
        if const_name.contains("TAG") || const_name.contains("DOMAIN") || const_name.contains("CONTEXT") {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &*i.expr
            {
                let tag_val = s.value();
                if tag_val.starts_with("contextra/") {
                    let line_num = get_line_num(self.content, i);
                    self.domain_tags
                        .entry(tag_val)
                        .or_default()
                        .push((self.file_path.to_string(), line_num));
                }
            }
        }

        syn::visit::visit_item_const(self, i);
    }
}

struct OkNoneVisitor {
    found_ok_none: bool,
    line_num: usize,
}

impl<'a> Visit<'a> for OkNoneVisitor {
    fn visit_expr_call(&mut self, i: &'a syn::ExprCall) {
        let call_str = quote::quote!(#i).to_string();
        if call_str.replace(' ', "").contains("Ok(None)") {
            self.found_ok_none = true;
        }
        syn::visit::visit_expr_call(self, i);
    }
}

fn get_line_num<T: quote::ToTokens>(content: &str, node: &T) -> usize {
    let tokens = quote::quote!(#node).to_string();
    let first_word = tokens.split_whitespace().next().unwrap_or("");
    if first_word.is_empty() {
        return 1;
    }
    for (idx, line) in content.lines().enumerate() {
        if line.contains(first_word) {
            return idx + 1;
        }
    }
    1
}
