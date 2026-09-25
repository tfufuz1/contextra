// Contextra — Check Workspace Crate References Gate
//
// Subkommando `cargo xtask check-crate-references`
// Durchsucht `*.md`-, `*.toml`-Dateien und `justfile` (ausgenommen `docs/decisions/` und
// `docs/GESAMTSPEZIFIKATION.md`) nach Crate-Referenzen des Musters `contextra-[a-z0-9-]+` und
// prüft, ob die Treffer aktive Workspace-Member aus `cargo metadata` sind.

use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CrateRefViolation {
    pub file: String,
    pub line: usize,
    pub crate_name: String,
}

#[allow(dead_code)]
pub fn get_active_workspace_members(root: &Path) -> Result<HashSet<String>, String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("Failed to execute cargo metadata: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("cargo metadata command failed: {}", stderr));
    }

    let json_val: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse cargo metadata JSON: {}", e))?;

    let packages = json_val
        .get("packages")
        .and_then(|p| p.as_array())
        .ok_or_else(|| "Missing 'packages' array in cargo metadata output".to_string())?;

    let workspace_members: HashSet<String> = packages
        .iter()
        .filter_map(|pkg| {
            pkg.get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string())
        })
        .collect();

    Ok(workspace_members)
}

#[allow(dead_code)]
pub fn extract_crate_references(line: &str) -> Vec<String> {
    if line.contains("<!-- crate-ref-ignore -->") {
        return Vec::new();
    }

    let re = match Regex::new(r"contextra-[a-z0-9-]+") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut refs: Vec<String> = re
        .find_iter(line)
        .map(|m| {
            let mut s = m.as_str();
            // Trim trailing punctuation if any (e.g., period, comma, colon, slash)
            s = s.trim_end_matches(|c: char| {
                c == '.' || c == ',' || c == ':' || c == '/' || c == '\'' || c == '"' || c == '`'
            });
            s.to_string()
        })
        .collect();

    refs.sort();
    refs.dedup();
    refs
}

/// Überprüft, ob eine Datei auf Crate-Referenzen gescannt werden soll.
///
/// `docs/archive/` wird bewusst ausgeschlossen, da es eingefrorene historische
/// Snapshots enthält, die absichtlich veraltete/entfernte Crate-Namen nennen.
#[allow(dead_code)]
pub fn should_check_file(rel_path_str: &str) -> bool {
    // Normalisiere Pfad-Trennzeichen
    let normalized = rel_path_str.replace('\\', "/");
    let clean = normalized.trim_start_matches("./");

    // Ausnahmen
    if clean.starts_with("docs/decisions/")
        || clean.starts_with("docs/archive/")
        || clean == "docs/GESAMTSPEZIFIKATION.md"
    {
        return false;
    }

    if clean.starts_with("target/")
        || clean.starts_with(".git/")
        || clean.starts_with("node_modules/")
    {
        return false;
    }

    if clean == "justfile" {
        return true;
    }

    clean.ends_with(".md") || clean.ends_with(".toml")
}

#[allow(dead_code)]
pub fn check_crate_references_in_content(
    content: &str,
    doc_rel_path: &str,
    active_members: &HashSet<String>,
) -> Vec<CrateRefViolation> {
    if !should_check_file(doc_rel_path) {
        return Vec::new();
    }

    let mut violations = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let c_refs = extract_crate_references(line);

        for c_ref in c_refs {
            if !active_members.contains(&c_ref) {
                violations.push(CrateRefViolation {
                    file: doc_rel_path.to_string(),
                    line: line_num,
                    crate_name: c_ref,
                });
            }
        }
    }

    violations
}

