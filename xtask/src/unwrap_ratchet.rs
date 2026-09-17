// xtask/src/unwrap_ratchet.rs
//
// IP-12: .unwrap-baseline.json Ratchet Tool
// Enforces zero-net-growth for `.unwrap()` / `.expect()` occurrences in production code.
// Excludes test code (files in tests/ / benches/, _test.rs, #[cfg(test)], mod tests)
// and explicitly marked checked exceptions (// unwrap-ok, // allow-unwrap, // safe unwrap, etc.).
//
// Zero-Panic Dogfooding: This file itself MUST contain zero `.unwrap()` or `.expect()` calls.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct UnwrapBaselineEntry {
    pub file: String,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct UnwrapOccurrence {
    pub file: String,
    pub line_num: usize,
    pub content: String,
    pub hash: String,
}

/// FNV-1a 64-bit hash function without unwraps.
pub fn fnv1a_hash(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

/// Checks whether a given path corresponds to a test file or benchmark.
pub fn is_test_file(path: &Path, rel_path: &str) -> bool {
    if rel_path.contains("/tests/") || rel_path.contains("/benches/") {
        return true;
    }
    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
        if file_name == "tests.rs"
            || file_name == "test.rs"
            || file_name.ends_with("_test.rs")
            || file_name.ends_with("_tests.rs")
        {
            return true;
        }
    }
    false
}

/// Checks whether a line or its preceding line contains an explicit checked exception marker.
pub fn is_explicit_exception(line: &str, prev_line: Option<&str>) -> bool {
    let markers = [
        "unwrap-ok",
        "allow-unwrap",
        "safe unwrap",
        "expect-ok",
        "allow(clippy::unwrap_used)",
        "SAFETY:",
    ];

    for marker in &markers {
        if line.contains(marker) {
            return true;
        }
        if let Some(prev) = prev_line {
            if prev.contains(marker) {
                return true;
            }
        }
    }

    false
}

/// Scans production code under `root/crates` for `.unwrap()` and `.expect(` occurrences.
pub fn scan_prod_unwrap_occurrences(root: &Path) -> Vec<UnwrapOccurrence> {
    let mut occurrences = Vec::new();
    let scan_dir = if root.join("crates").exists() {
        root.join("crates")
    } else {
        root.to_path_buf()
    };

    for entry in WalkDir::new(&scan_dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        if is_test_file(path, &rel_path) {
            continue;
        }

        if let Ok(content) = fs::read_to_string(path) {
            let lines: Vec<&str> = content.lines().collect();
            let mut in_cfg_test_mod = false;

            for (idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                if trimmed.contains("#[cfg(test)]")
                    || trimmed.starts_with("mod tests")
                    || trimmed.starts_with("mod test")
                {
                    in_cfg_test_mod = true;
                }

                if in_cfg_test_mod {
                    continue;
                }

                if (trimmed.contains(".unwrap()") || trimmed.contains(".expect("))
                    && !trimmed.starts_with("//")
                {
                    let prev_line = if idx > 0 { Some(lines[idx - 1]) } else { None };
                    if is_explicit_exception(trimmed, prev_line) {
                        continue;
                    }

                    let line_num = idx + 1;
                    let start = if idx > 0 { idx - 1 } else { 0 };
                    let end = std::cmp::min(idx + 1, lines.len().saturating_sub(1));
                    let window_str = lines[start..=end].join("\n");
                    let hash = fnv1a_hash(&window_str);

                    occurrences.push(UnwrapOccurrence {
                        file: rel_path.clone(),
                        line_num,
                        content: trimmed.to_string(),
                        hash,
                    });
                }
            }
        }
    }

    occurrences
}

