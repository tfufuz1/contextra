use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct Violation {
    pub file_path: String,
    pub line_num: usize,
    pub line_content: String,
}

pub fn run_check_nan_validation_in_hot_loop(root: &Path) -> Result<Vec<Violation>, String> {
    let mut violations = Vec::new();

    let is_nan_re = Regex::new(r"\.is_nan\(\)").unwrap();
    let hot_loop_context_re =
        Regex::new(r"\b(candidates|neighbor|connections|col_idx|pop\(\))\b").unwrap();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target" && name != ".git" && name != ".cargo" && name != "node_modules"
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            if rel_path.starts_with("xtask/") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                let lines: Vec<&str> = content.lines().collect();

                for (idx, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    if trimmed.starts_with("//")
                        || trimmed.contains("// NAN-CHECK-INSERT-VALIDATED")
                        || rel_path.ends_with("_test.rs")
                        || rel_path.contains("/tests/")
                    {
                        continue;
                    }

                    if is_nan_re.is_match(line) {
                        let start_idx = idx.saturating_sub(10);
                        let mut matches_hot_loop = false;

                        for ctx_idx in start_idx..idx {
                            if hot_loop_context_re.is_match(lines[ctx_idx]) {
                                matches_hot_loop = true;
                                break;
                            }
                        }

                        if matches_hot_loop {
                            violations.push(Violation {
                                file_path: rel_path.clone(),
                                line_num: idx + 1,
                                line_content: trimmed.to_string(),
                            });
                        }
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
    use tempfile::tempdir;

    #[test]
    fn test_detects_nan_in_hot_loop() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("distance.rs");
        fs::write(
            &file,
            r#"
fn traverse(candidates: &mut Vec<usize>) {
    while let Some(curr) = candidates.pop() {
        let dist = compute_dist(curr);
        if dist.is_nan() {
            panic!("nan");
        }
    }
}
"#,
        )
        .unwrap();

        let violations = run_check_nan_validation_in_hot_loop(dir.path()).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line_num, 5);
    }

    #[test]
    fn test_ignores_nan_check_insert_validated() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("distance.rs");
        fs::write(
            &file,
            r#"
fn traverse(candidates: &mut Vec<usize>) {
    while let Some(curr) = candidates.pop() {
        let dist = compute_dist(curr);
        if dist.is_nan() { // NAN-CHECK-INSERT-VALIDATED
            panic!("nan");
        }
    }
}
"#,
        )
        .unwrap();

        let violations = run_check_nan_validation_in_hot_loop(dir.path()).unwrap();
        assert_eq!(violations.len(), 0);
    }
}
