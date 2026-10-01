//! Test Integrity Gate: syn-basierte Prüfung geänderter/gelöschter Rust-Testdateien.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use syn::visit::Visit;
use syn::{Attribute, Expr, ExprCall, ExprMethodCall, ItemFn, Macro, ReturnType};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestIntegrityFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestIntegrityGateResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<TestIntegrityFinding>,
}

#[derive(Default)]
pub struct TestIntegrityAssertionVisitor {
    pub has_assertion: bool,
    pub has_assert_true: bool,
}

impl<'ast> Visit<'ast> for TestIntegrityAssertionVisitor {
    fn visit_macro(&mut self, node: &'ast Macro) {
        let mac_name = node
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        if mac_name.starts_with("assert") || mac_name == "matches" {
            self.has_assertion = true;
            let tokens_str = node.tokens.to_string();
            if mac_name == "assert" && tokens_str.trim() == "true" {
                self.has_assert_true = true;
            }
        }
        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(ref ep) = *node.func {
            if let Some(seg) = ep.path.segments.last() {
                let name = seg.ident.to_string();
                if test_integrity_is_assertion_helper(&name) {
                    self.has_assertion = true;
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let name = node.method.to_string();
        if test_integrity_is_assertion_helper(&name) {
            self.has_assertion = true;
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr(&mut self, node: &'ast Expr) {
        if let Expr::Try(_) = node {
            self.has_assertion = true;
        }
        syn::visit::visit_expr(self, node);
    }
}

pub fn test_integrity_is_assertion_helper(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("assert")
        || lower.contains("check")
        || lower.contains("verify")
        || lower.starts_with("expect_")
}

pub fn test_integrity_is_test_attr(attr: &Attribute) -> bool {
    let path_str = attr
        .path()
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    path_str == "test" || path_str == "tokio::test"
}

pub fn test_integrity_has_should_panic(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path()
            .segments
            .last()
            .is_some_and(|s| s.ident == "should_panic")
    })
}

pub fn test_integrity_has_ignore_attr(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|s| s.ident == "ignore")
}

pub fn run_test_integrity(args: &[String]) -> i32 {
    let mut root_dir = test_integrity_default_root();
    let mut base = String::new();
    let mut head = String::from("HEAD");
    let mut strict = false;
    let mut json_output = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                if i + 1 < args.len() {
                    root_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--base" => {
                if i + 1 < args.len() {
                    base = args[i + 1].clone();
                    i += 1;
                }
            }
            "--head" => {
                if i + 1 < args.len() {
                    head = args[i + 1].clone();
                    i += 1;
                }
            }
            "--strict" => {
                strict = true;
            }
            "--json" => {
                json_output = true;
            }
            _ => {}
        }
        i += 1;
    }

    if base.is_empty() {
        base = test_integrity_git_merge_base(&root_dir, &head);
    }

    let diff_files = test_integrity_get_changed_files(&root_dir, &base, &head);
    let mut findings = Vec::new();

    let has_non_test_src_changes = diff_files.iter().any(|f| {
        f.ends_with(".rs")
            && !f.contains("/tests/")
            && !f.ends_with("_test.rs")
            && !f.contains("tests.rs")
    });

    // Check script files in diff for insta bless / INSTA_UPDATE
    for file in &diff_files {
        if file.ends_with(".sh")
            || file.ends_with(".py")
            || file.ends_with(".yml")
            || file.ends_with(".yaml")
        {
            let full_path = root_dir.join(file);
            if let Ok(content) = fs::read_to_string(&full_path) {
                if content.contains("INSTA_UPDATE")
                    || content.contains("cargo insta accept")
                    || content.contains("--bless")
                {
                    findings.push(TestIntegrityFinding {
                        id: "TI-006".to_string(),
                        severity: "error".to_string(),
                        file: file.clone(),
                        line: 0,
                        message: "Nutzung von INSTA_UPDATE / cargo insta accept / --bless in Skripten im Diff entdeckt".to_string(),
                        fix: "Automatisches Segnen von Snapshots aus CI/Skripten entfernen".to_string(),
                    });
                }
            }
        }

        // Check snapshot files changed without non-test source code changes
        if (file.ends_with(".snap") || file.contains("/snapshots/")) && !has_non_test_src_changes {
            findings.push(TestIntegrityFinding {
                id: "TI-005".to_string(),
                severity: "error".to_string(),
                file: file.clone(),
                line: 0,
                message: "Snapshot-Datei geändert, ohne dass Nicht-Test-Quellcode im selben Diff geändert wurde".to_string(),
                fix: "Prüfen ob Snapshot-Änderung berechtigt ist und Nicht-Test-Quellcode anpassen".to_string(),
            });
        }
    }