#[allow(dead_code)]
pub fn scan_and_check_workspace(root: &Path) -> Result<Vec<CrateRefViolation>, String> {
    let active_members = get_active_workspace_members(root)?;
    let mut violations = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let rel = e.path().strip_prefix(root).unwrap_or_else(|_| e.path());
            let rel_str = rel.to_string_lossy();
            let norm = rel_str.replace('\\', "/");
            let clean = norm.trim_start_matches("./");

            if clean == "target" || clean == ".git" || clean == "node_modules" {
                return false;
            }
            if clean == "docs/decisions" || clean == "docs/archive" {
                return false;
            }
            true
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let rel_path = match path.strip_prefix(root) {
            Ok(p) => p,
            Err(_) => path,
        };

        let rel_str = rel_path.to_string_lossy().to_string();

        if should_check_file(&rel_str) {
            if let Ok(content) = fs::read_to_string(path) {
                let file_violations =
                    check_crate_references_in_content(&content, &rel_str, &active_members);
                violations.extend(file_violations);
            }
        }
    }

    Ok(violations)
}

#[allow(dead_code)]
pub fn run_check_crate_references() -> Result<(), String> {
    println!("=== Running xtask check-crate-references ===");
    let root = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            return Err(format!(
                "Failed to determine current working directory: {}",
                e
            ))
        }
    };

    let violations = scan_and_check_workspace(&root)?;

    if !violations.is_empty() {
        for v in &violations {
            eprintln!(
                "❌ [check-crate-references]: {}:{} — verwaiste/ungültige Crate-Referenz: '{}'",
                v.file, v.line, v.crate_name
            );
        }
        return Err(format!(
            "check-crate-references failed with {} stale/unknown crate reference(s)",
            violations.len()
        ));
    }

    println!(
        "✅ Alle Crate-Referenzen (*.md, *.toml, justfile) verweisen auf aktive Workspace-Member."
    );
    Ok(())
}

#[allow(dead_code)]
pub fn run() -> Result<(), String> {
    run_check_crate_references()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_crate_references() {
        let line = "Referenz auf `contextra-core` und `contextra-store` sowie `contextra-nonexistent-123`.";
        let refs = extract_crate_references(line);
        assert_eq!(
            refs,
            vec!["contextra-core", "contextra-nonexistent-123", "contextra-store"]
        );
    }

    #[test]
    fn test_extract_crate_references_with_ignore_tag() {
        let line = "Prüfe `contextra-deleted-crate` <!-- crate-ref-ignore -->";
        let refs = extract_crate_references(line);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_should_check_file_inclusions_and_exclusions() {
        assert!(should_check_file("README.md"));
        assert!(should_check_file("Cargo.toml"));
        assert!(should_check_file("justfile"));
        assert!(should_check_file("docs/ARCHITECTURE.md"));
        assert!(should_check_file("crates/contextra-core/Cargo.toml"));

        // Exclusions
        assert!(!should_check_file("docs/decisions/ADR-001.md"));
        assert!(!should_check_file("docs/GESAMTSPEZIFIKATION.md"));
        assert!(!should_check_file(
            "docs/archive/misc/audits/AUDIT_contextra-core_2026-09-13.md"
        ));
        assert!(!should_check_file("docs/archive/GESAMTSPEZIFIKATION.md"));
        assert!(!should_check_file("target/debug/build.rs"));
        assert!(!should_check_file("src/main.rs"));
    }

    #[test]
    fn test_check_crate_references_in_content() {
        let mut active = HashSet::new();
        active.insert("contextra-core".to_string());
        active.insert("contextra-store".to_string());

        let content = "Gültig: `contextra-core`\nUngültig: `contextra-old-crate`\n";
        let violations = check_crate_references_in_content(content, "test.md", &active);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].file, "test.md");
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[0].crate_name, "contextra-old-crate");
    }

    #[test]
    fn test_check_crate_references_in_content_archive_exclusion() {
        let mut active = HashSet::new();
        active.insert("contextra-core".to_string());

        let content = "Historischer Audit-Bericht: `contextra-security` und `contextra-index`.\n";
        let doc_path = "docs/archive/misc/audits/AUDIT_contextra-core_2026-09-13.md";
        let violations = check_crate_references_in_content(content, doc_path, &active);

        assert_eq!(violations.len(), 0);
    }
}
