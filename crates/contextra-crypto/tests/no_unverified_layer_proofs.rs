// NEW: crates/contextra-crypto/tests/no_unverified_layer_proofs.rs

use std::fs;
use std::path::{Path, PathBuf};

const PERMANENT_ALLOW: &[&str] = &[
    "crates/contextra-crypto/src/deletion_proof.rs",
    "crates/contextra-engine/src/lib.rs",
];

const TEMPORARY_ALLOW: &[&str] = &["crates/contextra-mcp/src/tools_crud.rs"];

#[derive(Debug, PartialEq, Eq)]
struct Violation {
    line: usize,
    last_arg: String,
}

fn preprocess_source(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let test_line_idx = lines
        .iter()
        .position(|l| l.contains("#[cfg(test)]"))
        .unwrap_or(lines.len());

    let mut result_lines = Vec::with_capacity(lines.len());
    for (idx, line) in lines.iter().enumerate() {
        if idx >= test_line_idx {
            result_lines.push("");
        } else if line.trim().starts_with("//") {
            result_lines.push("");
        } else {
            result_lines.push(*line);
        }
    }
    result_lines.join("\n")
}

fn find_unverified_layer_proof_violations(content: &str) -> Vec<Violation> {
    let preprocessed = preprocess_source(content);
    let key = "new_after_verified_empty(";
    let mut violations = Vec::new();

    for (match_idx, _) in preprocessed.match_indices(key) {
        let line = 1 + preprocessed[..match_idx]
            .bytes()
            .filter(|&b| b == b'\n')
            .count();
        let open_paren_idx = match_idx + key.len() - 1;

        if let Some(args) = extract_top_level_args(&preprocessed[open_paren_idx..]) {
            if let Some(last) = get_last_arg(&args) {
                let trimmed = last.trim();
                if trimmed == "0" || trimmed == "0usize" {
                    violations.push(Violation {
                        line,
                        last_arg: trimmed.to_string(),
                    });
                }
            }
        }
    }

    violations
}

fn extract_top_level_args(slice_from_open_paren: &str) -> Option<Vec<String>> {
    let mut chars = slice_from_open_paren.char_indices();
    match chars.next() {
        Some((_, '(')) => {}
        _ => return None,
    }

    let mut paren_depth = 1i32;
    let mut bracket_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;

    let mut args = Vec::new();
    let mut current_arg = String::new();

    for (_, c) in chars {
        if in_string {
            current_arg.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        match c {
            '"' => {
                in_string = true;
                current_arg.push(c);
            }
            '(' => {
                paren_depth += 1;
                current_arg.push(c);
            }
            ')' => {
                paren_depth -= 1;
                if paren_depth == 0 {
                    args.push(current_arg);
                    return Some(args);
                }
                current_arg.push(c);
            }
            '[' => {
                bracket_depth += 1;
                current_arg.push(c);
            }
            ']' => {
                bracket_depth -= 1;
                current_arg.push(c);
            }
            '{' => {
                brace_depth += 1;
                current_arg.push(c);
            }
            '}' => {
                brace_depth -= 1;
                current_arg.push(c);
            }
            ',' if paren_depth == 1 && bracket_depth == 0 && brace_depth == 0 => {
                args.push(current_arg);
                current_arg = String::new();
            }
            _ => {
                current_arg.push(c);
            }
        }
    }

    None
}

fn get_last_arg(args: &[String]) -> Option<&str> {
    let mut non_empty_args = Vec::new();
    for arg in args {
        let trimmed = arg.trim();
        if !trimmed.is_empty() {
            non_empty_args.push(trimmed);
        }
    }
    non_empty_args.last().copied()
}

fn scan_crate_src_files(crates_dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !crates_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(crates_dir)? {
        let entry = entry?;
        let crate_path = entry.path();
        if crate_path.is_dir() {
            let src_dir = crate_path.join("src");
            if src_dir.is_dir() {
                find_rs_files_recursive(&src_dir, files)?;
            }
        }
    }
    Ok(())
}

fn find_rs_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                find_rs_files_recursive(&path, files)?;
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    Ok(())
}

