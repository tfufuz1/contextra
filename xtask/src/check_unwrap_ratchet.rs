// xtask/src/check_unwrap_ratchet.rs
//
// IP-12: .unwrap-baseline.json Ratchet Check
// Scans production code for .unwrap() and .expect() occurrences,
// compares against .unwrap-baseline.json, fails if new unwraps or count increases occur.
// Supports --update flag to update baseline when explicitly requested.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::UnwrapBaselineEntry;

pub fn fnv1a_hash(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

pub fn scan_unwrap_expect_occurrences(root: &Path) -> Result<Vec<UnwrapBaselineEntry>, String> {
    let mut entries = Vec::new();

    let scan_dir = if root.join("crates").exists() {
        root.join("crates")
    } else {
        root.to_path_buf()
    };

    let mut walker_paths = Vec::new();
    collect_rs_files(&scan_dir, &mut walker_paths)?;

    for path in walker_paths {
        let path_str = match path.strip_prefix(root) {
            Ok(p) => match p.to_str() {
                Some(s) => s.replace('\\', "/"),
                None => continue,
            },
            Err(_) => continue,
        };

        // Exclude tests, benches, examples
        if path_str.contains("/tests/")
            || path_str.contains("/benches/")
            || path_str.contains("/examples/")
            || path_str.ends_with("_test.rs")
            || path_str.ends_with("_tests.rs")
        {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }

            if line.contains(".unwrap()") || line.contains(".expect(") {
                let start = if idx > 0 { idx - 1 } else { 0 };
                let end = std::cmp::min(idx + 1, lines.len().saturating_sub(1));
                let window_str = lines[start..=end].join("\n");
                let hash = fnv1a_hash(&window_str);

                entries.push(UnwrapBaselineEntry {
                    file: path_str.clone(),
                    hash,
                });
            }
        }
    }

    entries.sort();
    entries.dedup();
    Ok(entries)
}

fn collect_rs_files(dir: &Path, acc: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let read_dir = fs::read_dir(dir).map_err(|e| format!("Failed to read dir {dir:?}: {e}"))?;
    for entry_res in read_dir {
        let entry = entry_res.map_err(|e| format!("Dir entry error in {dir:?}: {e}"))?;
        let path = entry.path();
        if path.is_dir() {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if file_name != "target" && file_name != ".git" {
                collect_rs_files(&path, acc)?;
            }
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            acc.push(path);
        }
    }
    Ok(())
}

pub fn run_check_unwrap_ratchet(root: &Path, update_mode: bool) -> bool {
    println!("=== Running xtask check-unwrap-ratchet (IP-12) ===");

    let baseline_path = root.join(".unwrap-baseline.json");
    let current_entries = match scan_unwrap_expect_occurrences(root) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("❌ Failed to scan unwrap occurrences: {e}");
            return false;
        }
    };

    if update_mode {
        println!("Updating .unwrap-baseline.json ratchet baseline (--update mode)...");
        return save_baseline(&baseline_path, &current_entries);
    }

    if !baseline_path.exists() {
        eprintln!("❌ .unwrap-baseline.json missing! Run `cargo xtask check-unwrap-ratchet --update` to initialize.");
        return false;
    }

    let baseline_content = match fs::read_to_string(&baseline_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to read .unwrap-baseline.json: {e}");
            return false;
        }
    };

    let baseline_entries: Vec<UnwrapBaselineEntry> = match serde_json::from_str(&baseline_content) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("❌ Failed to parse .unwrap-baseline.json: {e}");
            return false;
        }
    };

    let baseline_set: HashSet<(&str, &str)> = baseline_entries
        .iter()
        .map(|e| (e.file.as_str(), e.hash.as_str()))
        .collect();

    let mut new_unwraps = Vec::new();
    for entry in &current_entries {
        if !baseline_set.contains(&(entry.file.as_str(), entry.hash.as_str())) {
            new_unwraps.push(entry);
        }
    }

    if !new_unwraps.is_empty() {
        eprintln!(
            "❌ [IP-12 RATCHET VIOLATION]: Detected {} new .unwrap()/.expect() call(s) not in baseline!",
            new_unwraps.len()
        );
        for entry in &new_unwraps {
            eprintln!("  - {} (hash: {})", entry.file, entry.hash);
        }
        eprintln!("💡 Production code must not introduce new .unwrap() or .expect() calls.");
        return false;
    }

    println!(
        "Current unwrap/expect count: {} | Baseline count: {}",
        current_entries.len(),
        baseline_entries.len()
    );

    if current_entries.len() < baseline_entries.len() {
        println!(
            "🎉 IMPROVEMENT: Total unwrap/expect count decreased from {} to {}!",
            baseline_entries.len(),
            current_entries.len()
        );
        println!("💡 Run `cargo xtask check-unwrap-ratchet --update` to lock in this baseline improvement.");
    }

    println!("✅ IP-12 Unwrap Baseline Ratchet check passed cleanly.");
    true
}

fn save_baseline(path: &Path, entries: &[UnwrapBaselineEntry]) -> bool {
    let json = match serde_json::to_string_pretty(entries) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("❌ Failed to serialize baseline: {e}");
            return false;
        }
    };

    if let Err(e) = fs::write(path, json) {
        eprintln!("❌ Failed to write .unwrap-baseline.json: {e}");
        return false;
    }

    println!(
        "Saved baseline to {} with {} entries.",
        path.display(),
        entries.len()
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ratchet_allows_improvement() {
        let dir = match tempdir() {
            Ok(d) => d,
            Err(_) => return,
        };
        let root = dir.path();
        let crates_dir = root.join("crates/memfuse-core/src");
        if fs::create_dir_all(&crates_dir).is_err() {
            return;
        }

        let file1 = crates_dir.join("lib.rs");
        if fs::write(
            &file1,
            "pub fn a() { let x: Option<i32> = None; x.unwrap(); }\n",
        )
        .is_err()
        {
            return;
        }

        // Initialize baseline
        assert!(run_check_unwrap_ratchet(root, true));

        // Remove unwrap
        if fs::write(
            &file1,
            "pub fn a() { let x: Option<i32> = None; if let Some(_) = x {} }\n",
        )
        .is_err()
        {
            return;
        }

        // Should pass check
        assert!(run_check_unwrap_ratchet(root, false));
    }

    #[test]
    fn test_ratchet_rejects_new_unwrap() {
        let dir = match tempdir() {
            Ok(d) => d,
            Err(_) => return,
        };
        let root = dir.path();
        let crates_dir = root.join("crates/memfuse-core/src");
        if fs::create_dir_all(&crates_dir).is_err() {
            return;
        }

        let file1 = crates_dir.join("lib.rs");
        if fs::write(&file1, "pub fn a() { }\n").is_err() {
            return;
        }

        // Initialize baseline
        assert!(run_check_unwrap_ratchet(root, true));

        // Add new unwrap
        if fs::write(
            &file1,
            "pub fn a() { let x: Option<i32> = None; x.unwrap(); }\n",
        )
        .is_err()
        {
            return;
        }

        // Should fail check
        assert!(!run_check_unwrap_ratchet(root, false));
    }

    #[test]
    fn test_ratchet_zero_panic_dogfooding() {
        // Confirm check_unwrap_ratchet.rs source file contains zero .unwrap() and .expect() calls
        let source = include_str!("check_unwrap_ratchet.rs");
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            assert!(
                !trimmed.contains(".unwrap()"),
                "check_unwrap_ratchet.rs must not contain .unwrap()"
            );
            assert!(
                !trimmed.contains(".expect("),
                "check_unwrap_ratchet.rs must not contain .expect()"
            );
        }
    }
}
