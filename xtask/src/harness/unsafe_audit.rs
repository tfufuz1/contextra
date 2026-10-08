//! Gate harness for auditing unsafe code and safety comments.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnsafeAuditFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UnsafeAuditOutput {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<UnsafeAuditFinding>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UnsafeSafetyBaselineFile {
    pub missing_safety_comments: BTreeMap<String, usize>,
}

pub struct UnsafeCrateInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_island: bool,
}

impl UnsafeCrateInfo {
    pub fn name_ref(&self) -> &str {
        &self.name
    }
}

pub fn unsafe_audit_get_islands_and_crates(
    root: &Path,
) -> Result<(Vec<String>, Vec<UnsafeCrateInfo>), String> {
    let caps_path = root.join("capabilities.toml");
    if !caps_path.exists() {
        return Err(format!(
            "capabilities.toml missing at {}",
            caps_path.display()
        ));
    }

    let content = fs::read_to_string(&caps_path)
        .map_err(|e| format!("failed to read capabilities.toml: {}", e))?;

    let val: serde_json::Value = toml::from_str(&content)
        .map_err(|e| format!("failed to parse capabilities.toml: {}", e))?;

    let mut islands = Vec::new();
    let mut all_crates = Vec::new();

    if let Some(crates_map) = val.get("crates").and_then(|v| v.as_object()) {
        for (name, obj) in crates_map {
            let is_island = obj
                .get("unsafe_island")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if is_island {
                islands.push(name.clone());
            }
            if let Some(rel_path) = obj.get("path").and_then(|p| p.as_str()) {
                all_crates.push(UnsafeCrateInfo {
                    name: name.clone(),
                    path: root.join(rel_path),
                    is_island,
                });
            }
        }
    }

    Ok((islands, all_crates))
}

pub fn unsafe_audit_scan_file(
    file_path: &Path,
    rel_path: &str,
    is_island: bool,
    findings: &mut Vec<UnsafeAuditFinding>,
    unsafe_counts: &mut BTreeMap<String, usize>,
) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let lines: Vec<&str> = content.lines().collect();
    let is_lib_or_main = rel_path.ends_with("/lib.rs") || rel_path.ends_with("/main.rs");

    if !is_island {
        if is_lib_or_main {
            let has_forbid = content.contains("#![forbid(unsafe_code)]")
                || content.contains("#![deny(unsafe_code)]");
            if !has_forbid {
                findings.push(UnsafeAuditFinding {
                    id: "MISSING_FORBID_UNSAFE".to_string(),
                    severity: "error".to_string(),
                    file: rel_path.to_string(),
                    line: 1,
                    message: format!(
                        "Non-island crate file {} lacks #![forbid(unsafe_code)] or #![deny(unsafe_code)]",
                        rel_path
                    ),
                    fix: "Add #![forbid(unsafe_code)] to crate root".to_string(),
                });
            }
        }

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("unsafe ") || trimmed == "unsafe" {
                if trimmed.starts_with("//") {
                    continue;
                }
                findings.push(UnsafeAuditFinding {
                    id: "UNSAFE_OUTSIDE_ISLAND".to_string(),
                    severity: "error".to_string(),
                    file: rel_path.to_string(),
                    line: idx + 1,
                    message:
                        "Unsafe code block/function used outside designated unsafe island crate"
                            .to_string(),
                    fix: "Move unsafe code to designated unsafe island or remove unsafe keyword"
                        .to_string(),
                });
            }
        }
    } else {
        let mut unsafe_sites = 0;
        for line in lines {
            let trimmed = line.trim();
            if (trimmed.contains("unsafe ") || trimmed == "unsafe") && !trimmed.starts_with("//") {
                unsafe_sites += 1;
            }
        }
        if unsafe_sites > 0 {
            *unsafe_counts.entry(rel_path.to_string()).or_insert(0) += unsafe_sites;
        }
    }
}

