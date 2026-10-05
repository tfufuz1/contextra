//! Harness Modul: build-freier Diff-Scope-Linter für die häufigsten Doktrin-Verstöße (fast-diff-lint).

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FastDiffFinding {
    pub file: String,
    pub line: usize,
    pub category: String,
    pub message: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastDiffGateResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<FastDiffFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffAddedLine {
    pub line_number: usize,
    pub content: String,
    pub diff_window_before: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffFile {
    pub path: String,
    pub added_lines: Vec<DiffAddedLine>,
}

pub(crate) fn parse_unified_diff(diff_text: &str) -> Vec<DiffFile> {
    let mut files = Vec::new();
    let mut current_file: Option<String> = None;
    let mut current_added_lines = Vec::new();
    let mut current_window = Vec::new();
    let mut new_line_num = 0;

    let re_hunk = Regex::new(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@").unwrap();

    for line in diff_text.lines() {
        if line.starts_with("+++ ") {
            if let Some(path) = current_file.take() {
                files.push(DiffFile {
                    path,
                    added_lines: std::mem::take(&mut current_added_lines),
                });
            }
            current_window.clear();

            let raw_path = line.trim_start_matches("+++ ").trim();
            if raw_path == "/dev/null" || raw_path.is_empty() {
                current_file = None;
            } else {
                let cleaned_path = raw_path
                    .strip_prefix("b/")
                    .unwrap_or(raw_path)
                    .to_string();
                current_file = Some(cleaned_path);
            }
            continue;
        }

        if line.starts_with("--- ") || line.starts_with("diff --git") || line.starts_with("index ") {
            continue;
        }

        if let Some(caps) = re_hunk.captures(line) {
            if let Some(start_str) = caps.get(1) {
                new_line_num = start_str.as_str().parse::<usize>().unwrap_or(1);
            }
            continue;
        }

        if current_file.is_none() {
            continue;
        }

        if line.starts_with('+') && !line.starts_with("+++") {
            let content = line[1..].to_string();
            current_added_lines.push(DiffAddedLine {
                line_number: new_line_num,
                content: content.clone(),
                diff_window_before: current_window.clone(),
            });
            current_window.push(content);
            new_line_num += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            let content = line[1..].to_string();
            current_window.push(content);
        } else if line.starts_with(' ') {
            let content = line[1..].to_string();
            current_window.push(content);
            new_line_num += 1;
        } else if line.is_empty() {
            current_window.push(String::new());
            new_line_num += 1;
        }
    }

    if let Some(path) = current_file.take() {
        files.push(DiffFile {
            path,
            added_lines: current_added_lines,
        });
    }

    files
}

fn extract_crate_name(file_path: &str) -> Option<String> {
    let re = Regex::new(r"^crates/([^/]+)/").unwrap();
    re.captures(file_path).map(|c| c[1].to_string())
}

fn is_test_context(file_path: &str, diff_window_before: &[String]) -> bool {
    if file_path.contains("/tests/") || file_path.starts_with("tests/") || file_path.ends_with("_test.rs") {
        return true;
    }

    let window_slice = if diff_window_before.len() > 60 {
        &diff_window_before[diff_window_before.len() - 60..]
    } else {
        diff_window_before
    };

    let re_cfg_test = Regex::new(r"#\[cfg\(test\)\]").unwrap();
    let re_mod_tests = Regex::new(r"\bmod\s+tests\b").unwrap();

    for line in window_slice.iter().rev() {
        if re_cfg_test.is_match(line) || re_mod_tests.is_match(line) {
            return true;
        }
    }

    false
}

fn lint_file(file: &DiffFile) -> Vec<FastDiffFinding> {
    let mut findings = Vec::new();
    let is_rs = file.path.ends_with(".rs");
    let crate_name = extract_crate_name(&file.path);

    let re_zero_panic = Regex::new(r"\.unwrap\(\)|\.expect\(|panic!|todo!|unimplemented!|unreachable!").unwrap();
    let re_debug = Regex::new(r"\bdbg!\(|^\s*println!\(").unwrap();
    let re_unsafe = Regex::new(r"\bunsafe\b.*(?:\{|\bfn\b)").unwrap();
    let re_ring2_use = Regex::new(r"\buse\s+(?:contextra_infer_candle|contextra_infer_ollama|contextra_infer_onnx|contextra_sandbox)\b").unwrap();
    let re_todo = Regex::new(r"\b(TODO|FIXME|XXX)\b").unwrap();
    let re_shell_commit = Regex::new(r#"(?i)git\s+commit\s+-m\s+["'](wip|fix|update|test|tmp|stuff)["']"#).unwrap();

    let allowed_islands: HashSet<&str> = ["contextra-sys", "contextra-simd", "contextra-wire"]
        .iter()
        .copied()
        .collect();

    let allowed_ring2_callers: HashSet<&str> = [
        "contextra",
        "contextra-mcp",
        "contextra-py",
        "contextra-infer-candle",
        "contextra-infer-ollama",
        "contextra-infer-onnx",
    ]
    .iter()
    .copied()
    .collect();

    for added_line in &file.added_lines {
        let text = &added_line.content;
        let line_num = added_line.line_number;

        // General rules (for ALL files)
        for mat in re_todo.find_iter(text) {
            let remainder = &text[mat.end()..];
            if !remainder.starts_with("(#") {
                findings.push(FastDiffFinding {
                    file: file.path.clone(),
                    line: line_num,
                    category: "todo-marker".to_string(),
                    message: "TODO/FIXME/XXX-Marker ohne Issue/Task-Referrenzsyntax (#...)".to_string(),
                    text: text.clone(),
                });
                break;
            }
        }

        if re_shell_commit.is_match(text) {
            findings.push(FastDiffFinding {
                file: file.path.clone(),
                line: line_num,
                category: "shell-commit".to_string(),
                message: "Potentielles Entwickler-Shell-Scripting mit 'git commit -m ...' entdeckt".to_string(),
                text: text.clone(),
            });
        }

        // Rust-specific rules (only for .rs files NOT in test context)
        if is_rs && !is_test_context(&file.path, &added_line.diff_window_before) {
            if re_zero_panic.is_match(text) {
                findings.push(FastDiffFinding {
                    file: file.path.clone(),
                    line: line_num,
                    category: "zero-panic".to_string(),
                    message: "Verwendung von panic-auslösenden Konstrukten in Nicht-Test-Code".to_string(),
                    text: text.clone(),
                });
            }

            if re_debug.is_match(text) {
                findings.push(FastDiffFinding {
                    file: file.path.clone(),
                    line: line_num,
                    category: "debug-leftover".to_string(),
                    message: "Debug-Ausgabe (dbg! oder println!) im Code hinterlassen".to_string(),
                    text: text.clone(),
                });
            }

            if re_unsafe.is_match(text) {
                let has_safety_comment = added_line
                    .diff_window_before
                    .iter()
                    .any(|prev| prev.contains("SAFETY:"));

                if !has_safety_comment {
                    findings.push(FastDiffFinding {
                        file: file.path.clone(),
                        line: line_num,
                        category: "unsafe-no-safety-comment".to_string(),
                        message: "unsafe-Block oder unsafe-Funktion ohne SAFETY:-Kommentar in den vorherigen Zeilen".to_string(),
                        text: text.clone(),
                    });
                }

                let is_island = crate_name
                    .as_deref()
                    .map_or(false, |c| allowed_islands.contains(c));

                if !is_island {
                    findings.push(FastDiffFinding {
                        file: file.path.clone(),
                        line: line_num,
                        category: "unsafe-outside-island".to_string(),
                        message: "unsafe außerhalb der erlaubten Unsafe-Islands (contextra-sys, contextra-simd, contextra-wire)".to_string(),
                        text: text.clone(),
                    });
                }
            }

            if re_ring2_use.is_match(text) {
                let is_allowed_ring2_caller = crate_name
                    .as_deref()
                    .map_or(false, |c| allowed_ring2_callers.contains(c));

                if !is_allowed_ring2_caller {
                    findings.push(FastDiffFinding {
                        file: file.path.clone(),
                        line: line_num,
                        category: "ring2-leaf-violation".to_string(),
                        message: "Ring-2/3 Dependency-Verstoß: Unzulässige Dependency auf Ring-2 Leaf Crates".to_string(),
                        text: text.clone(),
                    });
                }
            }
        }
    }

    findings
}

pub fn run_fast_diff_lint(args: &[String]) -> i32 {
    let mut base = "HEAD".to_string();
    let mut path_prefix: Option<String> = None;
    let mut json_output = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                if i + 1 < args.len() {
                    base = args[i + 1].clone();
                    i += 1;
                }
            }
            "--path" => {
                if i + 1 < args.len() {
                    path_prefix = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--json" => {
                json_output = true;
            }
            _ => {}
        }
        i += 1;
    }

    let mut git_args = vec!["diff", "--unified=1", &base];
    if let Some(ref path) = path_prefix {
        git_args.push("--");
        git_args.push(path);
    }

    let output = match Command::new("git").args(&git_args).output() {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Git error executing command: {}", e);
            return 1;
        }
    };

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        eprintln!("Git command failed: {}", err_msg.trim());
        return 1;
    }

    let diff_text = String::from_utf8_lossy(&output.stdout);
    let diff_files = parse_unified_diff(&diff_text);

    let mut all_findings = Vec::new();
    for file in &diff_files {
        all_findings.extend(lint_file(file));
    }

    let status = if all_findings.is_empty() { "pass" } else { "fail" };
    let summary = if all_findings.is_empty() {
        "Keine Doktrin-Verstöße im Diff gefunden".to_string()
    } else {
        format!("{} Doktrin-Verstoß/Verstöße im Diff gefunden", all_findings.len())
    };

    let gate_result = FastDiffGateResult {
        gate: "fast-diff-lint".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        findings: all_findings.clone(),
    };

    if json_output {
        println!("{}", serde_json::to_string(&gate_result).unwrap_or_default());
    } else {
        println!("=== Gate fast-diff-lint: {} ===", gate_result.status);
        println!("{}", gate_result.summary);
        for f in &gate_result.findings {
            println!(
                "[{}] {}:{}: {}\n  Code: {}",
                f.category, f.file, f.line, f.message, f.text
            );
        }
    }

    if all_findings.is_empty() {
        0
    } else {
        2
    }
}
