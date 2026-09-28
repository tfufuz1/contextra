//! Gate harness for verifying determinism across Ring 0 and Ring 1 crates.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeterminismCheckFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeterminismCheckOutput {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<DeterminismCheckFinding>,
}

#[derive(Debug, serde::Deserialize)]
pub struct DeterminismCheckAllowEntry {
    pub path: String,
    pub pattern: String,
    pub reason: String,
    pub adr: String,
}

impl DeterminismCheckAllowEntry {
    pub fn description(&self) -> String {
        format!("{}: {}", self.adr, self.reason)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct DeterminismCheckAllowFile {
    pub allow: Vec<DeterminismCheckAllowEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeterminismCheckBaselineEntry {
    pub file: String,
    pub line: usize,
    pub pattern: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeterminismCheckBaselineFile {
    pub baseline: Vec<DeterminismCheckBaselineEntry>,
}

pub fn determinism_check_parse_args(args: &[String]) -> (PathBuf, bool, bool, Option<String>) {
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut json = false;
    let mut write_baseline = false;
    let mut dynamic_crate = None;

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
            "--write-baseline" => {
                write_baseline = true;
            }
            "--crate" => {
                if i + 1 < args.len() {
                    dynamic_crate = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if let Ok(git_output) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&root)
        .output()
    {
        if git_output.status.success() {
            let top = String::from_utf8_lossy(&git_output.stdout).trim().to_string();
            if !top.is_empty() {
                root = PathBuf::from(top);
            }
        }
    }

    (root, json, write_baseline, dynamic_crate)
}

pub fn determinism_check_get_ring01_crates(root: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let caps_path = root.join("capabilities.toml");
    if !caps_path.exists() {
        return Err(format!("capabilities.toml missing at {}", caps_path.display()));
    }

    let content = fs::read_to_string(&caps_path)
        .map_err(|e| format!("failed to read capabilities.toml: {}", e))?;

    let val: serde_json::Value = toml::from_str(&content)
        .map_err(|e| format!("failed to parse capabilities.toml: {}", e))?;

    let mut crates = Vec::new();
    if let Some(crates_map) = val.get("crates").and_then(|v| v.as_object()) {
        for (name, obj) in crates_map {
            if let Some(ring) = obj.get("ring").and_then(|r| r.as_str()) {
                if ring == "Ring 0" || ring == "Ring 1" {
                    if let Some(rel_path) = obj.get("path").and_then(|p| p.as_str()) {
                        crates.push((name.clone(), root.join(rel_path)));
                    }
                }
            }
        }
    }

    Ok(crates)
}

pub fn determinism_check_is_test_or_bench(path: &Path, line: &str) -> bool {
    let path_str = path.to_string_lossy();
    if path_str.contains("/tests/") || path_str.contains("/benches/") || path_str.contains("/fuzz/") {
        return true;
    }
    let trimmed = line.trim();
    if trimmed.starts_with("#[test]") || trimmed.starts_with("#[cfg(test)]") || trimmed.contains("tokio::test") {
        return true;
    }
    false
}

pub fn determinism_check_scan_file(
    file_path: &Path,
    rel_path: &str,
    findings: &mut Vec<DeterminismCheckFinding>,
) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let forbidden_patterns = [
        ("SystemTime::now", "SystemTime::now() calls non-deterministic clock source"),
        ("Instant::now", "Instant::now() calls non-deterministic clock source"),
        ("thread_rng", "rand::thread_rng() uses unseeded thread-local random generator"),
        ("rand::random", "rand::random() uses unseeded random generation"),
        ("Uuid::new_v4", "Uuid::new_v4() generates random UUIDs breaking determinism"),
    ];

    let mut in_test_module = false;
    let mut current_fn_is_state_fn = false;

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim();

        if trimmed.contains("#[cfg(test)]") || trimmed.contains("mod tests") {
            in_test_module = true;
        }
        if in_test_module && trimmed.starts_with("pub mod ") && !trimmed.contains("tests") {
            in_test_module = false;
        }

        if determinism_check_is_test_or_bench(Path::new(rel_path), line) || in_test_module {
            continue;
        }

        for (pattern, msg) in forbidden_patterns {
            if line.contains(pattern) {
                findings.push(DeterminismCheckFinding {
                    id: format!("NON_DETERMINISTIC_{}", pattern.replace("::", "_")),
                    severity: "error".to_string(),
                    file: rel_path.to_string(),
                    line: line_num,
                    message: msg.to_string(),
                    fix: "Inject time/RNG via traits or context, or use collection.allocate_tx()".to_string(),
                });
            }
        }

        let lower_line = line.to_lowercase();
        if lower_line.contains("fn ") && (lower_line.contains("serialize") || lower_line.contains("encode") || lower_line.contains("to_bytes") || lower_line.contains("write") || lower_line.contains("flush") || lower_line.contains("checkpoint") || lower_line.contains("hash")) {
            current_fn_is_state_fn = true;
        } else if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
            current_fn_is_state_fn = false;
        }

        if current_fn_is_state_fn && (line.contains("for ") || line.contains(".iter()")) && (line.contains("hashmap") || line.contains("hashset") || line.contains("HashMap") || line.contains("HashSet")) {
            findings.push(DeterminismCheckFinding {
                id: "NON_DETERMINISTIC_HASH_ITERATION".to_string(),
                severity: "error".to_string(),
                file: rel_path.to_string(),
                line: line_num,
                message: "HashMap/HashSet iteration in serialization/hashing function can result in non-deterministic byte output".to_string(),
                fix: "Use BTreeMap/BTreeSet or sort keys before iterating".to_string(),
            });
        }
    }
}

pub fn run_determinism_check(args: &[String]) -> i32 {
    let (root, json, write_baseline, dynamic_crate) = determinism_check_parse_args(args);

    let ring01_crates = match determinism_check_get_ring01_crates(&root) {
        Ok(c) => c,
        Err(e) => {
            let out = DeterminismCheckOutput {
                gate: "determinism-check".to_string(),
                status: "error".to_string(),
                summary: e.clone(),
                findings: vec![],
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            } else {
                eprintln!("determinism-check ERROR: {}", e);
            }
            return 2;
        }
    };

    let allow_path = root.join("governance/determinism-allow.toml");
    let allow_entries: Vec<DeterminismCheckAllowEntry> = if allow_path.exists() {
        if let Ok(content) = fs::read_to_string(&allow_path) {
            if let Ok(parsed) = toml::from_str::<DeterminismCheckAllowFile>(&content) {
                parsed.allow
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let mut raw_findings = Vec::new();
    for (_crate_name, crate_path) in &ring01_crates {
        let src_dir = crate_path.join("src");
        if !src_dir.exists() {
            continue;
        }

        for entry in walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                let rel = match entry.path().strip_prefix(&root) {
                    Ok(p) => p.to_string_lossy().to_string(),
                    Err(_) => entry.path().to_string_lossy().to_string(),
                };
                determinism_check_scan_file(entry.path(), &rel, &mut raw_findings);
            }
        }
    }

    let filtered_findings: Vec<DeterminismCheckFinding> = raw_findings
        .into_iter()
        .filter(|f| {
            for allow in &allow_entries {
                if f.file.contains(&allow.path) && f.message.contains(&allow.pattern) {
                    let _desc = allow.description();
                    return false;
                }
            }
            true
        })
        .collect();

    let baseline_path = root.join("governance/determinism-baseline.toml");
    let mut baseline_entries: Vec<DeterminismCheckBaselineEntry> = Vec::new();

    if baseline_path.exists() {
        if let Ok(content) = fs::read_to_string(&baseline_path) {
            if let Ok(parsed) = toml::from_str::<DeterminismCheckBaselineFile>(&content) {
                baseline_entries = parsed.baseline;
            }
        }
    } else if write_baseline {
        let new_baseline: Vec<DeterminismCheckBaselineEntry> = filtered_findings
            .iter()
            .map(|f| DeterminismCheckBaselineEntry {
                file: f.file.clone(),
                line: f.line,
                pattern: f.id.clone(),
            })
            .collect();

        if let Ok(toml_str) = toml::to_string(&DeterminismCheckBaselineFile {
            baseline: new_baseline.clone(),
        }) {
            let _ = fs::create_dir_all(root.join("governance"));
            let _ = fs::write(&baseline_path, toml_str);
        }
        baseline_entries = new_baseline;
    }

    let mut final_findings = Vec::new();
    let mut new_error_count = 0;

    for mut finding in filtered_findings {
        let is_in_baseline = baseline_entries.iter().any(|b| {
            b.file == finding.file && b.pattern == finding.id && (b.line == finding.line || b.line == 0)
        });

        if is_in_baseline {
            finding.severity = "info".to_string();
        } else {
            finding.severity = "error".to_string();
            new_error_count += 1;
        }
        final_findings.push(finding);
    }

    if let Some(ref c) = dynamic_crate {
        let run1 = Command::new("cargo")
            .args(["test", "-p", c])
            .env("RUST_TEST_THREADS", "1")
            .output();
        let run2 = Command::new("cargo")
            .args(["test", "-p", c])
            .env("RUST_TEST_THREADS", "4")
            .output();

        if let (Ok(o1), Ok(o2)) = (run1, run2) {
            if o1.status.success() != o2.status.success() {
                final_findings.push(DeterminismCheckFinding {
                    id: "DYNAMIC_NON_DETERMINISM".to_string(),
                    severity: "error".to_string(),
                    file: c.clone(),
                    line: 0,
                    message: format!("Dynamic test run result mismatch for crate {} with RUST_TEST_THREADS 1 vs 4", c),
                    fix: "Fix concurrency or ordering dependencies in tests".to_string(),
                });
                new_error_count += 1;
            }
        }
    }

    let status = if new_error_count > 0 { "fail" } else { "pass" };
    let summary = format!(
        "Determinism check completed: {} findings total ({} new errors, {} baseline/info)",
        final_findings.len(),
        new_error_count,
        final_findings.len() - new_error_count
    );

    let output = DeterminismCheckOutput {
        gate: "determinism-check".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        findings: final_findings.clone(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!("{}", summary);
        if new_error_count > 0 {
            for f in &final_findings {
                if f.severity == "error" {
                    println!("  [ROT] WAS: {} in {}:{}", f.id, f.file, f.line);
                    println!("        WARUM: {}", f.message);
                    println!("        FIX: {}", f.fix);
                }
            }
        }
    }

    if status == "fail" {
        1
    } else {
        0
    }
}
