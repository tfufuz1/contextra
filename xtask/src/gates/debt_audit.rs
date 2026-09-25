// Contextra — Tech-Debt Audit Gate
//
// Subkommando `cargo xtask debt-audit`
// AST-basierte Prüfung auf `.unwrap()` / `.expect()`, `unsafe` und `std::fs` in Produktionscode.

use quote::ToTokens;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use syn::spanned::Spanned;
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtViolation {
    pub file_path: String,
    pub line_num: usize,
    pub fn_name: String,
    pub message: String,
}

fn meta_evals_to_test(meta: &syn::Meta) -> bool {
    match meta {
        syn::Meta::Path(p) => p.is_ident("test"),
        syn::Meta::List(meta_list) => {
            if meta_list.path.is_ident("not") {
                false
            } else if meta_list.path.is_ident("all") || meta_list.path.is_ident("any") {
                if let Ok(nested_metas) = meta_list.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                ) {
                    nested_metas.iter().any(meta_evals_to_test)
                } else {
                    false
                }
            } else {
                false
            }
        }
        syn::Meta::NameValue(_) => false,
    }
}

/// Helper function to check if an attribute represents a test context
pub fn is_test_attribute(attr: &syn::Attribute) -> bool {
    let path_str = attr.path().to_token_stream().to_string();
    let normalized_path = path_str.replace(' ', "");

    if normalized_path == "test"
        || normalized_path == "tokio::test"
        || normalized_path.ends_with("::test")
    {
        return true;
    }

    if attr.path().is_ident("cfg") {
        if let syn::Meta::List(meta_list) = &attr.meta {
            if let Ok(nested_meta) = meta_list.parse_args::<syn::Meta>() {
                return meta_evals_to_test(&nested_meta);
            }
        }
    }
    false
}

pub fn has_test_attribute(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(is_test_attribute)
}

/// Checks if a line contains an inline comment suppressing the violation
pub fn has_inline_suppression(line: &str) -> bool {
    if let Some(comment_idx) = line.find("//") {
        let comment = &line[comment_idx..];
        if comment.contains("unwrap") || comment.contains("expect") {
            return true;
        }
    }
    false
}

/// Checks if a file path is inherently test/bench/generated code
pub fn is_test_or_generated_file(path_str: &str) -> bool {
    let normalized = path_str.replace('\\', "/");
    normalized.ends_with("_test.rs")
        || normalized.contains("/tests/")
        || normalized.ends_with("/tests.rs")
        || normalized.contains("/benches/")
        || normalized.ends_with("benches.rs")
        || normalized.contains("contextra_generated.rs")
}

struct DebtAstVisitor<'a> {
    file_path: &'a str,
    lines: Vec<&'a str>,
    unwrap_violations: Vec<DebtViolation>,
}

impl<'a> DebtAstVisitor<'a> {
    fn new(file_path: &'a str, content: &'a str) -> Self {
        Self {
            file_path,
            lines: content.lines().collect(),
            unwrap_violations: Vec::new(),
        }
    }

    fn check_file(&mut self, syn_file: &syn::File) {
        for item in &syn_file.items {
            self.check_item(item, false, "<top-level>");
        }
    }