    // Inspect deleted files / deleted test functions
    let deleted_test_files = test_integrity_get_deleted_files(&root_dir, &base, &head);
    for del in &deleted_test_files {
        if del.starts_with("tests/") || del.contains("/tests/") {
            findings.push(TestIntegrityFinding {
                id: "TI-003".to_string(),
                severity: "error".to_string(),
                file: del.clone(),
                line: 0,
                message: format!("Testdatei '{}' wurde ohne Ersatz gelöscht", del),
                fix: "Gelöschte Testdatei wiederherstellen oder Ersatz im selben Diff nachweisen"
                    .to_string(),
            });
        }
    }

    // Analyze Rust file ASTs in diff
    for file in &diff_files {
        if !file.ends_with(".rs") {
            continue;
        }

        let full_path = root_dir.join(file);
        if !full_path.exists() {
            continue;
        }

        let content = match fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let file_ast = match syn::parse_file(&content) {
            Ok(ast) => ast,
            Err(_) => continue,
        };

        let added_lines = test_integrity_get_added_lines_for_file(&root_dir, &base, &head, file);

        for item in &file_ast.items {
            if let syn::Item::Fn(ref item_fn) = item {
                test_integrity_check_fn(
                    file,
                    item_fn,
                    &added_lines,
                    has_non_test_src_changes,
                    strict,
                    &mut findings,
                );
            }
        }
    }

    let has_error = findings.iter().any(|f| f.severity == "error");
    if has_error {
        let res = TestIntegrityGateResult {
            gate: "test-integrity".to_string(),
            status: "fail".to_string(),
            summary: format!(
                "{} Test-Integritätsverstoß/Verstöße gefunden",
                findings.len()
            ),
            findings,
        };
        test_integrity_emit(res, json_output, 1)
    } else {
        let res = TestIntegrityGateResult {
            gate: "test-integrity".to_string(),
            status: "pass".to_string(),
            summary: "Alle geänderten Tests erfüllen die Integritätsanforderungen".to_string(),
            findings,
        };
        test_integrity_emit(res, json_output, 0)
    }
}

