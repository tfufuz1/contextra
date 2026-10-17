//! Validates doc-truth for deletion documentation tables in docs/deletion/*.md.

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn run_doc_truth_deletion(_args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let deletion_docs_dir = root.join("docs/deletion");

    if !deletion_docs_dir.exists() {
        println!("✅ doc-truth-deletion gate passed (docs/deletion/ directory not present)");
        return 0;
    }

    let violations = scan_doc_truth_deletion(&deletion_docs_dir);
    if !violations.is_empty() {
        eprintln!("❌ doc-truth-deletion gate failed with {} violations:", violations.len());
        for v in &violations {
            eprintln!("  {}", v);
        }
        1
    } else {
        println!("✅ doc-truth-deletion gate passed");
        0
    }
}

pub fn scan_doc_truth_deletion(dir: &Path) -> Vec<String> {
    let mut violations = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let path_str = path.to_string_lossy();
            if let Ok(content) = fs::read_to_string(path) {
                let file_violations = scan_doc_markdown(&path_str, &content);
                violations.extend(file_violations);
            }
        }
    }

    violations
}

pub fn scan_doc_markdown(file_path: &str, content: &str) -> Vec<String> {
    let mut violations = Vec::new();
    let mut in_table = false;

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            if trimmed.contains("---|") || trimmed.contains(":---|") || trimmed.contains("---:") {
                in_table = true;
                continue;
            }

            if in_table {
                // Table row
                let lower = trimmed.to_lowercase();
                let has_anchor = trimmed.contains('`') || trimmed.contains("crates/") || trimmed.contains("src/") || trimmed.contains("docs/");
                let is_planned = lower.contains("geplant")
                    || lower.contains("planned")
                    || lower.contains("nicht prüfbar")
                    || lower.contains("n/a")
                    || lower.contains("ausstehend");

                if !has_anchor && !is_planned {
                    violations.push(format!("{}:{}: Table row missing code anchor or planned status: {}", file_path, line_idx + 1, trimmed));
                }
            }
        } else {
            in_table = false;
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_truth_deletion_negative() {
        let doc = r#"
| Feature | Implementation | Notes |
|---|---|---|
| Unanchored Purge | Missing code reference | Active component |
"#;
        let v = scan_doc_markdown("docs/deletion/test.md", doc);
        assert!(!v.is_empty());
    }

    #[test]
    fn test_doc_truth_deletion_positive() {
        let doc = r#"
| Feature | Implementation | Notes |
|---|---|---|
| Purge Engine | `crates/contextra-store/src/purge.rs` | Active |
| Future Feature | Geplant | Planned for v2 |
"#;
        let v = scan_doc_markdown("docs/deletion/test.md", doc);
        assert!(v.is_empty());
    }
}
