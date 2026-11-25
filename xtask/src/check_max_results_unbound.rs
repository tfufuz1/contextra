use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct Violation {
    pub file_path: String,
    pub line_num: usize,
    pub line_content: String,
}

#[allow(dead_code)]
pub fn run_check_max_results_unbound(root: &Path) -> Result<Vec<Violation>, String> {
    run_check_max_results_unbound_with_options(root, false)
}

pub fn run_check_max_results_unbound_with_options(
    root: &Path,
    include_tests: bool,
) -> Result<Vec<Violation>, String> {
    let mut violations = Vec::new();

    let max_val_re = Regex::new(r"\b(usize::MAX|u64::MAX|i64::MAX|u32::MAX)\b").unwrap();
    let ctx_param_re = Regex::new(
        r"\b(max_results|max_items|max_docs|max_entries|max_nodes|max_count|max_hits|max_candidates|max_neighbors|max_hops|max_depth|limit_[a_zA_Z0_9_]+|[a_zA_Z0_9_]+_limit|limit)\b",
    )
    .unwrap();

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

            if !include_tests && (rel_path.ends_with("_test.rs") || rel_path.contains("/tests/")) {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                let lines: Vec<&str> = content.lines().collect();

                for (idx, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    // Skip comment lines, lines with // UNBOUNDED-OK
                    if trimmed.starts_with("//") || trimmed.contains("// UNBOUNDED-OK") {
                        continue;
                    }

                    if max_val_re.is_match(line) {
                        let start_idx = idx.saturating_sub(3);
                        let end_idx = (idx + 3).min(lines.len().saturating_sub(1));

                        // Check if UNBOUNDED-OK is present within context window
                        let has_unbounded_ok =
                            (start_idx..=end_idx).any(|i| lines[i].contains("// UNBOUNDED-OK"));
                        if has_unbounded_ok {
                            continue;
                        }

                        let mut matches_ctx = false;
                        for ctx_idx in start_idx..=end_idx {
                            let ctx_line = lines[ctx_idx].trim();
                            // Skip comment lines when evaluating context matching
                            if ctx_line.starts_with("//") {
                                continue;
                            }
                            if ctx_param_re.is_match(ctx_line) {
                                matches_ctx = true;
                                break;
                            }
                        }

                        if matches_ctx {
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
    fn test_detects_unbounded_max_results() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("search.rs");
        fs::write(
            &file,
            r#"
fn search(max_results: usize) {
    let k = usize::MAX;
}
"#,
        )
        .unwrap();

        let violations = run_check_max_results_unbound(dir.path()).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line_num, 3);
    }

    #[test]
    fn test_ignores_unbounded_ok() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("search.rs");
        fs::write(
            &file,
            r#"
fn search(max_results: usize) {
    let k = usize::MAX; // UNBOUNDED-OK: deliberate test unbounded
}
"#,
        )
        .unwrap();

        let violations = run_check_max_results_unbound(dir.path()).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_various_max_literals_and_param_patterns() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("bounded.rs");
        fs::write(
            &file,
            r#"
fn run_hops(max_hops: u32) {
    let h = u32::MAX;
}

fn run_items(max_items: u64) {
    let t = u64::MAX;
}

fn run_count(max_count: usize) {
    let b = usize::MAX;
}

fn run_depth(max_depth: u32) {
    let c = i64::MAX;
}
"#,
        )
        .unwrap();

        let violations = run_check_max_results_unbound(dir.path()).unwrap();
        assert_eq!(violations.len(), 4);
    }

    #[test]
    fn test_unbounded_ok_suppresses_various_patterns() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("bounded.rs");
        fs::write(
            &file,
            r#"
fn traverse(max_hops: u32, timeout: u64, memory_budget: usize) {
    let h = u32::MAX; // UNBOUNDED-OK: unlimited hops allowed
    let t = u64::MAX; // UNBOUNDED-OK: no timeout
    let b = usize::MAX; // UNBOUNDED-OK: unlimited budget
}
"#,
        )
        .unwrap();

        let violations = run_check_max_results_unbound(dir.path()).unwrap();
        assert_eq!(violations.len(), 0);
    }
}
