//! Harness Modul: konsolidierter Doktrin-Report (Zero-Panic, Unsafe-Inseln, Allow-Overrides, Status-Widerspruch) (doctrine-scan).

use crate::check_unsafe_islands;
use crate::panic_inventory;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctrineFinding {
    pub category: String, // "panic" | "unsafe" | "allow_override" | "status_contradiction"
    pub id: String,
    pub severity: String, // "error" | "warning" | "info"
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoctrineScanOutput {
    pub gate: String,
    pub status: String, // "pass" | "fail"
    pub summary: String,
    pub findings: Vec<DoctrineFinding>,
}

pub fn run_doctrine_scan(args: &[String]) -> i32 {
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut json = false;
    let mut crate_filter: Option<String> = None;

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
            "--crate" => {
                if i + 1 < args.len() {
                    crate_filter = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if !root.join("capabilities.toml").exists() {
        if let Ok(git_output) = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(&root)
            .output()
        {
            if git_output.status.success() {
                let top = String::from_utf8_lossy(&git_output.stdout)
                    .trim()
                    .to_string();
                if !top.is_empty() {
                    let top_path = PathBuf::from(top);
                    if top_path.join("capabilities.toml").exists() {
                        root = top_path;
                    }
                }
            }
        }
    }

    let mut findings: Vec<DoctrineFinding> = Vec::new();

    // 1. Panic findings
    let panic_entries = match panic_inventory::run_panic_inventory_in_root(
        &root,
        crate_filter.as_deref(),
        false,
        false,
    ) {
        Ok(entries) => entries,
        Err(e) => {
            let out = DoctrineScanOutput {
                gate: "doctrine-scan".to_string(),
                status: "fail".to_string(),
                summary: format!("Panic inventory failed: {}", e),
                findings: vec![],
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            } else {
                eprintln!("Panic inventory failed: {}", e);
            }
            return 2;
        }
    };

    for entry in panic_entries {
        findings.push(DoctrineFinding {
            category: "panic".to_string(),
            id: format!("PANIC_{}", entry.kind.as_str().to_uppercase()),
            severity: if entry.is_test_code {
                "info".to_string()
            } else {
                "warning".to_string()
            },
            file: entry.file,
            line: entry.line,
            message: format!("{} ({})", entry.context, entry.kind.as_str()),
            fix: "Replace unwrap/expect/panic with safe error handling".to_string(),
        });
    }

    // 2. Unsafe findings
    let scan_dir = if let Some(ref c) = crate_filter {
        root.join("crates").join(c)
    } else {
        root.join("crates")
    };

    if scan_dir.exists() {
        for entry in walkdir::WalkDir::new(&scan_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|s| s.to_str()) == Some("rs")
            {
                let rel = match entry.path().strip_prefix(&root) {
                    Ok(p) => p.to_string_lossy().replace('\\', "/"),
                    Err(_) => entry.path().to_string_lossy().replace('\\', "/"),
                };
                if rel.contains("/src/") || rel.starts_with("crates/") {
                    let occurrences =
                        check_unsafe_islands::scan_file_for_unsafe(entry.path(), &root);
                    for occ in occurrences {
                        let is_allowed = check_unsafe_islands::ALLOWED_ISLANDS
                            .contains(&occ.crate_name.as_str());
                        findings.push(DoctrineFinding {
                            category: "unsafe".to_string(),
                            id: "UNSAFE_CODE".to_string(),
                            severity: if is_allowed {
                                "info".to_string()
                            } else {
                                "error".to_string()
                            },
                            file: occ.file_path,
                            line: occ.line_num,
                            message: format!(
                                "Unsafe occurrence in crate '{}' ({:?})",
                                occ.crate_name, occ.kind
                            ),
                            fix: if is_allowed {
                                "Ensure // SAFETY: comment is present".to_string()
                            } else {
                                "Remove unsafe code or move to allowed unsafe island".to_string()
                            },
                        });
                    }
                }
            }
        }
    }

    // 3. Allow-Override findings
    let allow_regex = Regex::new(r"#!?\[allow\(([^)]*)\)\]").unwrap();
    let forbidden_lints: HashSet<&str> = [
        "unwrap_used",
        "expect_used",
        "panic",
        "unsafe_code",
        "todo",
        "unimplemented",
    ]
    .into_iter()
    .collect();

    let scan_roots = if let Some(ref c) = crate_filter {
        vec![root.join("crates").join(c)]
    } else {
        vec![
            root.join("crates"),
            root.join("xtask"),
            root.join("benchmarks"),
        ]
    };

    for sr in scan_roots {
        if !sr.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&sr)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|s| s.to_str()) == Some("rs")
            {
                let content = match fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let rel = match entry.path().strip_prefix(&root) {
                    Ok(p) => p.to_string_lossy().replace('\\', "/"),
                    Err(_) => entry.path().to_string_lossy().replace('\\', "/"),
                };

                for (idx, line) in content.lines().enumerate() {
                    for cap in allow_regex.captures_iter(line) {
                        if let Some(inner) = cap.get(1) {
                            for raw_item in inner.as_str().split(',') {
                                let trimmed = raw_item.trim();
                                let lint_name = trimmed.strip_prefix("clippy::").unwrap_or(trimmed);
                                if forbidden_lints.contains(lint_name) {
                                    findings.push(DoctrineFinding {
                                        category: "allow_override".to_string(),
                                        id: format!("ALLOW_OVERRIDE_{}", lint_name.to_uppercase()),
                                        severity: "warning".to_string(),
                                        file: rel.clone(),
                                        line: idx + 1,
                                        message: format!(
                                            "Allow override for security-relevant lint '{}'",
                                            lint_name
                                        ),
                                        fix: "Remove #[allow(...)] attribute and handle errors/safety explicitly"
                                            .to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 4 & 5. Status Correlation & Status Contradiction
    let caps_path = root.join("capabilities.toml");
    let mut completed_crates: HashSet<String> = HashSet::new();

    if caps_path.exists() {
        if let Ok(content) = fs::read_to_string(&caps_path) {
            if let Ok(toml_val) = toml::from_str::<serde_json::Value>(&content) {
                if let Some(crates_map) = toml_val.get("crates").and_then(|v| v.as_object()) {
                    for (c_name, c_obj) in crates_map {
                        if let Some(ref cf) = crate_filter {
                            if c_name != cf {
                                continue;
                            }
                        }

                        let status_val = c_obj
                            .get("maturity")
                            .or_else(|| c_obj.get("status"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_lowercase();

                        if matches!(
                            status_val.as_str(),
                            "stable" | "completed" | "fertig" | "done" | "production" | "prod"
                        ) {
                            completed_crates.insert(c_name.clone());
                        }
                    }
                }
            }
        }
    }

    // Group panic and unsafe findings by crate name
    let mut crates_with_doctrine_issues: HashSet<String> = HashSet::new();
    for finding in &findings {
        if finding.category == "panic" || finding.category == "unsafe" {
            // Extract crate name
            if let Some(c_name) = check_unsafe_islands::extract_crate_name_from_path(&finding.file)
            {
                crates_with_doctrine_issues.insert(c_name);
            } else {
                // Fallback: parse file path
                let parts: Vec<&str> = finding.file.split('/').collect();
                if parts.first() == Some(&"crates") && parts.len() > 1 {
                    if let Some(c) = parts.get(1) {
                        crates_with_doctrine_issues.insert(c.to_string());
                    }
                }
            }
        }
    }

    // Generate status_contradiction findings
    for completed_crate in &completed_crates {
        if crates_with_doctrine_issues.contains(completed_crate) {
            findings.push(DoctrineFinding {
                category: "status_contradiction".to_string(),
                id: "STATUS_CONTRADICTION".to_string(),
                severity: "error".to_string(),
                file: format!("crates/{}", completed_crate),
                line: 1,
                message: format!(
                    "Crate '{}' is marked as completed/stable in capabilities.toml but contains panic or unsafe findings",
                    completed_crate
                ),
                fix: "Resolve panic/unsafe findings in completed crate or update maturity status in capabilities.toml"
                    .to_string(),
            });
        }
    }

    // Gate Status evaluation
    let has_unsafe_error = findings
        .iter()
        .any(|f| f.category == "unsafe" && f.severity == "error");
    let has_status_contradiction = findings
        .iter()
        .any(|f| f.category == "status_contradiction");

    let gate_failed = has_unsafe_error || has_status_contradiction;
    let status = if gate_failed { "fail" } else { "pass" };

    let panic_count = findings.iter().filter(|f| f.category == "panic").count();
    let unsafe_count = findings.iter().filter(|f| f.category == "unsafe").count();
    let allow_count = findings
        .iter()
        .filter(|f| f.category == "allow_override")
        .count();
    let contradiction_count = findings
        .iter()
        .filter(|f| f.category == "status_contradiction")
        .count();

    let summary = format!(
        "Doctrine scan completed: {} findings total ({} panic, {} unsafe, {} allow_override, {} status_contradiction)",
        findings.len(),
        panic_count,
        unsafe_count,
        allow_count,
        contradiction_count
    );

    let output = DoctrineScanOutput {
        gate: "doctrine-scan".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        findings: findings.clone(),
    };

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    } else {
        println!("=== Doctrine Scan: {} ===", status.to_uppercase());
        println!("{}", summary);
        for f in &findings {
            println!(
                "  [{}] ({}) {}:{} - {}",
                f.severity.to_uppercase(),
                f.category,
                f.file,
                f.line,
                f.message
            );
        }
    }

    if gate_failed {
        1
    } else {
        0
    }
}
