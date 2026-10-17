//! Ratchet check for hardcoded evidence literals in deletion witness code.

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn run_erasure_evidence_literals(_args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let src_dir = root.join("crates");

    let count = count_hardcoded_evidence_literals(&src_dir);
    println!("Evidence literal count: {} (Target: 0)", count);

    if count > 0 {
        eprintln!("❌ erasure-evidence-literals gate failed: found {} hardcoded evidence literals in witness code", count);
        1
    } else {
        println!("✅ erasure-evidence-literals gate passed");
        0
    }
}

pub fn count_hardcoded_evidence_literals(dir: &Path) -> usize {
    let mut count = 0;
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let path_str = path.to_string_lossy();
            if path_str.contains("witness") || path_str.contains("deletion") || path_str.contains("erasure") {
                if let Ok(content) = fs::read_to_string(path) {
                    count += scan_content_for_literals(&content);
                }
            }
        }
    }
    count
}

pub fn scan_content_for_literals(content: &str) -> usize {
    let mut hits = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }
        if trimmed.contains("HARDCODED_EVIDENCE_LITERAL_MOCK") || trimmed.contains("mock_evidence_witness_hex") {
            hits += 1;
        }
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_literals_negative() {
        let mock_code = r#"
pub fn issue_witness() -> Witness {
    let mock = "HARDCODED_EVIDENCE_LITERAL_MOCK";
}
"#;
        assert_eq!(scan_content_for_literals(mock_code), 1);
    }

    #[test]
    fn test_evidence_literals_positive() {
        let clean_code = r#"
pub fn issue_witness(ev: &Evidence) -> Witness {
    Witness::from_evidence(ev)
}
"#;
        assert_eq!(scan_content_for_literals(clean_code), 0);
    }
}
