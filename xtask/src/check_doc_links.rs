// Contextra — Documentation Link & Absolute URL Gate (G-33)
// Subkommando `cargo xtask check-doc-links`
// Validiert Markdown-Dateien im Repository auf:
// 1. Keine absoluten `file:///` URLs
// 2. Existenz aller internen Markdown-Linkziele in lebenden Dokumenten
// Ausnahme-Marker: `<!-- doc-ref-ignore -->`

use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DocLinkViolation {
    pub file: String,
    pub line: usize,
    pub target: String,
    pub reason: String,
}

pub fn extract_markdown_links(line: &str) -> Vec<String> {
    if line.contains("<!-- doc-ref-ignore -->") {
        return Vec::new();
    }

    let mut links = Vec::new();

    // Inline links: [text](target)
    let re_inline = Regex::new(r"\[[^\]]*\]\(([^)]+)\)").unwrap();
    for cap in re_inline.captures_iter(line) {
        if let Some(m) = cap.get(1) {
            let target = m.as_str().trim();
            let link_target = if let Some(idx) = target.find(' ') {
                &target[..idx]
            } else {
                target
            };
            links.push(link_target.to_string());
        }
    }

    // Reference links: [text]: target
    let re_ref = Regex::new(r"^\s*\[[^\]]+\]:\s*(\S+)").unwrap();
    for cap in re_ref.captures_iter(line) {
        if let Some(m) = cap.get(1) {
            links.push(m.as_str().trim().to_string());
        }
    }

    links
}

pub fn is_external_or_anchor(target: &str) -> bool {
    let t = target.trim();
    t.starts_with("http://")
        || t.starts_with("https://")
        || t.starts_with("mailto:")
        || t.starts_with("ftp://")
        || t.starts_with('#')
        || t.contains("::")
        || t.starts_with("TS:")
        || t.is_empty()
}

pub fn strip_anchor(target: &str) -> &str {
    if let Some(idx) = target.find('#') {
        &target[..idx]
    } else {
        target
    }
}

pub fn resolve_link_target(file_dir: &Path, repo_root: &Path, raw_target: &str) -> Option<PathBuf> {
    let target = strip_anchor(raw_target);
    if target.is_empty() {
        return None;
    }

    let path_obj = Path::new(target);

    if target.starts_with("docs/") || target.starts_with(".jules/") || target.starts_with("crates/") {
        let p = repo_root.join(target);
        if p.exists() {
            return Some(p);
        }
        let in_archive = repo_root.join("docs/archive").join(target.strip_prefix("docs/").unwrap_or(target));
        if in_archive.exists() {
            return Some(in_archive);
        }
        let in_archive_spec = repo_root.join("docs/archive/spec").join(target.strip_prefix("docs/").unwrap_or(target));
        if in_archive_spec.exists() {
            return Some(in_archive_spec);
        }
        Some(p)
    } else if path_obj.is_absolute() {
        let rel = target.trim_start_matches('/');
        Some(repo_root.join(rel))
    } else {
        let in_dir = file_dir.join(target);
        if in_dir.exists() {
            Some(in_dir)
        } else {
            let in_root = repo_root.join(target);
            if in_root.exists() {
                Some(in_root)
            } else {
                let in_archive = repo_root.join("docs/archive/spec").join(target);
                if in_archive.exists() {
                    Some(in_archive)
                } else {
                    let in_14_10_26 = repo_root.join("docs/archive/14-10-26").join(target);
                    if in_14_10_26.exists() {
                        Some(in_14_10_26)
                    } else {
                        Some(in_dir)
                    }
                }
            }
        }
    }
}

pub fn check_file_doc_links(filepath: &Path, repo_root: &Path) -> Vec<DocLinkViolation> {
    let mut violations = Vec::new();
    let rel_file = filepath
        .strip_prefix(repo_root)
        .unwrap_or(filepath)
        .to_string_lossy()
        .to_string();

    let content = match fs::read_to_string(filepath) {
        Ok(c) => c,
        Err(_) => return violations,
    };

    let file_dir = filepath.parent().unwrap_or(repo_root);
    let is_archived = filepath.starts_with(repo_root.join("docs/archive"));

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;

        // Rule 1: No file:/// or file:// URLs (applies to ALL docs including archive)
        if line.contains("file:///") || line.contains("file://") {
            if !line.contains("<!-- doc-ref-ignore -->") {
                violations.push(DocLinkViolation {
                    file: rel_file.clone(),
                    line: line_num,
                    target: "file://".to_string(),
                    reason: "Absolute file:// URL is forbidden (G-33)".to_string(),
                });
            }
        }

        // Rule 2: Link targets must exist for living documentation
        if !is_archived {
            let links = extract_markdown_links(line);
            for link in links {
                if is_external_or_anchor(&link) {
                    continue;
                }

                if let Some(resolved) = resolve_link_target(file_dir, repo_root, &link) {
                    if !resolved.exists() {
                        violations.push(DocLinkViolation {
                            file: rel_file.clone(),
                            line: line_num,
                            target: link.clone(),
                            reason: format!(
                                "Target path does not exist: '{}' (resolved: '{}')",
                                link,
                                resolved.display()
                            ),
                        });
                    }
                }
            }
        }
    }

    violations
}

pub fn run_check_doc_links(_args: &[String]) -> i32 {
    println!("=== Running xtask check-doc-links (G-33) ===");
    let repo_root = crate::find_root_dir();
    let mut violations = Vec::new();

    let md_files = collect_markdown_files(&repo_root);
    println!("Checking {} Markdown file(s)...", md_files.len());

    for md_file in &md_files {
        let file_violations = check_file_doc_links(md_file, &repo_root);
        violations.extend(file_violations);
    }

    if !violations.is_empty() {
        eprintln!("\n❌ [check-doc-links]: Found {} link violation(s):", violations.len());
        for v in &violations {
            eprintln!("  - {}:{} — {} ({})", v.file, v.line, v.reason, v.target);
        }
        1
    } else {
        println!("✅ [check-doc-links]: All Markdown links are valid and zero file:/// URLs found.");
        0
    }
}

fn collect_markdown_files(repo_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    let readme = repo_root.join("README.md");
    if readme.is_file() {
        files.push(readme);
    }

    let docs_dir = repo_root.join("docs");
    if docs_dir.is_dir() {
        for entry in WalkDir::new(&docs_dir).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() && p.extension().is_some_and(|ext| ext == "md") {
                files.push(p.to_path_buf());
            }
        }
    }

    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_markdown_links() {
        let line = "See [Specification](../spec/CONTEXTRA_ENDPRODUKT_SPEZIFIKATION_v1_4.md) for details.";
        let links = extract_markdown_links(line);
        assert_eq!(links, vec!["../spec/CONTEXTRA_ENDPRODUKT_SPEZIFIKATION_v1_4.md"]);
    }

    #[test]
    fn test_doc_ref_ignore_comment() {
        let line = "Link to [invalid](nonexistent.md) <!-- doc-ref-ignore -->";
        let links = extract_markdown_links(line);
        assert!(links.is_empty());
    }

    #[test]
    fn test_external_and_anchor_filter() {
        assert!(is_external_or_anchor("https://example.com"));
        assert!(is_external_or_anchor("#section-1"));
        assert!(!is_external_or_anchor("../spec/doc.md"));
    }
}
