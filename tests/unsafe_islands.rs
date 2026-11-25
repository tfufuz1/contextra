// FILE-CONTEXT
// STAND: 2026-09-19T20:22:00Z (SESSION: 01c5be8b)
// ZWECK: Unsafe Islands inventory verification (GESAMTSPEZIFIKATION §0.2, §0.4).
// INVARIANTEN: Exactly 3 Unsafe Islands (contextra-simd, contextra-sys, contextra-wire).
// Temporary transition allowance until Phase 1c for legacy store, index, db, and crypto test.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const ALLOWED_UNSAFE_ISLANDS: &[&str] = &[
    "contextra-simd",
    "contextra-sys",
    "contextra-wire",
];

pub const TRANSITION_ALLOWED_CRATES: &[&str] = &[
    "contextra-vector",        // SIMD + Mmap, being moved to contextra-simd / contextra-sys in Phase 1c
    "contextra-store",        // Win32 ACL, being moved to contextra-sys in Phase 1c
    "contextra-db",           // volatile-vault mlock, being moved to contextra-sys in Phase 1c
    "contextra-crypto",       // test-only Zeroize drop semantics verification
];

pub fn check_unsafe_islands(root: &Path) -> (bool, Vec<String>) {
    let crates_dir = root.join("crates");
    let mut violations = Vec::new();
    let islands: HashSet<&str> = ALLOWED_UNSAFE_ISLANDS.iter().copied().collect();
    let transition: HashSet<&str> = TRANSITION_ALLOWED_CRATES.iter().copied().collect();

    let entries = match fs::read_dir(&crates_dir) {
        Ok(e) => e,
        Err(err) => return (false, vec![format!("Failed to read crates dir: {}", err)]),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let crate_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        let is_island = islands.contains(crate_name);
        let is_transition = transition.contains(crate_name);

        let src_dir = path.join("src");
        if !src_dir.exists() {
            continue;
        }

        let lib_rs = src_dir.join("lib.rs");
        if lib_rs.exists() {
            if let Ok(lib_content) = fs::read_to_string(&lib_rs) {
                let has_allow_unsafe = lib_content.contains("#![allow(unsafe_code)]");
                if has_allow_unsafe && !is_island && !is_transition {
                    violations.push(format!(
                        "Crate '{}' has '#![allow(unsafe_code)]' but is not an approved Unsafe Island ({:?})",
                        crate_name, ALLOWED_UNSAFE_ISLANDS
                    ));
                }
            }
        }

        // Check for unsafe blocks in non-island, non-transition crates
        if !is_island && !is_transition {
            for file_entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
                let p = file_entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rs") {
                    if let Ok(file_content) = fs::read_to_string(p) {
                        for (line_no, line) in file_content.lines().enumerate() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("//") {
                                continue;
                            }
                            if trimmed.contains("unsafe ") || trimmed.starts_with("unsafe{") || trimmed == "unsafe" {
                                violations.push(format!(
                                    "Forbidden unsafe found in non-island crate '{}' at {}:{}",
                                    crate_name,
                                    p.strip_prefix(root).unwrap_or(p).display(),
                                    line_no + 1
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    println!("=== Contextra Unsafe Islands Inventory Verification ===");
    println!("Approved Islands: {:?}", ALLOWED_UNSAFE_ISLANDS);
    println!("Transition Crates (Phase 0R..1c): {:?}", TRANSITION_ALLOWED_CRATES);

    if !violations.is_empty() {
        for v in &violations {
            eprintln!("❌ {}", v);
        }
        (false, violations)
    } else {
        println!("✅ Unsafe Islands Inventory verified (0 illegal unsafe occurrences found)");
        (true, Vec::new())
    }
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (success, _) = check_unsafe_islands(&root);
    if !success {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsafe_islands_inventory() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let (success, violations) = check_unsafe_islands(&root);
        assert!(success, "Unsafe islands check failed with violations: {:?}", violations);
    }
}
