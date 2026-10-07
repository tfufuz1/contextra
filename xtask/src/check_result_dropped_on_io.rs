use regex::Regex;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Violation {
    pub file_path: String,
    pub line_num: usize,
    pub line_content: String,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct IoResultBaselineFile {
    pub violations: BTreeMap<String, bool>,
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

fn get_crate_ring(root: &Path, rel_path: &str) -> Option<u8> {
    let caps_path = root.join("capabilities.toml");
    if !caps_path.exists() {
        return None;
    }
    let content = fs::read_to_string(&caps_path).ok()?;
    let val: serde_json::Value = toml::from_str(&content).ok()?;
    let crates_map = val.get("crates")?.as_object()?;

    for (crate_name, crate_obj) in crates_map {
        let path = crate_obj.get("path")?.as_str()?;
        if rel_path.starts_with(path) {
            let ring_str = crate_obj.get("ring")?.as_str()?;
            if ring_str.contains('0') {
                return Some(0);
            } else if ring_str.contains('1') {
                return Some(1);
            }
            let _ = crate_name;
        }
    }
    None
}

pub fn run_check_result_dropped_on_io_with_options(
    root: &Path,
    include_tests: bool,
) -> Result<Vec<Violation>, String> {
    let mut raw_violations = Vec::new();

    let drop_re = Regex::new(r"\blet\s+_\s*=").unwrap();
    let io_expr_re = Regex::new(
        r"\b(write|write_all|flush|set_len|fsync|sync_all|seek|truncate|remove_file|rename|create_dir_all)\s*\(",
    )
    .unwrap();

    let if_let_ok_io_re = Regex::new(
        r"\bif\s+let\s+Ok\s*\([^)]*\)\s*=\s*(File::|OpenOptions::|fs::|std::fs::)",
    )
    .unwrap();

    let ok_ignored_re = Regex::new(
        r"\.(write|write_all|flush|set_len|fsync|sync_all|seek|truncate|remove_file|rename|create_dir_all)\s*\([^;]*\)\s*\.ok\s*\(\s*\)\s*;",
    )
    .unwrap();

    let unwrap_or_else_re = Regex::new(r"\bunwrap_or_else\s*\(\s*\|[^|]*\|\s*").unwrap();

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

            let is_test = rel_path.ends_with("_test.rs") || rel_path.contains("/tests/");

            if !include_tests && is_test {
                continue;
            }

            let ring = get_crate_ring(root, &rel_path);
            let is_ring0_or_1 = ring == Some(0) || ring == Some(1);

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

                    let mut matched = false;

                    // 1. Existing let _ = with I/O function
                    if drop_re.is_match(stripped_line) && io_expr_re.is_match(stripped_line) {
                        matched = true;
                    }

                    // 2. if let Ok(...) = File::open / OpenOptions / fs::* in Ring 0 and Ring 1
                    if !matched && is_ring0_or_1 && if_let_ok_io_re.is_match(stripped_line) {
                        matched = true;
                    }

                    // 3. .ok(); after I/O calls
                    if !matched && ok_ignored_re.is_match(stripped_line) {
                        matched = true;
                    }

                    // 4. unwrap_or_else(|_| ...) in Ring 0 and Ring 1, not in test code
                    if !matched && is_ring0_or_1 && !is_test && unwrap_or_else_re.is_match(stripped_line) {
                        matched = true;
                    }

                    if matched {
                        raw_violations.push(Violation {
                            file_path: rel_path.clone(),
                            line_num: idx + 1,
                            line_content: trimmed.to_string(),
                        });
                    }
                }
            }
        }
    }

    // Baseline Ratchet Filtering
    let baseline_path = root.join("governance/io-result-baseline.toml");
    let baseline_set: HashSet<String> = if baseline_path.exists() {
        if let Ok(content) = fs::read_to_string(&baseline_path) {
            if let Ok(parsed) = toml::from_str::<IoResultBaselineFile>(&content) {
                parsed.violations.into_keys().collect()
            } else {
                HashSet::new()
            }
        } else {
            HashSet::new()
        }
    } else {
        HashSet::new()
    };

    let new_violations: Vec<Violation> = raw_violations
        .into_iter()
        .filter(|v| {
            let key = format!("{}:{}:{}", v.file_path, v.line_num, v.line_content);
            !baseline_set.contains(&key)
        })
        .collect();

    Ok(new_violations)
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

    #[test]
    fn test_detects_remove_file_rename_create_dir_all_dropped() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("fs_op.rs");
        fs::write(
            &file,
            r#"
fn ops() {
    let _ = std::fs::remove_file("a.tmp");
    let _ = std::fs::rename("a.tmp", "b.tmp");
    let _ = std::fs::create_dir_all("/tmp/dir");
}
"#,
        )
        .unwrap();

        let violations = run_check_result_dropped_on_io(dir.path()).unwrap();
        assert_eq!(violations.len(), 3);
    }

    #[test]
    fn test_detects_ok_ignored_call() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("ok_op.rs");
        fs::write(
            &file,
            r#"
fn ops(file: &mut std::fs::File) {
    file.flush().ok();
}
"#,
        )
        .unwrap();

        let violations = run_check_result_dropped_on_io(dir.path()).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line_num, 3);
    }

    #[test]
    fn test_detects_if_let_ok_and_unwrap_or_else_in_ring0_1() {
        let dir = tempdir().unwrap();

        let caps = r#"
[crates.contextra-mvcc]
ring = "Ring 0"
path = "crates/contextra-mvcc"
"#;
        fs::write(dir.path().join("capabilities.toml"), caps).unwrap();

        let mvcc_src = dir.path().join("crates/contextra-mvcc/src");
        fs::create_dir_all(&mvcc_src).unwrap();

        let file = mvcc_src.join("sample.rs");
        fs::write(
            &file,
            r#"
fn sample() {
    if let Ok(f) = std::fs::File::open("data") {}
    let val = parse_val().unwrap_or_else(|_| 0);
}
"#,
        )
        .unwrap();

        let violations = run_check_result_dropped_on_io(dir.path()).unwrap();
        assert_eq!(violations.len(), 2);
    }
}
