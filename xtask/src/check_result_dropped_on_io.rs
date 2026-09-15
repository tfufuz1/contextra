use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct Violation {
    pub file_path: String,
    pub line_num: usize,
    pub line_content: String,
}

pub fn run_check_result_dropped_on_io(root: &Path) -> Result<Vec<Violation>, String> {
    run_check_result_dropped_on_io_with_options(root, false)
}

pub fn run_check_result_dropped_on_io_with_options(
    root: &Path,
    include_tests: bool,
) -> Result<Vec<Violation>, String> {
    let mut violations = Vec::new();

    let drop_re = Regex::new(r"\blet\s+_\s*=").unwrap();
    let io_expr_re =
        Regex::new(r"\b(write|write_all|flush|set_len|fsync|sync_all|seek|store|truncate)\s*\(")
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

                    if trimmed.starts_with("//")
                        || trimmed.contains("// INTENTIONAL-DROP")
                    {
                        continue;
                    }

                    if drop_re.is_match(line) && io_expr_re.is_match(line) {
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

    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_detects_io_result_dropped() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("wal_io.rs");
        fs::write(
            &file,
            r#"
fn write_wal(file: &mut std::fs::File) {
    let _ = file.flush();
}
"#,
        )
        .unwrap();

        let violations = run_check_result_dropped_on_io(dir.path()).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line_num, 3);
    }

    #[test]
    fn test_ignores_intentional_drop() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("wal_io.rs");
        fs::write(
            &file,
            r#"
fn write_wal(file: &mut std::fs::File) {
    let _ = file.flush(); // INTENTIONAL-DROP
}
"#,
        )
        .unwrap();

        let violations = run_check_result_dropped_on_io(dir.path()).unwrap();
        assert_eq!(violations.len(), 0);
    }
}
