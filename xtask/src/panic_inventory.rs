//! Module: panic_inventory
//! Scans `crates/**/*.rs` for `.unwrap(`, `.expect(`, `panic!(`, `unimplemented!(`, `todo!(`.
//!
//! Guarantees: Zero-Panic (no unwrap/expect/panic in runtime code), Result-based error propagation.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanicKind {
    Unwrap,
    Expect,
    Panic,
    Unimplemented,
    Todo,
}

impl PanicKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PanicKind::Unwrap => "unwrap",
            PanicKind::Expect => "expect",
            PanicKind::Panic => "panic!",
            PanicKind::Unimplemented => "unimplemented!",
            PanicKind::Todo => "todo!",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanicEntry {
    pub file: String,
    pub line: usize,
    pub kind: PanicKind,
    pub context: String,
    pub is_test_code: bool,
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

fn load_baseline(root: &Path) -> HashSet<(String, usize)> {
    let baseline_path = root.join(".github").join("unwrap_baseline.txt");
    let mut set = HashSet::new();
    if let Ok(content) = fs::read_to_string(&baseline_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((file, num_str)) = line.rsplit_once(':') {
                if let Ok(line_num) = num_str.parse::<usize>() {
                    let norm_file = file.replace('\\', "/");
                    set.insert((norm_file, line_num));
                }
            }
        }
    }
    set
}

fn scan_rs_file(
    file_path: &Path,
    root: &Path,
    re_unwrap: &Regex,
    re_expect: &Regex,
    re_panic: &Regex,
    re_unimplemented: &Regex,
    re_todo: &Regex,
) -> Result<Vec<PanicEntry>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;

    let rel_path = file_path
        .strip_prefix(root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .replace('\\', "/");

    let file_is_test_dir = rel_path.contains("/tests/")
        || rel_path.contains("/benches/")
        || rel_path.starts_with("tests/")
        || rel_path.starts_with("benches/");

    let mut entries = Vec::new();
    let mut in_cfg_test = false;
    let mut cfg_test_brace_depth: i32 = 0;
    let mut cfg_test_seen_brace = false;

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let line_trimmed = line.trim();

        // Check if line starts or contains #[cfg(test)]
        if line_trimmed.contains("#[cfg(test)]") {
            in_cfg_test = true;
            cfg_test_brace_depth = 0;
            cfg_test_seen_brace = false;
        }

        if in_cfg_test {
            let open_braces = line.chars().filter(|&c| c == '{').count() as i32;
            let close_braces = line.chars().filter(|&c| c == '}').count() as i32;
            cfg_test_brace_depth += open_braces;
            if open_braces > 0 {
                cfg_test_seen_brace = true;
            }
            cfg_test_brace_depth -= close_braces;
            if cfg_test_seen_brace && cfg_test_brace_depth <= 0 {
                in_cfg_test = false;
            }
        }

        let is_test_code = file_is_test_dir || in_cfg_test;

        // Ignore single-line comments or doc comments if they contain panic words
        if line_trimmed.starts_with("//") {
            continue;
        }

        let kinds = [
            (re_unwrap, PanicKind::Unwrap),
            (re_expect, PanicKind::Expect),
            (re_panic, PanicKind::Panic),
            (re_unimplemented, PanicKind::Unimplemented),
            (re_todo, PanicKind::Todo),
        ];

        for (re, kind) in kinds {
            if re.is_match(line) {
                entries.push(PanicEntry {
                    file: rel_path.clone(),
                    line: line_num,
                    kind,
                    context: line.trim().to_string(),
                    is_test_code,
                });
            }
        }
    }

    Ok(entries)
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

pub fn run_panic_inventory(
    crate_filter: Option<&str>,
    exclude_tests: bool,
    strict: bool,
) -> Result<Vec<PanicEntry>, String> {
    let root = find_root_dir();
    run_panic_inventory_in_root(&root, crate_filter, exclude_tests, strict)
}

pub fn run_panic_inventory_in_root(
    root: &Path,
    crate_filter: Option<&str>,
    exclude_tests: bool,
    strict: bool,
) -> Result<Vec<PanicEntry>, String> {
    let crates_dir = root.join("crates");
    let baseline = load_baseline(root);

    let re_unwrap =
        Regex::new(r"\.unwrap\s*\(").map_err(|e| format!("Regex error for unwrap: {}", e))?;
    let re_expect =
        Regex::new(r"\.expect\s*\(").map_err(|e| format!("Regex error for expect: {}", e))?;
    let re_panic =
        Regex::new(r"\bpanic!\s*").map_err(|e| format!("Regex error for panic: {}", e))?;
    let re_unimplemented = Regex::new(r"\bunimplemented!\s*")
        .map_err(|e| format!("Regex error for unimplemented: {}", e))?;
    let re_todo = Regex::new(r"\btodo!\s*").map_err(|e| format!("Regex error for todo: {}", e))?;

    let rs_files = collect_rs_files(&crates_dir, crate_filter);
    let mut all_entries = Vec::new();

    for file_path in rs_files {
        let entries = scan_rs_file(
            &file_path,
            root,
            &re_unwrap,
            &re_expect,
            &re_panic,
            &re_unimplemented,
            &re_todo,
        )?;
        for entry in entries {
            if exclude_tests && entry.is_test_code {
                continue;
            }
            all_entries.push(entry);
        }
    }

    if strict {
        let mut new_regressions = Vec::new();
        for entry in &all_entries {
            if !entry.is_test_code {
                let key = (entry.file.clone(), entry.line);
                if !baseline.contains(&key) {
                    new_regressions.push(entry);
                }
            }
        }
        if !new_regressions.is_empty() {
            let msg = format!(
                "Strict mode failure: {} new non-test panic/unwrap regression(s) found since baseline:\n{}",
                new_regressions.len(),
                new_regressions
                    .iter()
                    .take(10)
                    .map(|e| format!("  {}:{} [{}] {}", e.file, e.line, e.kind.as_str(), e.context))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            return Err(msg);
        }
    }

    Ok(all_entries)
}
