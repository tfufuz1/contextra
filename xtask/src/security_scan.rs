//! Module: security_scan
//! Scans `crates/**/*.rs` for security anti-patterns:
//! 1. ShellInterpolation: Command::new("sh"), Command::new("bash"), or .arg("-c")
//! 2. StdFsInAsync: std::fs:: in async fn without spawn_blocking
//! 3. HardcodedSecret: assignments to API_KEY, SECRET, PASSWORD, TOKEN with string literal > 8 chars.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityPattern {
    ShellInterpolation,
    StdFsInAsync,
    HardcodedSecret,
}

impl SecurityPattern {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityPattern::ShellInterpolation => "ShellInterpolation",
            SecurityPattern::StdFsInAsync => "StdFsInAsync",
            SecurityPattern::HardcodedSecret => "HardcodedSecret",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub file: String,
    pub line: usize,
    pub pattern: SecurityPattern,
    pub context: String,
}

fn find_root_dir() -> PathBuf {
    if let Ok(cargo_manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(cargo_manifest);
        if path.file_name().and_then(|s| s.to_str()) == Some("xtask") {
            if let Some(parent) = path.parent() {
                return parent.to_path_buf();
            }
        }
        return path;
    }
    let mut curr = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if curr.join("Cargo.toml").exists() && curr.join("capabilities.toml").exists() {
            return curr;
        }
        if !curr.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

fn collect_rs_files(dir: &Path, crate_filter: Option<&str>) -> Vec<PathBuf> {
    let mut rs_files = Vec::new();
    if !dir.exists() {
        return rs_files;
    }

    let walk = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok());
    for entry in walk {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Some(filter) = crate_filter {
                let path_str = path.to_string_lossy().replace('\\', "/");
                if !path_str.contains(&format!("/crates/{}/", filter))
                    && !path_str.contains(&format!("crates/{}/", filter))
                {
                    continue;
                }
            }
            rs_files.push(path.to_path_buf());
        }
    }
    rs_files.sort();
    rs_files
}

struct AsyncFnBlock {
    has_std_fs: bool,
    has_spawn_blocking: bool,
    std_fs_lines: Vec<(usize, String)>,
}

fn scan_rs_file(
    file_path: &Path,
    root: &Path,
    pattern_filter: Option<SecurityPattern>,
) -> Result<Vec<SecurityFinding>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;

    let rel_path = file_path
        .strip_prefix(root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .replace('\\', "/");

    let mut findings = Vec::new();

    let check_shell =
        pattern_filter.is_none() || pattern_filter == Some(SecurityPattern::ShellInterpolation);
    let check_async =
        pattern_filter.is_none() || pattern_filter == Some(SecurityPattern::StdFsInAsync);
    let check_secret =
        pattern_filter.is_none() || pattern_filter == Some(SecurityPattern::HardcodedSecret);

    let secret_re = Regex::new(
        r#"(?i)\b(?:let\s+mut\s+|let\s+|const\s+|static\s+)?([a_z0-9_]*(?:api_key|secret|password|token)[a_z0-9_]*)\s*[:=]\s*"(([^"\\]|\\.)*)"#
    ).map_err(|e| format!("Secret regex error: {}", e))?;

    // State machine for async fn scanning
    let mut current_async_fn: Option<AsyncFnBlock> = None;
    let mut async_brace_depth = 0;
    let mut async_seen_brace = false;

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let line_trimmed = line.trim();

        if line_trimmed.starts_with("//") {
            continue;
        }

        // 1. ShellInterpolation
        if check_shell
            && (line_trimmed.contains(r#"Command::new("sh")"#)
                || line_trimmed.contains(r#"Command::new("bash")"#)
                || line_trimmed.contains(r#".arg("-c")"#))
        {
            findings.push(SecurityFinding {
                file: rel_path.clone(),
                line: line_num,
                pattern: SecurityPattern::ShellInterpolation,
                context: line_trimmed.to_string(),
            });
        }

        // 2. StdFsInAsync
        if check_async {
            // Check for start of async fn
            if current_async_fn.is_none() && line.contains("async fn ") {
                current_async_fn = Some(AsyncFnBlock {
                    has_std_fs: false,
                    has_spawn_blocking: false,
                    std_fs_lines: Vec::new(),
                });
                async_brace_depth = 0;
                async_seen_brace = false;
            }

            if let Some(ref mut async_block) = current_async_fn {
                if line.contains("std::fs::") {
                    async_block.has_std_fs = true;
                    async_block
                        .std_fs_lines
                        .push((line_num, line_trimmed.to_string()));
                }
                if line.contains("spawn_blocking") {
                    async_block.has_spawn_blocking = true;
                }

                let open_braces = line.chars().filter(|&c| c == '{').count() as i32;
                let close_braces = line.chars().filter(|&c| c == '}').count() as i32;
                async_brace_depth += open_braces;
                if open_braces > 0 {
                    async_seen_brace = true;
                }
                async_brace_depth -= close_braces;

                if async_seen_brace && async_brace_depth <= 0 {
                    // Fn ended
                    if async_block.has_std_fs && !async_block.has_spawn_blocking {
                        for (fs_line, fs_ctx) in &async_block.std_fs_lines {
                            findings.push(SecurityFinding {
                                file: rel_path.clone(),
                                line: *fs_line,
                                pattern: SecurityPattern::StdFsInAsync,
                                context: fs_ctx.clone(),
                            });
                        }
                    }
                    current_async_fn = None;
                }
            }
        }

        // 3. HardcodedSecret
        if check_secret {
            for cap in secret_re.captures_iter(line) {
                if let Some(secret_val) = cap.get(2) {
                    let secret_str = secret_val.as_str();
                    if secret_str.len() > 8 && secret_str != "changeme" {
                        findings.push(SecurityFinding {
                            file: rel_path.clone(),
                            line: line_num,
                            pattern: SecurityPattern::HardcodedSecret,
                            context: line_trimmed.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(findings)
}

pub fn run_security_scan(
    crate_filter: Option<&str>,
    pattern_filter: Option<SecurityPattern>,
) -> Result<Vec<SecurityFinding>, String> {
    let root = find_root_dir();
    run_security_scan_in_root(&root, crate_filter, pattern_filter)
}

pub fn run_security_scan_in_root(
    root: &Path,
    crate_filter: Option<&str>,
    pattern_filter: Option<SecurityPattern>,
) -> Result<Vec<SecurityFinding>, String> {
    let crates_dir = root.join("crates");
    let rs_files = collect_rs_files(&crates_dir, crate_filter);

    let mut all_findings = Vec::new();

    for file_path in rs_files {
        let findings = scan_rs_file(&file_path, root, pattern_filter)?;
        all_findings.extend(findings);
    }

    Ok(all_findings)
}
