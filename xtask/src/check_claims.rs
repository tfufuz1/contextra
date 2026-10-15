//! Gate G-19: `check-claims`
//!
//! Scans documentation, README, CHANGELOG, and error text string literals in Rust code
//! for forbidden claims per Spec §5.14, F-28, and REG-16.

use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use syn::LitStr;

#[derive(Debug, Deserialize)]
struct ForbiddenRules {
    title: Option<String>,
    version: Option<String>,
    #[serde(default)]
    forbidden: Vec<ForbiddenRule>,
}

#[derive(Debug, Deserialize)]
struct ForbiddenRule {
    id: String,
    pattern: String,
    description: String,
}

#[derive(Debug, Clone)]
pub struct ClaimViolation {
    pub file_path: String,
    pub line_num: usize,
    pub rule_id: String,
    pub matched_text: String,
    pub description: String,
}

struct CompiledRule {
    id: String,
    regex: Regex,
    description: String,
}

struct StringLiteralVisitor<'a> {
    file_path: &'a str,
    rules: &'a [CompiledRule],
    violations: Vec<ClaimViolation>,
    content: &'a str,
}

impl<'a, 'ast> Visit<'ast> for StringLiteralVisitor<'a> {
    fn visit_attribute(&mut self, _attr: &'ast syn::Attribute) {
        // Skip attributes (including doc comments #[doc = "..."])
    }

    fn visit_lit_str(&mut self, lit: &'ast LitStr) {
        let value = lit.value();
        for rule in self.rules {
            if let Some(mat) = rule.regex.find(&value) {
                let span = lit.span();
                let line_num = span.start().line;

                self.violations.push(ClaimViolation {
                    file_path: self.file_path.to_string(),
                    line_num,
                    rule_id: rule.id.clone(),
                    matched_text: mat.as_str().to_string(),
                    description: rule.description.clone(),
                });
            }
        }
    }
}

pub fn run_check_claims(root_dir: &Path) -> Result<Vec<ClaimViolation>, String> {
    let rules_path = root_dir.join("docs/decisions/claims/claims-forbidden.toml");
    if !rules_path.exists() {
        return Err(format!(
            "Rules file not found at: {}",
            rules_path.display()
        ));
    }

    let rules_content = fs::read_to_string(&rules_path)
        .map_err(|e| format!("Failed to read rules file {}: {}", rules_path.display(), e))?;

    let parsed: ForbiddenRules = toml::from_str(&rules_content)
        .map_err(|e| format!("Failed to parse rules file {}: {}", rules_path.display(), e))?;

    let mut compiled_rules = Vec::new();
    for rule in parsed.forbidden {
        let regex = Regex::new(&rule.pattern).map_err(|e| {
            format!(
                "Invalid regex pattern in rule '{}' ({}): {}",
                rule.id, rule.pattern, e
            )
        })?;
        compiled_rules.push(CompiledRule {
            id: rule.id,
            regex,
            description: rule.description,
        });
    }

    let mut violations = Vec::new();

    // 1. Scan text files: README.md, CHANGELOG.md, docs/** (excluding docs/archive/**, docs/baseline/**, docs/decisions/ADR-E*, docs/decisions/claims/)
    let mut text_files = Vec::new();

    let readme = root_dir.join("README.md");
    if readme.exists() {
        text_files.push(readme);
    }
    let changelog = root_dir.join("CHANGELOG.md");
    if changelog.exists() {
        text_files.push(changelog);
    }

    let docs_dir = root_dir.join("docs");
    if docs_dir.exists() {
        for entry in walkdir::WalkDir::new(&docs_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                let rel = path
                    .strip_prefix(root_dir)
                    .unwrap_or(path)
                    .to_string_lossy();

                // Skip forbidden directories / prefixes
                if rel.starts_with("docs/archive/")
                    || rel.starts_with("docs/baseline/")
                    || rel.starts_with("docs/decisions/ADR-E")
                    || rel.starts_with("docs/decisions/claims/")
                {
                    continue;
                }

                if let Some(ext) = path.extension() {
                    if ext == "md" || ext == "txt" || ext == "toml" {
                        text_files.push(path.to_path_buf());
                    }
                }
            }
        }
    }

    for path in text_files {
        let rel_path = path
            .strip_prefix(root_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (line_idx, line) in content.lines().enumerate() {
            for rule in &compiled_rules {
                if let Some(mat) = rule.regex.find(line) {
                    violations.push(ClaimViolation {
                        file_path: rel_path.clone(),
                        line_num: line_idx + 1,
                        rule_id: rule.id.clone(),
                        matched_text: mat.as_str().to_string(),
                        description: rule.description.clone(),
                    });
                }
            }
        }
    }

    // 2. Scan Rust source files: crates/**/src/** (excluding crates/contextra-mcp/src/server_tools.rs)
    let crates_dir = root_dir.join("crates");
    if crates_dir.exists() {
        for entry in walkdir::WalkDir::new(&crates_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "rs") {
                    let rel_path = path
                        .strip_prefix(root_dir)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();

                    // Skip crates/contextra-mcp/src/server_tools.rs
                    if rel_path == "crates/contextra-mcp/src/server_tools.rs" {
                        continue;
                    }

                    // Only scan if inside src/ directory of a crate
                    if !rel_path.contains("/src/") {
                        continue;
                    }

                    let content = match fs::read_to_string(path) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    if let Ok(syntax) = syn::parse_file(&content) {
                        let mut visitor = StringLiteralVisitor {
                            file_path: &rel_path,
                            rules: &compiled_rules,
                            violations: Vec::new(),
                            content: &content,
                        };
                        visitor.visit_file(&syntax);
                        violations.extend(visitor.violations);
                    }
                }
            }
        }
    }

    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forbidden_claim_triggers_violation() {
        let rules_toml = r#"
[[forbidden]]
id = "unwiederbringlich-geloescht"
pattern = "(?i)unwiederbringlich\\s+gelöscht"
description = "Behauptung von unwiederbringlicher Löschung"
"#;
        let parsed: ForbiddenRules = toml::from_str(rules_toml).unwrap();
        let regex = Regex::new(&parsed.forbidden[0].pattern).unwrap();
        let rule = CompiledRule {
            id: parsed.forbidden[0].id.clone(),
            regex,
            description: parsed.forbidden[0].description.clone(),
        };

        let rust_code = r#"
fn bad_error() {
    let msg = "Daten wurden unwiederbringlich gelöscht!";
    println!("{}", msg);
}
"#;
        let syntax = syn::parse_file(rust_code).unwrap();
        let mut visitor = StringLiteralVisitor {
            file_path: "crates/dummy/src/lib.rs",
            rules: &[rule],
            violations: Vec::new(),
            content: rust_code,
        };
        visitor.visit_file(&syntax);

        assert_eq!(visitor.violations.len(), 1);
        assert_eq!(visitor.violations[0].rule_id, "unwiederbringlich-geloescht");
        assert_eq!(visitor.violations[0].file_path, "crates/dummy/src/lib.rs");
    }
}