#[test]
fn test_no_unverified_layer_proofs() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.join("../..").canonicalize()?;

    // Verify PERMANENT_ALLOW files exist
    for perm_file in PERMANENT_ALLOW {
        let path = workspace_root.join(perm_file);
        assert!(
            path.exists(),
            "PERMANENT_ALLOW file does not exist: {perm_file}"
        );
    }

    // Verify engine lib.rs has at most ONE violation
    let engine_lib_path = workspace_root.join("crates/contextra-engine/src/lib.rs");
    let engine_lib_content = fs::read_to_string(&engine_lib_path)?;
    let engine_violations = find_unverified_layer_proof_violations(&engine_lib_content);
    assert!(
        engine_violations.len() <= 1,
        "crates/contextra-engine/src/lib.rs contains more than 1 violation: {engine_violations:?}"
    );

    // Verify TEMPORARY_ALLOW files exist and still contain violations (shrink-only rule)
    for temp_file in TEMPORARY_ALLOW {
        let path = workspace_root.join(temp_file);
        assert!(
            path.exists(),
            "TEMPORARY_ALLOW file does not exist: {temp_file}"
        );
        let content = fs::read_to_string(&path)?;
        let violations = find_unverified_layer_proof_violations(&content);
        if violations.is_empty() {
            panic!("remove stale entry from TEMPORARY_ALLOW: {temp_file}");
        }
    }

    // Scan all crates/*/src/**/*.rs
    let crates_dir = workspace_root.join("crates");
    let mut files = Vec::new();
    scan_crate_src_files(&crates_dir, &mut files)?;

    let mut unexpected_violations = Vec::new();

    for file_path in files {
        let rel_path = match file_path.strip_prefix(&workspace_root) {
            Ok(p) => p.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };

        if PERMANENT_ALLOW.contains(&rel_path.as_str())
            || TEMPORARY_ALLOW.contains(&rel_path.as_str())
        {
            continue;
        }

        let content = fs::read_to_string(&file_path)?;
        let violations = find_unverified_layer_proof_violations(&content);
        for v in violations {
            unexpected_violations.push(format!(
                "{}:{}: violation found\n  hint: use LayerCleanupProof::verify_and_create(layer, || real post-condition check) or pass the real remaining count",
                rel_path, v.line
            ));
        }
    }

    if !unexpected_violations.is_empty() {
        panic!(
            "Found unverified LayerCleanupProof creation in production code:\n{}",
            unexpected_violations.join("\n")
        );
    }

    Ok(())
}

#[test]
fn test_fixture_literal_zero_detected() {
    let fixture = r#"
pub fn do_something() {
    /* Test-Fixture, keine Produktion */ LayerCleanupProof::verify_and_create(DeletionLayer::LsmMemtable, || Ok(true));
}
"#;
    let violations = find_unverified_layer_proof_violations(fixture);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].line, 3);
    assert_eq!(violations[0].last_arg, "0");
}

#[test]
fn test_fixture_literal_zero_usize_detected() {
    let fixture = r#"
pub fn do_something() {
    LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0usize);
}
"#;
    let violations = find_unverified_layer_proof_violations(fixture);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].line, 3);
    assert_eq!(violations[0].last_arg, "0usize");
}

#[test]
fn test_fixture_remaining_len_not_detected() {
    let fixture = r#"
pub fn do_something() {
    let remaining = vec![1, 2];
    LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, remaining.len());
}
"#;
    let violations = find_unverified_layer_proof_violations(fixture);
    assert!(violations.is_empty());
}

#[test]
fn test_fixture_nested_parens_and_multiline_handled() {
    let fixture = r#"
pub fn do_something() {
    LayerCleanupProof::new_after_verified_empty(
        DeletionLayer::WalAllSegments { seq_after: (10 + 2) },
        0,
    );
}
"#;
    let violations = find_unverified_layer_proof_violations(fixture);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].line, 3);
    assert_eq!(violations[0].last_arg, "0");
}

#[test]
fn test_fixture_call_inside_comment_ignored() {
    let fixture = r#"
pub fn do_something() {
    // /* Test-Fixture, keine Produktion */ LayerCleanupProof::verify_and_create(DeletionLayer::LsmMemtable, || Ok(true));
    /// /* Test-Fixture, keine Produktion */ LayerCleanupProof::verify_and_create(DeletionLayer::LsmMemtable, || Ok(true));
}
"#;
    let violations = find_unverified_layer_proof_violations(fixture);
    assert!(violations.is_empty());
}

#[test]
fn test_fixture_content_after_cfg_test_ignored() {
    let fixture = r#"
pub fn do_something() {
    let n = 5;
}

#[cfg(test)]
mod tests {
    fn test_foo() {
        /* Test-Fixture, keine Produktion */ LayerCleanupProof::verify_and_create(DeletionLayer::LsmMemtable, || Ok(true));
    }
}
"#;
    let violations = find_unverified_layer_proof_violations(fixture);
    assert!(violations.is_empty());
}