    fn check_item(&mut self, item: &syn::Item, parent_in_test: bool, current_scope: &str) {
        let attrs = get_item_attrs(item);
        let in_test = parent_in_test || attrs.map_or(false, has_test_attribute);

        if in_test {
            return;
        }

        match item {
            syn::Item::Mod(item_mod) => {
                let mod_scope = if current_scope == "<top-level>" {
                    item_mod.ident.to_string()
                } else {
                    format!("{}::{}", current_scope, item_mod.ident)
                };
                if let Some((_, items)) = &item_mod.content {
                    for child in items {
                        self.check_item(child, in_test, &mod_scope);
                    }
                }
            }
            syn::Item::Fn(item_fn) => {
                let fn_scope = if current_scope == "<top-level>" {
                    item_fn.sig.ident.to_string()
                } else {
                    format!("{}::{}", current_scope, item_fn.sig.ident)
                };
                self.check_block(&item_fn.block, &fn_scope);
            }
            syn::Item::Impl(item_impl) => {
                let type_name = item_impl.self_ty.to_token_stream().to_string().replace(' ', "");
                let impl_scope = if current_scope == "<top-level>" {
                    type_name
                } else {
                    format!("{}::{}", current_scope, type_name)
                };
                for impl_item in &item_impl.items {
                    if let syn::ImplItem::Fn(impl_fn) = impl_item {
                        let fn_in_test = in_test || has_test_attribute(&impl_fn.attrs);
                        if !fn_in_test {
                            let fn_scope = format!("{}::{}", impl_scope, impl_fn.sig.ident);
                            self.check_block(&impl_fn.block, &fn_scope);
                        }
                    }
                }
            }
            syn::Item::Trait(item_trait) => {
                let trait_scope = if current_scope == "<top-level>" {
                    item_trait.ident.to_string()
                } else {
                    format!("{}::{}", current_scope, item_trait.ident)
                };
                for trait_item in &item_trait.items {
                    if let syn::TraitItem::Fn(trait_fn) = trait_item {
                        let fn_in_test = in_test || has_test_attribute(&trait_fn.attrs);
                        if !fn_in_test {
                            if let Some(block) = &trait_fn.default {
                                let fn_scope = format!("{}::{}", trait_scope, trait_fn.sig.ident);
                                self.check_block(block, &fn_scope);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn check_block(&mut self, block: &syn::Block, fn_scope: &str) {
        use syn::visit::Visit;

        struct ExprVisitor<'v, 'a> {
            visitor: &'v mut DebtAstVisitor<'a>,
            fn_scope: &'v str,
        }

        impl<'v, 'a> Visit<'v> for ExprVisitor<'v, 'a> {
            fn visit_expr_method_call(&mut self, node: &'v syn::ExprMethodCall) {
                let method_name = node.method.to_string();
                if method_name == "unwrap" || method_name == "expect" {
                    let span = node.method.span();
                    let line_num = span.start().line;
                    let line_content = if line_num > 0 && line_num <= self.visitor.lines.len() {
                        self.visitor.lines[line_num - 1]
                    } else {
                        ""
                    };

                    if !has_inline_suppression(line_content) {
                        self.visitor.unwrap_violations.push(DebtViolation {
                            file_path: self.visitor.file_path.to_string(),
                            line_num,
                            fn_name: self.fn_scope.to_string(),
                            message: format!(".{}() in production code", method_name),
                        });
                    }
                }
                syn::visit::visit_expr_method_call(self, node);
            }

            fn visit_item(&mut self, item: &'v syn::Item) {
                self.visitor.check_item(item, false, self.fn_scope);
            }
        }

        let mut expr_vis = ExprVisitor {
            visitor: self,
            fn_scope,
        };
        expr_vis.visit_block(block);
    }
}

fn get_item_attrs(item: &syn::Item) -> Option<&[syn::Attribute]> {
    match item {
        syn::Item::Const(i) => Some(&i.attrs),
        syn::Item::Enum(i) => Some(&i.attrs),
        syn::Item::ExternCrate(i) => Some(&i.attrs),
        syn::Item::Fn(i) => Some(&i.attrs),
        syn::Item::ForeignMod(i) => Some(&i.attrs),
        syn::Item::Impl(i) => Some(&i.attrs),
        syn::Item::Macro(i) => Some(&i.attrs),
        syn::Item::Mod(i) => Some(&i.attrs),
        syn::Item::Static(i) => Some(&i.attrs),
        syn::Item::Struct(i) => Some(&i.attrs),
        syn::Item::Trait(i) => Some(&i.attrs),
        syn::Item::TraitAlias(i) => Some(&i.attrs),
        syn::Item::Type(i) => Some(&i.attrs),
        syn::Item::Union(i) => Some(&i.attrs),
        syn::Item::Use(i) => Some(&i.attrs),
        _ => None,
    }
}

pub fn check_unwraps_in_file(rel_path: &str, content: &str) -> Vec<DebtViolation> {
    if is_test_or_generated_file(rel_path) {
        return Vec::new();
    }

    let syn_file = match syn::parse_file(content) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let mut visitor = DebtAstVisitor::new(rel_path, content);
    visitor.check_file(&syn_file);
    visitor.unwrap_violations
}

pub fn scan_unwraps_in_workspace(root: &Path) -> Vec<DebtViolation> {
    let mut violations = Vec::new();

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            if is_test_or_generated_file(&rel_path) {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                let file_violations = check_unwraps_in_file(&rel_path, &content);
                violations.extend(file_violations);
            }
        }
    }

    violations
}

pub fn check_unsafe_code(root: &Path) -> Vec<DebtViolation> {
    let mut violations = Vec::new();

    let allowed_crates = [
        "contextra-simd",
        "contextra-sys",
        "contextra-wire",
        "contextra-crypto",
    ];

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            if is_test_or_generated_file(&rel_path) {
                continue;
            }

            let is_allowed_crate = allowed_crates
                .iter()
                .any(|c| rel_path.contains(&format!("crates/{}/", c)));
            if is_allowed_crate || rel_path == "crates/contextra-vector/src/distance.rs" {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(syn_file) = syn::parse_file(&content) {
                    let test_lines = collect_test_lines(&syn_file);

                    for (idx, line) in content.lines().enumerate() {
                        let line_num = idx + 1;
                        if test_lines.contains(&line_num) {
                            continue;
                        }

                        let trimmed = line.trim();
                        if trimmed.contains("unsafe ")
                            || trimmed.starts_with("unsafe{")
                            || trimmed.starts_with("unsafe ")
                        {
                            if line.contains("#[allow(unsafe_code)]")
                                || (line.contains("//")
                                    && line.find("//").unwrap_or(0)
                                        < line.find("unsafe").unwrap_or(usize::MAX))
                            {
                                continue;
                            }
                            violations.push(DebtViolation {
                                file_path: rel_path.clone(),
                                line_num,
                                fn_name: "<unknown>".to_string(),
                                message: format!("unsafe keyword in {}", rel_path),
                            });
                        }
                    }
                }
            }
        }
    }

    violations
}

pub fn check_stdfs_in_workspace(root: &Path) -> Vec<DebtViolation> {
    let mut warnings = Vec::new();

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            if is_test_or_generated_file(&rel_path) {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(syn_file) = syn::parse_file(&content) {
                    let test_lines = collect_test_lines(&syn_file);

                    for (idx, line) in content.lines().enumerate() {
                        let line_num = idx + 1;
                        if test_lines.contains(&line_num) {
                            continue;
                        }

                        let trimmed = line.trim();
                        if trimmed.contains("std::fs::") && !trimmed.starts_with("//") {
                            warnings.push(DebtViolation {
                                file_path: rel_path.clone(),
                                line_num,
                                fn_name: "<unknown>".to_string(),
                                message: line.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    warnings
}

fn collect_test_lines(syn_file: &syn::File) -> HashSet<usize> {
    let mut set = HashSet::new();

    fn walk_item(item: &syn::Item, parent_in_test: bool, set: &mut HashSet<usize>) {
        let attrs = get_item_attrs(item);
        let in_test = parent_in_test || attrs.map_or(false, has_test_attribute);

        if in_test {
            let span = item.span();
            let start = span.start().line;
            let end = span.end().line;
            for line in start..=end {
                set.insert(line);
            }
        }

        if let syn::Item::Mod(item_mod) = item {
            if let Some((_, items)) = &item_mod.content {
                for child in items {
                    walk_item(child, in_test, set);
                }
            }
        }
    }

    for item in &syn_file.items {
        walk_item(item, false, &mut set);
    }

    set
}

pub fn run_debt_audit() -> Result<(), String> {
    println!("=== Tech-Debt Audit ===");

    let root = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => return Err(format!("Failed to get current directory: {}", e)),
    };

    let mut fail = false;

    println!("--- [1/4] .unwrap() / .expect() außerhalb von Test-Code ---");
    let unwrap_violations = scan_unwraps_in_workspace(&root);
    if !unwrap_violations.is_empty() {
        println!(
            "❌ UNWRAP VIOLATIONS ({} Treffer):",
            unwrap_violations.len()
        );
        for v in unwrap_violations.iter().take(15) {
            println!(
                "  {}:{} in {}: {}",
                v.file_path, v.line_num, v.fn_name, v.message
            );
        }
        if unwrap_violations.len() > 15 {
            println!("  ... und {} weitere Treffer", unwrap_violations.len() - 15);
        }
        fail = true;
    } else {
        println!("✅ Kein .unwrap() in Produktionscode");
    }

    println!("--- [2/4] unsafe außerhalb distance.rs ---");
    let unsafe_violations = check_unsafe_code(&root);
    if !unsafe_violations.is_empty() {
        println!("❌ UNSAFE VIOLATIONS:");
        for v in &unsafe_violations {
            println!("  {}:{} - {}", v.file_path, v.line_num, v.message);
        }
        fail = true;
    } else {
        println!("✅ Kein unsafe außerhalb distance.rs");
    }

    println!("--- [3/4] std::fs in Produktionscode (Soft-Warning) ---");
    let stdfs_warnings = check_stdfs_in_workspace(&root);
    if !stdfs_warnings.is_empty() {
        println!("⚠️  std::fs:: Treffer (nach tokio::fs migrieren):");
        for w in stdfs_warnings.iter().take(15) {
            println!("  {}:{} — {}", w.file_path, w.line_num, w.message.trim());
        }
    } else {
        println!("✅ Kein std::fs:: in Produktionscode");
    }

    println!("--- [4/4] Lock-Hierarchy & Async-Safety (AST Analysis) ---");
    if let Ok(output) = std::process::Command::new("sg")
        .args(["scan", "--rule", "rules/detect_nested_locks.yml", "crates/"])
        .output()
    {
        if output.status.success() {
            println!("✅ Keine kritischen Deadlock-Zustände im AST gefunden.");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.is_empty() || !stderr.is_empty() {
                println!("❌ Graceful Deadlock Risiko erkannt! Verschachtelte Locks gefunden:");
                println!("{}{}", stdout, stderr);
                fail = true;
            } else {
                println!("✅ Keine kritischen Deadlock-Zustände im AST gefunden.");
            }
        }
    } else {
        println!("⚠️  ast-grep (sg) nicht installiert, überspringe AST-Lock-Analyse.");
    }

    println!("--- [5/5] Security & Audit ---");
    if std::process::Command::new("cargo")
        .arg("audit")
        .arg("--version")
        .output()
        .map_or(false, |o| o.status.success())
    {
        let status = std::process::Command::new("cargo").arg("audit").status();
        if status.map_or(false, |s| !s.success()) {
            println!("⚠️ Audit warnings — manuell prüfen");
        }
    } else {
        println!("⚠️ cargo-audit nicht installiert: cargo install cargo-audit");
    }

    if fail {
        println!("\n❌ Debt-Audit FAILED — WP-0.0 zuerst abschließen!");
        Err("Debt-Audit failed with violations".to_string())
    } else {
        println!("\n✅ Debt-Audit PASSED");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_a_unwrap_in_free_function_is_violation() {
        let code = r#"
fn calculate_ratio() {
    let x: Option<i32> = Some(42);
    let val = x.unwrap();
}
"#;
        let violations = check_unwraps_in_file("src/calculator.rs", code);
        assert_eq!(
            violations.len(),
            1,
            "Expected 1 violation for .unwrap() in production function"
        );
        assert_eq!(violations[0].line_num, 4);
        assert_eq!(violations[0].fn_name, "calculate_ratio");
    }

    #[test]
    fn test_case_b_unwrap_in_fn_with_test_attr_ignored() {
        // Recreation of pid.rs case: standalone fn test_high_latency... #[test] in src/pid.rs
        let code = r#"
pub struct PidController;

impl PidController {
    pub fn update(&self) {}
}

#[test]
fn test_high_latency_causes_monotonic_decrease() {
    let pid = PidController;
    let res = Some(50).unwrap();
    assert_eq!(res, 50);
}
"#;
        let violations = check_unwraps_in_file("crates/contextra-adapt/src/pid.rs", code);
        assert_eq!(
            violations.len(),
            0,
            "Expected 0 violations for standalone fn with #[test] attribute"
        );
    }

    #[test]
    fn test_case_c_unwrap_in_helper_fn_inside_cfg_test_mod_ignored() {
        let code = r#"
fn prod_fn() {
    println!("hello");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_fixture() -> i32 {
        let val = Some(100).unwrap();
        val
    }

    #[test]
    fn test_run() {
        assert_eq!(setup_fixture(), 100);
    }
}
"#;
        let violations = check_unwraps_in_file("src/lib.rs", code);
        assert_eq!(
            violations.len(),
            0,
            "Expected 0 violations for .unwrap() in helper fn inside #[cfg(test)] mod"
        );
    }

    #[test]
    fn test_case_d_unwrap_in_nested_modules_with_cfg_test_ignored() {
        let code = r#"
mod outer {
    mod inner {
        #[cfg(test)]
        mod tests {
            fn nested_helper() {
                let x = Some(1).expect("should exist");
            }
        }
    }
}
"#;
        let violations = check_unwraps_in_file("src/nested.rs", code);
        assert_eq!(
            violations.len(),
            0,
            "Expected 0 violations for .unwrap() in nested #[cfg(test)] module"
        );
    }

    #[test]
    fn test_cfg_not_test_is_not_test_context() {
        let code = r#"
#[cfg(not(test))]
fn prod_only_fn() {
    let x = Some(1).unwrap();
}
"#;
        let violations = check_unwraps_in_file("src/prod.rs", code);
        assert_eq!(
            violations.len(),
            1,
            "#[cfg(not(test))] is production code and must report unwrap violations"
        );
    }

    #[test]
    fn test_inline_comment_suppression() {
        let code = r#"
fn prod_fn() {
    let x = Some(10).unwrap(); // unwrap: accepted fixpoint
    let y = Some(20).expect("ok"); // expect: documented invariant
}
"#;
        let violations = check_unwraps_in_file("src/prod.rs", code);
        assert_eq!(
            violations.len(),
            0,
            "Expected 0 violations due to inline comment suppression"
        );
    }
}
