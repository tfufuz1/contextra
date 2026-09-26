//! Gate: check-duplicate-core-primitives
//! Erkenne Typen/Structs mit identischem Namen, die in MEHR ALS EINEM Crate
//! unabhängig voneinander definiert werden, und melde dies.

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimitiveDuplicate {
    pub symbol_name: String,
    pub symbol_kind: String, // "struct" or "type"
    pub occurrences: Vec<PrimitiveOccurrence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimitiveOccurrence {
    pub crate_name: String,
    pub file_path: String,
    pub line_number: usize,
}

/// Scannt alle Crate-Quellcodedateien (`crates/*/src/**/*.rs`) auf `struct` und `type`
/// Deklarationen und identifiziert Symbole, die in mehr als einem Crate unabhängig definiert sind.
pub fn scan_duplicate_core_primitives(
    root_dir: &Path,
) -> Result<Vec<PrimitiveDuplicate>, String> {
    let decl_re = Regex::new(
        r"^(?:pub(?:\([^)]+\))?\s+)?(?:async\s+|unsafe\s+)?(struct|type)\s+([A-Za-z_][A-Za-z0-9_]*)",
    )
    .map_err(|e| format!("Invalid regex: {}", e))?;

    let crates_dir = root_dir.join("crates");
    if !crates_dir.exists() {
        return Ok(Vec::new());
    }

    let mut occurrences_map: HashMap<(String, String), Vec<PrimitiveOccurrence>> = HashMap::new();

    for entry in WalkDir::new(&crates_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        // Only inspect production src files, ignore test files or benches inside crates
        let rel_path = match path.strip_prefix(root_dir) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => path.to_string_lossy().to_string(),
        };

        let normalized_path = rel_path.replace('\\', "/");
        let components: Vec<&str> = normalized_path.split('/').collect();
        if components.len() < 3 || components[0] != "crates" || components[2] != "src" {
            continue;
        }

        let crate_name = components[1].to_string();

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut brace_depth: i32 = 0;
        let mut in_test_module = false;

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("#[cfg(test)]") {
                in_test_module = true;
            }

            if brace_depth == 0 && !in_test_module && !trimmed.is_empty() {
                if let Some(caps) = decl_re.captures(trimmed) {
                    let kind = caps[1].to_string();
                    let symbol = caps[2].to_string();

                    // Filter out generic short or common standard aliases if any
                    if symbol != "_" && symbol != "Result" && symbol != "Option" {
                        let occ = PrimitiveOccurrence {
                            crate_name: crate_name.clone(),
                            file_path: rel_path.clone(),
                            line_number: line_num,
                        };
                        occurrences_map
                            .entry((kind, symbol))
                            .or_default()
                            .push(occ);
                    }
                }
            }

            // Track brace depth
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth = brace_depth.saturating_sub(1);
                    if brace_depth == 0 {
                        in_test_module = false;
                    }
                }
            }
        }
    }

    let mut duplicates = Vec::new();

    for ((symbol_kind, symbol_name), occs) in occurrences_map {
        // Group by crate name
        let mut by_crate: HashMap<String, Vec<PrimitiveOccurrence>> = HashMap::new();
        for occ in occs {
            by_crate.entry(occ.crate_name.clone()).or_default().push(occ);
        }

        if by_crate.len() > 1 {
            let mut all_occs = Vec::new();
            for (_c_name, mut c_occs) in by_crate {
                c_occs.sort_by(|a, b| a.file_path.cmp(&b.file_path).then_with(|| a.line_number.cmp(&b.line_number)));
                all_occs.extend(c_occs);
            }
            all_occs.sort_by(|a, b| a.crate_name.cmp(&b.crate_name).then_with(|| a.file_path.cmp(&b.file_path)));

            duplicates.push(PrimitiveDuplicate {
                symbol_name,
                symbol_kind,
                occurrences: all_occs,
            });
        }
    }

    duplicates.sort_by(|a, b| a.symbol_name.cmp(&b.symbol_name));

    Ok(duplicates)
}

fn find_root_dir() -> PathBuf {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut dir = PathBuf::from(manifest_dir);
        loop {
            let cargo_path = dir.join("Cargo.toml");
            if cargo_path.exists() {
                if let Ok(content) = fs::read_to_string(&cargo_path) {
                    if content.contains("[workspace]") && content.contains("members") {
                        return dir;
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
    }
    if let Ok(curr) = std::env::current_dir() {
        let mut dir = curr;
        loop {
            let cargo_path = dir.join("Cargo.toml");
            if cargo_path.exists() {
                if let Ok(content) = fs::read_to_string(&cargo_path) {
                    if content.contains("[workspace]") && content.contains("members") {
                        return dir;
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
    }
    PathBuf::from(".")
}

pub fn run_check_duplicate_core_primitives() -> Result<bool, String> {
    println!("=== Running xtask check-duplicate-core-primitives ===");

    let root_dir = find_root_dir();
    let duplicates = scan_duplicate_core_primitives(&root_dir)?;

    if duplicates.is_empty() {
        println!("✅ check-duplicate-core-primitives: Keine Crate-übergreifenden Duplikate gefunden.");
        return Ok(true);
    }

    println!(
        "⚠️ check-duplicate-core-primitives: {} Crate-übergreifende Primitiv-Duplikate gefunden:",
        duplicates.len()
    );

    for dup in &duplicates {
        let crates_involved: Vec<String> = dup
            .occurrences
            .iter()
            .map(|o| o.crate_name.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        println!(
            "  - [{}] '{}' in {} Crates ({}) gefunden:",
            dup.symbol_kind,
            dup.symbol_name,
            crates_involved.len(),
            crates_involved.join(", ")
        );

        for occ in &dup.occurrences {
            println!("      {}:{} (Crate: {})", occ.file_path, occ.line_number, occ.crate_name);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_duplicate_core_primitives_current_workspace() {
        let root = find_root_dir();
        let dups = scan_duplicate_core_primitives(&root).unwrap();
        println!("Dups found ({}):", dups.len());
        for d in &dups {
            println!("  - {}", d.symbol_name);
        }

        let kv_locks_dup = dups.iter().find(|d| d.symbol_name == "KvKeyLocks");
        assert!(
            kv_locks_dup.is_some(),
            "Expected KvKeyLocks duplicate across contextra-store and contextra-engine"
        );

        let dup = kv_locks_dup.unwrap();
        let crates: Vec<_> = dup.occurrences.iter().map(|o| o.crate_name.as_str()).collect();
        assert!(crates.contains(&"contextra-store"));
        assert!(crates.contains(&"contextra-engine"));
    }
}
