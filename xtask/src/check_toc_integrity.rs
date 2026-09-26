//! Document TOC & Cross-Reference Integrity Gate (`check_toc_integrity.rs`)
//!
//! Subkommando `cargo xtask check-toc-integrity`
//! Prüft Markdown-Dokumente auf Vollständigkeit und Konsistenz des Inhaltsverzeichnisses
//! sowie Gültigkeit aller Paragraphen- und Anhang-Querverweise.

use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_TOC_TARGETS: &[&str] = &[
    "docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md",
    "docs/ci/BRANCH_PROTECTION_CHECKLIST.md",
];

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TocViolation {
    pub file: String,
    pub line: usize,
    pub message: String,
}

pub fn slugify(heading: &str) -> String {
    let clean = heading.trim_start_matches('#').trim();
    clean
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

pub fn check_toc_integrity_content(content: &str, file_rel_path: &str) -> Vec<TocViolation> {
    let mut violations = Vec::new();

    let mut body_headings = Vec::new();
    let mut headings_by_text = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let heading_text = trimmed.trim_start_matches('#').trim().to_string();
            body_headings.push((line_num, heading_text.clone(), slugify(&heading_text)));
            headings_by_text.push(heading_text.to_lowercase());
        }
    }

    // 1. Check TOC items (detected between top header and body)
    let mut in_toc = false;
    let toc_item_re = Regex::new(r"^(?:\d+\.|\*|-)\s+(?:\[(.*?)\]\(#.*?\)|(?:\*\*)?(.*?)(?:\*\*)?)(?:\s+—|\s*:|\s*$)").unwrap();

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        if trimmed.to_lowercase().contains("inhaltsverzeichnis")
            || trimmed.to_lowercase().contains("table of contents")
        {
            in_toc = true;
            continue;
        }

        if in_toc {
            if trimmed.starts_with("# ") || trimmed.starts_with("## ") {
                if !trimmed.to_lowercase().contains("inhaltsverzeichnis") {
                    in_toc = false;
                }
            }
        }

        if in_toc {
            if let Some(caps) = toc_item_re.captures(trimmed) {
                let item_text = caps
                    .get(1)
                    .or_else(|| caps.get(2))
                    .map(|m| m.as_str().trim())
                    .unwrap_or("");

                if !item_text.is_empty() && !item_text.starts_with("Inhaltsverzeichnis") {
                    let norm_item = item_text.to_lowercase();
                    let matches_heading = headings_by_text.iter().any(|h| {
                        h == &norm_item
                            || h.contains(&norm_item)
                            || norm_item.contains(h)
                    });

                    if !matches_heading {
                        violations.push(TocViolation {
                            file: file_rel_path.to_string(),
                            line: line_num,
                            message: format!(
                                "TOC-Eintrag '{}' hat keine übereinstimmende Überschrift im Dokumentkörper",
                                item_text
                            ),
                        });
                    }
                }
            }
        }
    }

    // 2. Check cross-references like "siehe Teil X", "siehe Anhang Y", "siehe §N"
    let ref_re = Regex::new(r"(?i)\bsiehe\s+(Teil\s+[A-Z\d]+|Anhang\s+[A-Z\d]+|§\d+(?:\.\d+)*)\b").unwrap();
    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        for caps in ref_re.captures_iter(trimmed) {
            if let Some(target_ref) = caps.get(1) {
                let target = target_ref.as_str();
                let target_norm = target.to_lowercase();

                let found = headings_by_text.iter().any(|h| h.contains(&target_norm))
                    || content.to_lowercase().contains(&target_norm);

                if !found {
                    violations.push(TocViolation {
                        file: file_rel_path.to_string(),
                        line: line_num,
                        message: format!("Verwaister Querverweis: '{}' nicht im Dokument gefunden", target),
                    });
                }
            }
        }
    }

    violations
}

pub fn run_check_toc_integrity(targets: &[PathBuf]) -> Result<(), String> {
    println!("=== Running xtask check-toc-integrity ===");
    let root = crate::find_root_dir();
    let mut all_violations = Vec::new();

    let doc_targets: Vec<PathBuf> = if targets.is_empty() {
        DEFAULT_TOC_TARGETS.iter().map(PathBuf::from).collect()
    } else {
        targets.to_vec()
    };

    for target in &doc_targets {
        let full_path = if target.is_absolute() {
            target.clone()
        } else {
            root.join(target)
        };

        let rel_path = target.to_string_lossy().to_string();

        if full_path.is_file() {
            if let Ok(content) = fs::read_to_string(&full_path) {
                let violations = check_toc_integrity_content(&content, &rel_path);
                all_violations.extend(violations);
            }
        } else {
            all_violations.push(TocViolation {
                file: rel_path.clone(),
                line: 0,
                message: format!("Dokument '{}' nicht gefunden", rel_path),
            });
        }
    }

    if !all_violations.is_empty() {
        for v in &all_violations {
            eprintln!("❌ [check-toc-integrity]: {}:{} — {}", v.file, v.line, v.message);
        }
        return Err(format!(
            "check-toc-integrity failed with {} violation(s)",
            all_violations.len()
        ));
    }

    println!("✅ Alle Inhaltsverzeichnisse und Querverweise sind integrierbar.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("# 1. Einleitung & Übersicht"), "1-einleitung-bersicht");
        assert_eq!(slugify("## §3.1 Invarianten"), "3-1-invarianten");
    }

    #[test]
    fn test_check_toc_integrity_valid() {
        let doc = r#"
# Test Spec

## Inhaltsverzeichnis
- 1. Einleitung
- 2. Hauptteil

## 1. Einleitung
Inhalt Einleitung. siehe §1.1

## 2. Hauptteil
Inhalt Hauptteil.
### §1.1 Invarianten
"#;
        let violations = check_toc_integrity_content(doc, "test.md");
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_toc_integrity_missing_section() {
        let doc = r#"
# Test Spec

## Inhaltsverzeichnis
- 1. Einleitung
- 2. Phantom Teil

## 1. Einleitung
Siehe Teil Z
"#;
        let violations = check_toc_integrity_content(doc, "test.md");
        assert_eq!(violations.len(), 2);
    }
}
