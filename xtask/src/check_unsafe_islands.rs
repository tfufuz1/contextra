//! Gate: check-unsafe-islands
//! Validiert die 3-Insel-Invarianten für `unsafe`-Code (§0.4, ADR-N03):
//! 1. Keyword `unsafe` ist nur in den definierten Inseln zulässig:
//!    `memfuse-sys`, `memfuse-simd`, `memfuse-wire`
//!    (Phase 0R Übergangsliste: `memfuse-index`, `memfuse-store`, `memfuse-db`).
//! 2. Jede Nicht-Insel-`lib.rs` enthält `#![forbid(unsafe_code)]`.
//! 3. `#![allow(unsafe_code)]` ist NUR in den 3 Inseln zulässig.
//!
//! Note: `memfuse-py` (isolierter PyO3 Workspace) ist vom Scan ausgenommen.

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Die 3 zulässigen Unsafe-Inseln
pub const ALLOWED_ISLANDS: &[&str] = &["memfuse-sys", "memfuse-simd", "memfuse-wire"];

/// Phase 0R Übergangsliste für Bestands-`unsafe` Vorkommen
pub const PHASE_0R_TRANSITION_ISLANDS: &[&str] = &[
    "memfuse-index",
    "memfuse-store",
    "memfuse-db",
    // TRANSITION-EXTRA: test allocators, benchmark fixtures
    "memfuse-crypto",
    "memfuse-graph",
    "memfuse-text",
    "memfuse-bench",
    "xtask",
];

#[derive(Debug, Clone)]
pub struct UnsafeOccurrence {
    pub file_path: String,
    pub line_num: usize,
    pub crate_name: String,
    pub kind: UnsafeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsafeKind {
    Keyword,
    AttributeAllow,
    MissingForbid,
}

/// Tokenisiert den Quellcode und entfernt Kommentare (`//...`, `/* ... */`), String- und Char-Literale.
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
        let next = if i + 1 < len { Some(chars[i + 1]) } else { None };

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
            let prev_alphanum = if i > 0 {
                chars[i - 1].is_alphanumeric() || chars[i - 1] == '_'
            } else {
                false
            };
            if !prev_alphanum {
                in_char = true;
                result.push(' ');
                i += 1;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

/// Ermittelt den Crate-Namen aus dem Pfad (z. B. `crates/memfuse-core/src/lib.rs` -> `memfuse-core`).
pub fn extract_crate_name_from_path(rel_path: &str) -> Option<String> {
    let p = Path::new(rel_path);
    let components: Vec<&str> = p.iter().filter_map(|c| c.to_str()).collect();

    if components.first() == Some(&"crates") || components.first() == Some(&"benchmarks") {
        if let Some(c_name) = components.get(1) {
            return Some(c_name.to_string());
        }
    } else if components.first() == Some(&"xtask") {
        return Some("xtask".to_string());
    }

    None
}

/// Scannt eine `.rs`-Datei nach Keyword `unsafe` und Attributen.
pub fn scan_file_for_unsafe(
    file_path: &Path,
    repo_root: &Path,
) -> Vec<UnsafeOccurrence> {
    let mut occurrences = Vec::new();

    let rel_path = file_path
        .strip_prefix(repo_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .replace('\\', "/");

    let crate_name = match extract_crate_name_from_path(&rel_path) {
        Some(name) => name,
        None => return occurrences,
    };

    // Exclude memfuse-py from scan (PyO3 C-FFI macros, isolated Cargo workspace)
    if crate_name == "memfuse-py" {
        return occurrences;
    }

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return occurrences,
    };

    let stripped = strip_comments_and_strings(&content);
    let unsafe_kw_re = regex::Regex::new(r"\bunsafe\b").unwrap();
    let allow_unsafe_re = regex::Regex::new(r"#!\s*\[\s*allow\s*\(\s*unsafe_code\s*\)\s*\]").unwrap();

    for (line_idx, line) in stripped.lines().enumerate() {
        let line_num = line_idx + 1;

        if allow_unsafe_re.is_match(line) {
            occurrences.push(UnsafeOccurrence {
                file_path: rel_path.clone(),
                line_num,
                crate_name: crate_name.clone(),
                kind: UnsafeKind::AttributeAllow,
            });
        }

        if unsafe_kw_re.is_match(line) && !line.contains("unsafe_code") {
            occurrences.push(UnsafeOccurrence {
                file_path: rel_path.clone(),
                line_num,
                crate_name: crate_name.clone(),
                kind: UnsafeKind::Keyword,
            });
        }
    }

    // Check Rule 2: Non-island lib.rs must contain `#![forbid(unsafe_code)]`
    if file_path.file_name().and_then(|s| s.to_str()) == Some("lib.rs")
        && !ALLOWED_ISLANDS.contains(&crate_name.as_str())
    {
        let forbid_re = regex::Regex::new(r"#!\s*\[\s*forbid\s*\(\s*unsafe_code\s*\)\s*\]").unwrap();
        if !forbid_re.is_match(&stripped) {
            occurrences.push(UnsafeOccurrence {
                file_path: rel_path,
                line_num: 1,
                crate_name,
                kind: UnsafeKind::MissingForbid,
            });
        }
    }

    occurrences
}

pub struct UnsafeIslandsResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn run_check_unsafe_islands_at(
    root: &Path,
    strict: bool,
) -> Result<UnsafeIslandsResult, String> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let mut rs_files = Vec::new();
    for sub in &["crates", "xtask", "benchmarks"] {
        let dir = root.join(sub);
        if dir.exists() {
            for entry in WalkDir::new(dir)
                .into_iter()
                .filter_entry(|e| e.file_name() != "target")
                .filter_map(|e| e.ok())
            {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rs") {
                    rs_files.push(p.to_path_buf());
                }
            }
        }
    }