/// Runs the ratchet check against `.unwrap-baseline.json`.
pub fn run_check_unwrap_ratchet(root: &Path, update_mode: bool) -> bool {
    println!(
        "=== Running xtask check-unwrap-ratchet (update_mode={}) ===",
        update_mode
    );

    let json_path = root.join(".unwrap-baseline.json");
    let current_occurrences = scan_prod_unwrap_occurrences(root);

    let mut current_entries: Vec<UnwrapBaselineEntry> = current_occurrences
        .iter()
        .map(|occ| UnwrapBaselineEntry {
            file: occ.file.clone(),
            hash: occ.hash.clone(),
        })
        .collect();

    current_entries.sort();
    current_entries.dedup();

    if update_mode {
        let json_content = match serde_json::to_string_pretty(&current_entries) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("❌ Failed to serialize baseline entries: {}", e);
                return false;
            }
        };

        if let Err(e) = fs::write(&json_path, json_content) {
            eprintln!("❌ Failed to write {}: {}", json_path.display(), e);
            return false;
        }

        println!(
            "🔒 Baseline updated: {} production unwrap/expect calls recorded.",
            current_entries.len()
        );
        return true;
    }

    if !json_path.exists() {
        eprintln!("❌ .unwrap-baseline.json missing at root!");
        eprintln!("💡 AUTOMATISCHE BEHEBUNG: cargo xtask check-unwrap-ratchet --update");
        return false;
    }

    let json_content = match fs::read_to_string(&json_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to read .unwrap-baseline.json: {}", e);
            return false;
        }
    };

    let baseline_entries: Vec<UnwrapBaselineEntry> = match serde_json::from_str(&json_content) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("❌ Failed to parse .unwrap-baseline.json: {}", e);
            return false;
        }
    };

    let baseline_set: HashSet<(&str, &str)> = baseline_entries
        .iter()
        .map(|e| (e.file.as_str(), e.hash.as_str()))
        .collect();

    let mut violations = Vec::new();
    for occ in &current_occurrences {
        if !baseline_set.contains(&(occ.file.as_str(), occ.hash.as_str())) {
            violations.push(occ);
        }
    }

    let current_count = current_occurrences.len();
    let baseline_count = baseline_entries.len();

    println!(
        "Current production unwraps: {} | Baseline: {}",
        current_count, baseline_count
    );

    if !violations.is_empty() || current_count > baseline_count {
        eprintln!("❌ [RATCHET VIOLATION]: Production unwrap count increased or new unapproved unwraps found!");
        for v in &violations {
            eprintln!(
                "  - {}:{} — new .unwrap()/.expect(): {}",
                v.file, v.line_num, v.content
            );
        }
        if current_count > baseline_count {
            eprintln!(
                "  - Total count increase: {} -> {} (+{} net growth)",
                baseline_count,
                current_count,
                current_count.saturating_sub(baseline_count)
            );
        }
        eprintln!(
            "💡 AUTOMATISCHE BEHEBUNG / RATCHET UPDATE: cargo xtask check-unwrap-ratchet --update"
        );
        return false;
    }

    if current_count < baseline_count {
        println!(
            "🎉 Improvement detected! Current production unwraps ({}) < baseline ({}).",
            current_count, baseline_count
        );
        println!(
            "💡 Run `cargo xtask check-unwrap-ratchet --update` to lock in the lower baseline!"
        );
    } else {
        println!(
            "✅ Ratchet check passed: production unwrap count ({}) within baseline.",
            current_count
        );
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_fnv1a_hash_deterministic() {
        let h1 = fnv1a_hash("test_string");
        let h2 = fnv1a_hash("test_string");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_is_test_file() {
        assert!(is_test_file(
            Path::new("crates/memfuse-core/tests/foo.rs"),
            "crates/memfuse-core/tests/foo.rs"
        ));
        assert!(is_test_file(
            Path::new("crates/memfuse-core/src/tests.rs"),
            "crates/memfuse-core/src/tests.rs"
        ));
        assert!(is_test_file(
            Path::new("crates/memfuse-core/src/lib_test.rs"),
            "crates/memfuse-core/src/lib_test.rs"
        ));
        assert!(!is_test_file(
            Path::new("crates/memfuse-core/src/lib.rs"),
            "crates/memfuse-core/src/lib.rs"
        ));
    }

    #[test]
    fn test_is_explicit_exception() {
        assert!(is_explicit_exception(
            "let x = foo().unwrap(); // unwrap-ok",
            None
        ));
        assert!(is_explicit_exception(
            "let x = foo().unwrap();",
            Some("// allow-unwrap")
        ));
        assert!(!is_explicit_exception("let x = foo().unwrap();", None));
    }

    #[test]
    fn test_check_unwrap_ratchet_flow() {
        let dir = match tempdir() {
            Ok(d) => d,
            Err(_) => return,
        };
        let root = dir.path();

        let crates_src = root.join("crates/foo/src");
        if fs::create_dir_all(&crates_src).is_err() {
            return;
        }
        let lib_rs = crates_src.join("lib.rs");

        if fs::write(&lib_rs, "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").is_err() {
            return;
        }

        // Update mode creates baseline
        assert!(run_check_unwrap_ratchet(root, true));
        // Check mode passes
        assert!(run_check_unwrap_ratchet(root, false));

        // Add an unwrap call
        if fs::write(
            &lib_rs,
            "pub fn add(a: i32, b: i32) -> i32 { Some(a + b).unwrap() }\n",
        )
        .is_err()
        {
            return;
        }

        // Check mode fails
        assert!(!run_check_unwrap_ratchet(root, false));
    }
}