pub fn run_unsafe_audit(args: &[String]) -> i32 {
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut json = false;
    let mut run_miri = false;
    let mut _write_baseline = false;

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
            "--miri" => {
                run_miri = true;
            }
            "--write-baseline" => {
                _write_baseline = true;
            }
            _ => {}
        }
        i += 1;
    }

    let (islands, all_crates) = match unsafe_audit_get_islands_and_crates(&root) {
        Ok(c) => c,
        Err(e) => {
            let out = UnsafeAuditOutput {
                gate: "unsafe-audit".to_string(),
                status: "error".to_string(),
                summary: e,
                findings: vec![],
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            }
            return 2;
        }
    };

    let mut findings = Vec::new();
    let mut unsafe_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut missing_safety_counts: BTreeMap<String, usize> = BTreeMap::new();

    // Scan non-island crates for forbidden unsafe blocks / missing forbid lints, and count unsafe sites
    for crate_info in &all_crates {
        let src_dir = crate_info.path.join("src");
        if !src_dir.exists() {
            continue;
        }

        for entry in walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|s| s.to_str()) == Some("rs")
            {
                let rel = match entry.path().strip_prefix(&root) {
                    Ok(p) => p.to_string_lossy().to_string(),
                    Err(_) => entry.path().to_string_lossy().to_string(),
                };
                unsafe_audit_scan_file(
                    entry.path(),
                    &rel,
                    crate_info.is_island,
                    &mut findings,
                    &mut unsafe_counts,
                );
            }
        }
    }

    // Run cargo clippy -p <island> -- -D clippy::undocumented_unsafe_blocks for unsafe islands
    for island in &islands {
        let clippy_output = Command::new("cargo")
            .args([
                "clippy",
                "-p",
                island,
                "--message-format=json",
                "--",
                "-D",
                "clippy::undocumented_unsafe_blocks",
            ])
            .current_dir(&root)
            .output();

        if let Ok(output) = clippy_output {
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            for line in stdout_str.lines() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                    if val.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                        if let Some(msg) = val.get("message") {
                            let code = msg
                                .get("code")
                                .and_then(|c| c.get("code"))
                                .and_then(|s| s.as_str());
                            if code == Some("clippy::undocumented_unsafe_blocks") {
                                if let Some(spans) = msg.get("spans").and_then(|s| s.as_array()) {
                                    for span in spans {
                                        if span.get("is_primary").and_then(|b| b.as_bool())
                                            == Some(true)
                                        {
                                            if let Some(file_name) = span
                                                .get("file_name")
                                                .and_then(|f| f.as_str())
                                            {
                                                let norm_file = file_name.replace('\\', "/");
                                                *missing_safety_counts
                                                    .entry(norm_file)
                                                    .or_insert(0) += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let baseline_path = root.join("governance/unsafe-safety-baseline.toml");
    let baseline_map: BTreeMap<String, usize> = if baseline_path.exists() {
        if let Ok(content) = fs::read_to_string(&baseline_path) {
            if let Ok(parsed) = toml::from_str::<UnsafeSafetyBaselineFile>(&content) {
                parsed.missing_safety_comments
            } else {
                BTreeMap::new()
            }
        } else {
            BTreeMap::new()
        }
    } else {
        // Enforce baseline existence requirement! Do NOT write baseline silently.
        findings.push(UnsafeAuditFinding {
            id: "MISSING_BASELINE_FILE".to_string(),
            severity: "error".to_string(),
            file: "governance/unsafe-safety-baseline.toml".to_string(),
            line: 0,
            message: "Required baseline file governance/unsafe-safety-baseline.toml is missing!"
                .to_string(),
            fix: "Create governance/unsafe-safety-baseline.toml".to_string(),
        });
        BTreeMap::new()
    };

    if baseline_path.exists() {
        for (file, current_missing) in &missing_safety_counts {
            let baseline_missing = baseline_map.get(file).copied().unwrap_or(0);
            if *current_missing > baseline_missing {
                findings.push(UnsafeAuditFinding {
                    id: "MISSING_SAFETY_COMMENT_INCREASE".to_string(),
                    severity: "error".to_string(),
                    file: file.clone(),
                    line: 1,
                    message: format!(
                        "Undocumented unsafe blocks increased in {} from {} to {}",
                        file, baseline_missing, current_missing
                    ),
                    fix: "Add // SAFETY: comment explaining safety invariants before unsafe block"
                        .to_string(),
                });
            } else if *current_missing < baseline_missing {
                findings.push(UnsafeAuditFinding {
                    id: "SAFETY_COMMENT_IMPROVED".to_string(),
                    severity: "info".to_string(),
                    file: file.clone(),
                    line: 1,
                    message: format!(
                        "Undocumented unsafe blocks decreased in {} from {} to {}! Consider lowering baseline.",
                        file, baseline_missing, current_missing
                    ),
                    fix: "Update governance/unsafe-safety-baseline.toml".to_string(),
                });
            }
        }
    }

    for (file, count) in &unsafe_counts {
        findings.push(UnsafeAuditFinding {
            id: "UNSAFE_COUNT_INFO".to_string(),
            severity: "info".to_string(),
            file: file.clone(),
            line: 0,
            message: format!("Unsafe island site count for {}: {}", file, count),
            fix: "Information only".to_string(),
        });
    }

    if run_miri {
        for island in &islands {
            let miri_output = Command::new("cargo")
                .args(["+nightly", "miri", "test", "-p", island, "--locked"])
                .env("MIRIFLAGS", "-Zmiri-disable-isolation")
                .current_dir(&root)
                .output();

            match miri_output {
                Ok(o) => {
                    if !o.status.success() {
                        findings.push(UnsafeAuditFinding {
                            id: "MIRI_FAILURE".to_string(),
                            severity: "error".to_string(),
                            file: island.clone(),
                            line: 0,
                            message: format!("Miri validation failed for unsafe island {}", island),
                            fix: "Fix undefined behavior in unsafe block".to_string(),
                        });
                    }
                }
                Err(_) => {
                    let output = UnsafeAuditOutput {
                        gate: "unsafe-audit".to_string(),
                        status: "error".to_string(),
                        summary: "Miri toolchain missing or cargo +nightly miri execution failed"
                            .to_string(),
                        findings: vec![],
                    };
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&output).unwrap_or_default()
                        );
                    }
                    return 2;
                }
            }
        }
    }

    let has_errors = findings.iter().any(|f| f.severity == "error");
    let status = if has_errors { "fail" } else { "pass" };
    let summary = format!(
        "Unsafe audit completed for {} islands: {} findings",
        islands.len(),
        findings.len()
    );

    let output = UnsafeAuditOutput {
        gate: "unsafe-audit".to_string(),
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
        println!("{}", summary);
        for f in &findings {
            println!(
                "  [{}] {}: {}",
                f.severity.to_uppercase(),
                f.file,
                f.message
            );
        }
    }

    if status == "fail" {
        1
    } else {
        0
    }
}
