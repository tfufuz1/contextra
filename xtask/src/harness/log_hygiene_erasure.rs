//! Checks log hygiene on erasure paths (no unredacted keys or raw contents in tracing macros).

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn run_log_hygiene_erasure(_args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let crates_dir = root.join("crates");

    let violations = scan_log_hygiene_erasure(&crates_dir);
    if !violations.is_empty() {
        eprintln!("❌ log-hygiene-erasure gate failed with {} violations:", violations.len());
        for v in &violations {
            eprintln!("  {}", v);
        }
        1
    } else {
        println!("✅ log-hygiene-erasure gate passed");
        0
    }
}

pub fn scan_log_hygiene_erasure(dir: &Path) -> Vec<String> {
    let mut violations = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let path_str = path.to_string_lossy();
            if path_str.contains("forget") || path_str.contains("purge") || path_str.contains("erasure") || path_str.contains("deletion") {
                if let Ok(content) = fs::read_to_string(path) {
                    let file_violations = scan_file_log_hygiene(&path_str, &content);
                    violations.extend(file_violations);
                }
            }
        }
    }

    violations
}

pub fn scan_file_log_hygiene(file_path: &str, content: &str) -> Vec<String> {
    let mut violations = Vec::new();
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }

        if trimmed.contains("tracing::") || trimmed.contains("info!(") || trimmed.contains("debug!(") || trimmed.contains("warn!(") || trimmed.contains("error!(") || trimmed.contains("trace!(") {
            if trimmed.contains("raw_key") || trimmed.contains("raw_content") || trimmed.contains("secret_key") || trimmed.contains("UNREDACTED_ERASURE_KEY") {
                violations.push(format!("{}:{}: Potential sensitive field in tracing: {}", file_path, line_idx + 1, trimmed));
            }
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_hygiene_negative() {
        let bad_code = r#"
tracing::info!(raw_key = ?key, "Purging key");
"#;
        let v = scan_file_log_hygiene("test.rs", bad_code);
        assert!(!v.is_empty());
    }

    #[test]
    fn test_log_hygiene_positive() {
        let good_code = r#"
tracing::info!(key_hash = %hash, "Purged key safely");
"#;
        let v = scan_file_log_hygiene("test.rs", good_code);
        assert!(v.is_empty());
    }
}
