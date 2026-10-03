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
pub fn run_check_result_dropped_on_io(root: &Path) -> Result<Vec<Violation>, String> {
    run_check_result_dropped_on_io_with_options(root, false)
}

/// Baut Kommentare und String-Literale aus einer Datei-Zeile für Zeile ab,
/// damit `let _ =` in Kommentaren oder Dok-Strings keine Fehlalarme erzeugt.
fn strip_comments_and_strings(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;

    let mut in_line_comment = false;
    let mut in_block_comment = 0;
    let mut in_string = false;
    let mut in_char = false;

    while i < len {
        let c = chars[i];
        let next = if i + 1 < len {
            Some(chars[i + 1])
        } else {
            None
        };

        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                result.push('\n');
            } else {
                result.push(' ');
            }
            i += 1;
            continue;
        }

        if in_block_comment > 0 {
            if c == '/' && next == Some('*') {
                in_block_comment += 1;
                result.push(' ');
                result.push(' ');
                i += 2;
            } else if c == '*' && next == Some('/') {
                in_block_comment -= 1;
                result.push(' ');
                result.push(' ');
                i += 2;
            } else {
                if c == '\n' {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
                i += 1;
            }
            continue;
        }

        if in_string {
            if c == '\\' {
                result.push(' ');
                result.push(' ');
                i += 2;
            } else if c == '"' {
                in_string = false;
                result.push(' ');
                i += 1;
            } else {
                if c == '\n' {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
                i += 1;
            }
            continue;
        }

        if in_char {
            if c == '\\' {
                result.push(' ');
                result.push(' ');
                i += 2;
            } else if c == '\'' {
                in_char = false;
                result.push(' ');
                i += 1;
            } else {
                if c == '\n' {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
                i += 1;
            }
            continue;
        }

        if c == '/' && next == Some('/') {
            in_line_comment = true;
            result.push(' ');
            result.push(' ');
            i += 2;
            continue;
        }

        if c == '/' && next == Some('*') {
            in_block_comment = 1;
            result.push(' ');
            result.push(' ');
            i += 2;
            continue;
        }

        if c == '"' {
            in_string = true;
            result.push(' ');
            i += 1;
            continue;
        }

        if c == '\'' {
            let is_char_lit = if i + 2 < len && chars[i + 2] == '\'' && chars[i + 1] != '\\' {
                true
            } else if i + 3 < len && chars[i + 1] == '\\' && chars[i + 3] == '\'' {
                true
            } else if i + 4 < len
                && chars[i + 1] == '\\'
                && chars[i + 2] == 'x'
                && chars[i + 4] == '\''
            {
                true
            } else if i + 3 < len && chars[i + 1] == '\\' && chars[i + 2] == 'u' {
                let mut found_end = false;
                for j in (i + 3)..len.min(i + 12) {
                    if chars[j] == '\'' {
                        found_end = true;
                        break;
                    }
                }
                found_end
            } else {
                false
            };

            if is_char_lit {
                in_char = true;
                result.push(' ');
                i += 1;
                continue;
            } else {
                result.push(c);
                i += 1;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

pub fn run_check_result_dropped_on_io_with_options(
    root: &Path,
    include_tests: bool,
) -> Result<Vec<Violation>, String> {
    let mut violations = Vec::new();

    let drop_re = Regex::new(r"\blet\s+_\s*=").unwrap();
    let io_expr_re =
        Regex::new(r"\b(write|write_all|flush|set_len|fsync|sync_all|seek|truncate)\s*\(").unwrap();

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
                let stripped = strip_comments_and_strings(&content);
                let lines: Vec<&str> = content.lines().collect();
                let stripped_lines: Vec<&str> = stripped.lines().collect();

                for (idx, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    if trimmed.starts_with("//") || trimmed.contains("// INTENTIONAL-DROP") {
                        continue;
                    }

                    let stripped_line = stripped_lines.get(idx).copied().unwrap_or("");

                    if drop_re.is_match(stripped_line) && io_expr_re.is_match(stripped_line) {
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

    #[test]
    fn test_ignores_comment_and_string_false_positives() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("false_positives.rs");
        fs::write(
            &file,
            r#"
fn test_comments_and_strings() {
    // let _ = file.write_all(buf);
    /* let _ = file.flush(); */
    let s = "let _ = file.write_all(buf)";
    let atomic_val = std::sync::atomic::AtomicU64::new(0);
    atomic_val.store(42, std::sync::atomic::Ordering::SeqCst);
    let _ = atomic_val;
}
"#,
        )
        .unwrap();

        let violations = run_check_result_dropped_on_io(dir.path()).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_async_io_result_dropped_negative() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("fuzz_target.rs");
        fs::write(
            &file,
            r#"
async fn run(storage: &Storage) {
    let _ = storage.flush().await;
}
"#,
        )
        .unwrap();

        let violations = run_check_result_dropped_on_io(dir.path()).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line_num, 3);
        assert_eq!(violations[0].line_content, "let _ = storage.flush().await;");
    }

    #[test]
    fn test_async_io_result_handled_positive() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("fuzz_target.rs");
        fs::write(
            &file,
            r#"
async fn run(storage: &Storage) {
    if let Err(_e) = storage.flush().await {}
}
"#,
        )
        .unwrap();

        let violations = run_check_result_dropped_on_io(dir.path()).unwrap();
        assert_eq!(violations.len(), 0);
    }
}