pub fn test_integrity_check_fn(
    file: &str,
    item_fn: &ItemFn,
    added_lines: &HashSet<usize>,
    has_non_test_src_changes: bool,
    strict: bool,
    findings: &mut Vec<TestIntegrityFinding>,
) {
    let fn_name = item_fn.sig.ident.to_string();
    let is_test = item_fn.attrs.iter().any(test_integrity_is_test_attr);

    if !is_test {
        return;
    }

    // TI-004: Newly added #[ignore]
    for attr in &item_fn.attrs {
        if test_integrity_has_ignore_attr(attr) {
            findings.push(TestIntegrityFinding {
                id: "TI-004".to_string(),
                severity: "error".to_string(),
                file: file.to_string(),
                line: 0,
                message: format!("Neu hinzugefügtes #[ignore] bei Test '{}'", fn_name),
                fix: "#[ignore] entfernen und Test korrigieren".to_string(),
            });
        }
    }

    // Check assertions in test body
    let mut visitor = TestIntegrityAssertionVisitor::default();

    if test_integrity_has_should_panic(&item_fn.attrs) {
        visitor.has_assertion = true;
    }

    if let ReturnType::Type(_, ref ty) = item_fn.sig.output {
        let ty_str = quote::quote!(#ty).to_string();
        if ty_str.contains("Result") {
            visitor.has_assertion = true;
        }
    }

    visitor.visit_block(&item_fn.block);

    // TI-001: Test without assertions
    if !visitor.has_assertion {
        findings.push(TestIntegrityFinding {
            id: "TI-001".to_string(),
            severity: "error".to_string(),
            file: file.to_string(),
            line: 0,
            message: format!("Testfunktion '{}' enthält keinerlei Assertion", fn_name),
            fix: "Assertion (assert!, matches!, Result-Rückgabe) ergänzen".to_string(),
        });
    }

    // TI-002: assert!(true) tautology
    if visitor.has_assert_true {
        findings.push(TestIntegrityFinding {
            id: "TI-002".to_string(),
            severity: "error".to_string(),
            file: file.to_string(),
            line: 0,
            message: format!(
                "Tautologische Assertion 'assert!(true)' in Test '{}'",
                fn_name
            ),
            fix: "Tautologische Assertion durch aussagekräftige Prüfung ersetzen".to_string(),
        });
    }

    // TI-007 (Soft finding): Modified literal expectation in assert_eq! without non-test source changes
    if !has_non_test_src_changes && !added_lines.is_empty() {
        let fn_code = quote::quote!(#item_fn).to_string();
        if fn_code.contains("assert_eq !") {
            let severity = if strict { "error" } else { "warn" };
            findings.push(TestIntegrityFinding {
                id: "TI-007".to_string(),
                severity: severity.to_string(),
                file: file.to_string(),
                line: 0,
                message: format!(
                    "Mögliche Überanpassung: Geänderte assert_eq!-Erwartung in '{}' ohne Änderung an Nicht-Test-Quellcode",
                    fn_name
                ),
                fix: "Prüfen ob die Erwartungswert-Änderung fachlich begründet ist".to_string(),
            });
        }
    }
}

fn test_integrity_emit(res: TestIntegrityGateResult, json_output: bool, exit_code: i32) -> i32 {
    if json_output {
        println!("{}", serde_json::to_string(&res).unwrap_or_default());
    } else {
        println!("=== Gate test-integrity: {} ===", res.status);
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

fn test_integrity_default_root() -> PathBuf {
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

fn test_integrity_git_merge_base(root: &Path, head: &str) -> String {
    let output = Command::new("git")
        .current_dir(root)
        .args(["merge-base", head, "origin/main"])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    let output_head = Command::new("git")
        .current_dir(root)
        .args(["rev-parse", &format!("{}^", head)])
        .output();
    if let Ok(out) = output_head {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    head.to_string()
}

fn test_integrity_get_changed_files(root: &Path, base: &str, head: &str) -> Vec<String> {
    let range = if base == head {
        format!("{}^..{}", head, head)
    } else {
        format!("{}...{}", base, head)
    };
    let output = Command::new("git")
        .current_dir(root)
        .args(["diff", "--name-only", &range])
        .output();

    let mut files = Vec::new();
    if let Ok(out) = output {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                files.push(trimmed.to_string());
            }
        }
    }
    files
}

fn test_integrity_get_deleted_files(root: &Path, base: &str, head: &str) -> Vec<String> {
    let range = if base == head {
        format!("{}^..{}", head, head)
    } else {
        format!("{}...{}", base, head)
    };
    let output = Command::new("git")
        .current_dir(root)
        .args(["diff", "--name-only", "--diff-filter=D", &range])
        .output();

    let mut files = Vec::new();
    if let Ok(out) = output {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                files.push(trimmed.to_string());
            }
        }
    }
    files
}

fn test_integrity_get_added_lines_for_file(
    root: &Path,
    base: &str,
    head: &str,
    file: &str,
) -> HashSet<usize> {
    let range = if base == head {
        format!("{}^..{}", head, head)
    } else {
        format!("{}...{}", base, head)
    };
    let output = Command::new("git")
        .current_dir(root)
        .args(["diff", "-U0", &range, "--", file])
        .output();

    let mut added = HashSet::new();
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            if line.starts_with("@@") {
                // Parse @@ -a,b +c,d @@
                if let Some(plus_idx) = line.find('+') {
                    let rest = &line[plus_idx + 1..];
                    let num_str = rest.split([',', ' ']).next().unwrap_or("0");
                    if let Ok(line_num) = num_str.parse::<usize>() {
                        added.insert(line_num);
                    }
                }
            }
        }
    }
    added
}