    for file in rs_files {
        let occurrences = scan_file_for_unsafe(&file, root);
        for occ in occurrences {
            let is_allowed_island = ALLOWED_ISLANDS.contains(&occ.crate_name.as_str());
            let is_transition_island = PHASE_0R_TRANSITION_ISLANDS.contains(&occ.crate_name.as_str());

            match occ.kind {
                UnsafeKind::Keyword => {
                    if !is_allowed_island {
                        let msg = format!(
                            "{}:{} — `unsafe` keyword in non-island crate '{}'",
                            occ.file_path, occ.line_num, occ.crate_name
                        );
                        if is_transition_island && !strict {
                            warnings.push(format!("TRANSITION-UNSAFE: {}", msg));
                        } else {
                            errors.push(msg);
                        }
                    }
                }
                UnsafeKind::AttributeAllow => {
                    if !is_allowed_island {
                        let msg = format!(
                            "{}:{} — `#![allow(unsafe_code)]` in non-island crate '{}'",
                            occ.file_path, occ.line_num, occ.crate_name
                        );
                        if strict {
                            errors.push(msg);
                        } else {
                            warnings.push(format!("ATTRIBUTE-ALLOW: {}", msg));
                        }
                    }
                }
                UnsafeKind::MissingForbid => {
                    let msg = format!(
                        "{} — missing `#![forbid(unsafe_code)]` in non-island lib.rs of crate '{}'",
                        occ.file_path, occ.crate_name
                    );
                    if strict {
                        errors.push(msg);
                    } else {
                        warnings.push(format!("MISSING-FORBID: {}", msg));
                    }
                }
            }
        }
    }

    errors.sort();
    warnings.sort();

    Ok(UnsafeIslandsResult { errors, warnings })
}

pub fn run_check_unsafe_islands(strict: bool) -> Result<bool, String> {
    println!("=== Running xtask check-unsafe-islands (strict={}) ===", strict);
    let root = crate::find_root_dir();
    let res = run_check_unsafe_islands_at(&root, strict)?;

    if !res.warnings.is_empty() {
        println!("⚠️ check-unsafe-islands warning(s):");
        for w in &res.warnings {
            println!("  {}", w);
        }
    }

    if !res.errors.is_empty() {
        eprintln!(
            "❌ check-unsafe-islands failed: {} error(s) found:",
            res.errors.len()
        );
        for e in &res.errors {
            eprintln!("  {}", e);
        }
        return Ok(false);
    }

    println!("✅ check-unsafe-islands: 0 errors.");
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_allowed_island_keyword_permitted() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let crate_dir = repo_root.join("crates/memfuse-sys/src");
        fs::create_dir_all(&crate_dir).unwrap();
        let file = crate_dir.join("lib.rs");

        fs::write(&file, "pub unsafe fn f() {}\n").unwrap();

        let occs = scan_file_for_unsafe(&file, repo_root);
        assert_eq!(occs.len(), 1);
        assert_eq!(occs[0].crate_name, "memfuse-sys");
        assert_eq!(occs[0].kind, UnsafeKind::Keyword);

        let res = run_check_unsafe_islands_at(repo_root, true).unwrap();
        assert!(res.errors.is_empty());
    }

    #[test]
    fn test_non_island_keyword_detected() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let crate_dir = repo_root.join("crates/memfuse-core/src");
        fs::create_dir_all(&crate_dir).unwrap();
        let file = crate_dir.join("lib.rs");

        fs::write(&file, "#![forbid(unsafe_code)]\npub unsafe fn f() {}\n").unwrap();

        let res = run_check_unsafe_islands_at(repo_root, true).unwrap();
        assert_eq!(res.errors.len(), 1);
        assert!(res.errors[0].contains("memfuse-core"));
    }

    #[test]
    fn test_transition_island_warns_in_default_fails_in_strict() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path();
        let crate_dir = repo_root.join("crates/memfuse-store/src");
        fs::create_dir_all(&crate_dir).unwrap();
        let file = crate_dir.join("lib.rs");

        fs::write(&file, "#![forbid(unsafe_code)]\npub unsafe fn f() {}\n").unwrap();

        let res_default = run_check_unsafe_islands_at(repo_root, false).unwrap();
        assert!(res_default.errors.is_empty());
        assert_eq!(res_default.warnings.len(), 1);

        let res_strict = run_check_unsafe_islands_at(repo_root, true).unwrap();
        assert_eq!(res_strict.errors.len(), 1);
    }
}
