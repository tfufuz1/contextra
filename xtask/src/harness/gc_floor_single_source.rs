//! Harness gate forbidding calls to `min_active_seqno()` outside `crates/contextra-mvcc/src/floor.rs`.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct GcFloorSingleSourceFinding {
    pub file_path: String,
    pub line_num: usize,
    pub line_content: String,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct GcFloorSingleSourceBaselineFile {
    pub occurrences: BTreeMap<String, bool>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GcFloorSingleSourceOutput {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<GcFloorSingleSourceFinding>,
}

pub fn run_gc_floor_single_source(args: &[String]) -> i32 {
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut json = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                if i + 1 < args.len() {
                    root = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--json" => {
                json = true;
            }
            _ => {}
        }
        i += 1;
    }

    let mut raw_findings = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target" && name != ".git" && name != ".cargo" && name != "node_modules"
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let rel_path = path
                .strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            if rel_path.starts_with("xtask/") {
                continue;
            }

            // Allowed single source location: crates/contextra-mvcc/src/floor.rs
            if rel_path == "crates/contextra-mvcc/src/floor.rs" {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                for (idx, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("//") {
                        continue;
                    }
                    if trimmed.contains("min_active_seqno()") {
                        raw_findings.push(GcFloorSingleSourceFinding {
                            file_path: rel_path.clone(),
                            line_num: idx + 1,
                            line_content: trimmed.to_string(),
                        });
                    }
                }
            }
        }
    }

    let baseline_path = root.join("governance/gc-floor-single-source-baseline.toml");
    let baseline_set: HashSet<String> = if baseline_path.exists() {
        if let Ok(content) = fs::read_to_string(&baseline_path) {
            if let Ok(parsed) = toml::from_str::<GcFloorSingleSourceBaselineFile>(&content) {
                parsed.occurrences.into_keys().collect()
            } else {
                HashSet::new()
            }
        } else {
            HashSet::new()
        }
    } else {
        HashSet::new()
    };

    let new_findings: Vec<GcFloorSingleSourceFinding> = raw_findings
        .into_iter()
        .filter(|f| {
            let key = format!("{}:{}:{}", f.file_path, f.line_num, f.line_content);
            !baseline_set.contains(&key)
        })
        .collect();

    let has_errors = !new_findings.is_empty();
    let status = if has_errors { "fail" } else { "pass" };
    let summary = format!(
        "gc-floor-single-source completed: {} new violation(s)",
        new_findings.len()
    );

    let output = GcFloorSingleSourceOutput {
        gate: "gc-floor-single-source".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        findings: new_findings.clone(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!("{}", summary);
        for f in &new_findings {
            println!("  {}:{} — {}", f.file_path, f.line_num, f.line_content);
        }
    }

    if status == "fail" {
        1
    } else {
        0
    }
}
