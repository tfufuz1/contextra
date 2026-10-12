// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Gate check-loom-mocks — scannt crates/*/tests/loom*.rs auf Muster Mock|mirror.
// INVARIANTEN: Loom-Tests müssen echte Produktionsstrukturen testen, nicht Mock- oder Mirror-Abstraktionen.

use std::collections::HashSet;
use std::fs;
use walkdir::WalkDir;

/// Allowlist of legacy loom test files containing `Mock` or `mirror`.
/// New loom tests MUST test real production structs under Loom rather than mock/mirror implementations.
pub const ALLOWLIST: &[&str] = &[
    "crates/contextra-crypto/tests/loom_eviction_worker_shutdown_race.rs",
    "crates/contextra-crypto/tests/loom_kv_deferred_zeroize.rs",
    "crates/contextra-mvcc/tests/loom_floor.rs",
    "crates/contextra-mvcc/tests/loom_snapshot_registry.rs",
];

pub fn run_check_loom_mocks(_args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let allowlist_set: HashSet<&str> = ALLOWLIST.iter().copied().collect();

    let crates_dir = root.join("crates");
    let mut violations = Vec::new();

    if crates_dir.exists() {
        for entry in WalkDir::new(&crates_dir)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if !file_name.starts_with("loom") || !file_name.ends_with(".rs") {
                continue;
            }

            let rel_path = match path.strip_prefix(&root) {
                Ok(p) => p.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let has_mock = content.contains("Mock");
            let has_mirror = content.contains("mirror");

            if (has_mock || has_mirror) && !allowlist_set.contains(rel_path.as_str()) {
                violations.push((rel_path, has_mock, has_mirror));
            }
        }
    }

    if !violations.is_empty() {
        eprintln!(
            "❌ check-loom-mocks failed: {} unapproved loom mock/mirror file(s) found:",
            violations.len()
        );
        for (file_path, has_mock, has_mirror) in &violations {
            let mut patterns = Vec::new();
            if *has_mock {
                patterns.push("Mock");
            }
            if *has_mirror {
                patterns.push("mirror");
            }
            eprintln!("  {} (patterns: {})", file_path, patterns.join(", "));
        }
        eprintln!("Loom tests should exercise real production code without mock or mirror abstractions.");
        return 1;
    }

    println!("✅ check-loom-mocks: no unapproved loom mock/mirror patterns found.");
    0
}
