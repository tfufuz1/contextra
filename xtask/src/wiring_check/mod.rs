//! Meta-Gate `cargo xtask wiring-check`
//!
//! Validates end-to-end CI wiring across GitHub workflows, justfile, git hooks,
//! rulesets, and verdict requirements.

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct WiringFinding {
    pub file: String,
    pub line: usize,
    pub check: String,
    pub message: String,
    pub severity: String, // "error" or "warning"
}

#[derive(Debug, Serialize)]
pub struct WiringCheckReport {
    pub gate: String,
    pub pass: bool,
    pub findings: Vec<WiringFinding>,
}

pub fn run_wiring_check_cli(args: &[String]) -> i32 {
    let json_output = args.iter().any(|a| a == "--json");
    let root = crate::find_root_dir();

    let mut findings = run_all_checks(&root);

    // Deterministic sorting of findings
    findings.sort();

    let has_errors = findings.iter().any(|f| f.severity == "error");
    let pass = !has_errors;

    let report = WiringCheckReport {
        gate: "wiring-check".to_string(),
        pass,
        findings,
    };

    if json_output {
        if let Ok(json_str) = serde_json::to_string_pretty(&report) {
            println!("{}", json_str);
        } else {
            eprintln!("❌ Failed to serialize wiring-check report to JSON");
            return 1;
        }
    } else {
        println!("=== Meta-Gate: wiring-check ===");
        println!(
            "Status: {}",
            if pass {
                "PASSED"
            } else {
                "FAILED (violations found)"
            }
        );
        println!("Total findings: {}", report.findings.len());
        for f in &report.findings {
            let icon = if f.severity == "error" {
                "❌"
            } else {
                "⚠️"
            };
            println!(
                "{} [{}] {}:{}: {}",
                icon, f.check, f.file, f.line, f.message
            );
        }
    }

    if pass {
        0
    } else {
        1
    }
}

pub fn run_all_checks(root: &Path) -> Vec<WiringFinding> {
    let mut findings = Vec::new();

    findings.extend(check1_yaml_validity(root));
    findings.extend(check2_xtask_commands(root));
    findings.extend(check3_package_references(root));
    findings.extend(check4_feature_references(root));
    findings.extend(check5_fuzz_targets(root));
    findings.extend(check6_required_checks(root));
    findings.extend(check7_verdict_producers(root));
    findings.extend(check8_runner_labels(root));
    findings.extend(check9_security_warnings(root));

    findings
}

// -----------------------------------------------------------------------------
// Helper Functions
// -----------------------------------------------------------------------------

fn get_workflow_files(root: &Path) -> Vec<PathBuf> {
    let wf_dir = root.join(".github/workflows");
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&wf_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "yml" || ext == "yaml" {
                        files.push(path);
                    }
                }
            }
        }
    }
    files.sort();
    files
}

fn get_all_scannable_files(root: &Path) -> Vec<PathBuf> {
    let mut files = get_workflow_files(root);

    let justfile = root.join("justfile");
    if justfile.is_file() {
        files.push(justfile);
    }

    let hooks_dir = root.join(".githooks");
    if let Ok(entries) = fs::read_dir(&hooks_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

fn rel_path(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

// -----------------------------------------------------------------------------
// Check 1: YAML Validity
// -----------------------------------------------------------------------------

pub fn check1_yaml_validity(root: &Path) -> Vec<WiringFinding> {
    let mut findings = Vec::new();
    let workflow_files = get_workflow_files(root);

    // Try actionlint if in PATH
    let actionlint_available = Command::new("actionlint")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if actionlint_available {
        for wf in &workflow_files {
            let rel = rel_path(root, wf);
            if let Ok(output) = Command::new("actionlint")
                .arg("-oneline")
                .arg(wf)
                .output()
            {
                if !output.status.success() {
                    let stderr_or_stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stderr_or_stdout.lines() {
                        let line_trimmed = line.trim();
                        if !line_trimmed.is_empty() {
                            let parts: Vec<&str> = line_trimmed.splitn(3, ':').collect();
                            let line_num = if parts.len() >= 2 {
                                parts[1].parse::<usize>().unwrap_or(1)
                            } else {
                                1
                            };
                            let msg = if parts.len() >= 3 {
                                parts[2].trim().to_string()
                            } else {
                                line_trimmed.to_string()
                            };
                            findings.push(WiringFinding {
                                check: "check_1_yaml_validity".to_string(),
                                file: rel.clone(),
                                line: line_num,
                                message: format!("actionlint error: {}", msg),
                                severity: "error".to_string(),
                            });
                        }
                    }
                }
            }
        }
    } else {
        // Fallback structural parse check
        for wf in &workflow_files {
            let rel = rel_path(root, wf);
            if let Ok(content) = fs::read_to_string(wf) {
                findings.extend(check_yaml_content_fallback(&rel, &content));
            }
        }
    }

    findings
}

pub fn check_yaml_content_fallback(file_rel: &str, content: &str) -> Vec<WiringFinding> {
    let mut findings = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut in_jobs = false;
    let mut current_job: Option<String> = None;
    let mut job_lines: HashMap<String, Vec<(usize, &str)>> = HashMap::new();

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        if line.trim_start().starts_with("jobs:") {
            in_jobs = true;
            continue;
        }
        if in_jobs {
            if line.starts_with("  ") && !line.starts_with("   ") && line.trim().ends_with(':') {
                let job_id = line.trim().trim_end_matches(':').to_string();
                current_job = Some(job_id.clone());
                job_lines.entry(job_id).or_default();
                continue;
            }
            if let Some(ref job) = current_job {
                if line.starts_with("    ") || line.trim().is_empty() {
                    job_lines.entry(job.clone()).or_default().push((line_num, line));
                } else if line.starts_with("  ") {
                    current_job = None;
                }
            }
        }
    }

    for (job_id, jlines) in job_lines {
        let mut in_steps = false;
        let mut job_level_uses: Option<(usize, &str)> = None;
        let mut has_timeout = false;
        let mut has_runs_on = false;
        let mut has_steps = false;

        for (ln, line) in jlines {
            let trimmed = line.trim();
            if trimmed.starts_with("steps:") {
                in_steps = true;
                has_steps = true;
            }
            if !in_steps {
                if trimmed.starts_with("uses:") {
                    job_level_uses = Some((ln, trimmed));
                }
                if trimmed.starts_with("timeout-minutes:") {
                    has_timeout = true;
                }
                if trimmed.starts_with("runs-on:") {
                    has_runs_on = true;
                }
            }
        }

        if let Some((line_num, _)) = job_level_uses {
            if has_timeout || has_runs_on || has_steps {
                findings.push(WiringFinding {
                    check: "check_1_yaml_validity".to_string(),
                    file: file_rel.to_string(),
                    line: line_num,
                    message: format!(
                        "Job '{}' with job-level 'uses:' must not specify 'timeout-minutes', 'runs-on', or 'steps'",
                        job_id
                    ),
                    severity: "error".to_string(),
                });
            }
        }
    }

    findings
}

// -----------------------------------------------------------------------------
// Check 2: xtask Command Registry
// -----------------------------------------------------------------------------

pub fn check2_xtask_commands(root: &Path) -> Vec<WiringFinding> {
    let mut findings = Vec::new();
    let valid_commands = get_all_registered_xtask_commands(root);
    let files = get_all_scannable_files(root);

    let xtask_regex = match regex::Regex::new(
        r"(?:cargo\s+xtask|cargo\s+run\s+(?:--manifest-path\s+\S+\s+)?-p\s+xtask\s+--)\s+([a-z0-9_-]+)",
    ) {
        Ok(re) => re,
        Err(_) => return findings,
    };

    for file in &files {
        let rel = rel_path(root, file);
        if let Ok(content) = fs::read_to_string(file) {
            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;
                for caps in xtask_regex.captures_iter(line) {
                    if let Some(cmd_match) = caps.get(1) {
                        let cmd = cmd_match.as_str();
                        if !valid_commands.contains(cmd) {
                            findings.push(WiringFinding {
                                check: "check_2_xtask_commands".to_string(),
                                file: rel.clone(),
                                line: line_num,
                                message: format!("Unknown cargo xtask subcommand '{}'", cmd),
                                severity: "error".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    findings
}

pub fn get_all_registered_xtask_commands(root: &Path) -> HashSet<String> {
    let mut valid = HashSet::new();

    // 1. From cli/mod.rs
    for (cmd_name, _) in crate::cli::COMMAND_DISPATCH_TABLE {
        valid.insert((*cmd_name).to_string());
    }

    // 2. From registry.toml
    let registry_path = root.join("xtask/registry.toml");
    if let Ok(content) = fs::read_to_string(&registry_path) {
        if let Ok(name_regex) = regex::Regex::new(r#"name\s*=\s*"([^"]+)""#) {
            for caps in name_regex.captures_iter(&content) {
                if let Some(m) = caps.get(1) {
                    valid.insert(m.as_str().to_string());
                }
            }
        }
    }

    // 3. From harness modules
    let harness_dir = root.join("xtask/src/harness");
    if let Ok(entries) = fs::read_dir(&harness_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem != "mod" {
                        valid.insert(stem.replace('_', "-"));
                    }
                }
            }
        }
    }

    // 4. Builtins
    valid.insert("harness-list".to_string());
    valid.insert("harness-help".to_string());
    valid.insert("wiring-check".to_string());

    valid
}

// -----------------------------------------------------------------------------
// Check 3: Package References (-p / --package)
// -----------------------------------------------------------------------------

pub fn check3_package_references(root: &Path) -> Vec<WiringFinding> {
    let mut findings = Vec::new();
    let ws_packages = get_workspace_package_names(root);
    let files = get_all_scannable_files(root);

    let pkg_flag_regex = match regex::Regex::new(r"(?:^|\s)(?:-p|--package)(?:\s+|=)([a-zA-Z0-9_-]+)") {
        Ok(re) => re,
        Err(_) => return findings,
    };
    let crate_ref_regex = match regex::Regex::new(r"\b(contextra-[a-zA-Z0-9_-]+)\b") {
        Ok(re) => re,
        Err(_) => return findings,
    };

    for file in &files {
        let rel = rel_path(root, file);
        if let Ok(content) = fs::read_to_string(file) {
            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;

                // Check 3.1: Explicit -p or --package flags on cargo invocations or justfile receipts
                if line.contains("cargo ") || line.contains("cargo\t") || line.contains("just ") || rel == "justfile" {
                    for caps in pkg_flag_regex.captures_iter(line) {
                        if let Some(pkg_match) = caps.get(1) {
                            let pkg = pkg_match.as_str();
                            if !ws_packages.contains(pkg)
                                && !pkg.starts_with('$')
                                && !pkg.starts_with('{')
                            {
                                findings.push(WiringFinding {
                                    check: "check_3_package_references".to_string(),
                                    file: rel.clone(),
                                    line: line_num,
                                    message: format!(
                                        "Package '{}' in '-p / --package' flag does not exist in workspace",
                                        pkg
                                    ),
                                    severity: "error".to_string(),
                                });
                            }
                        }
                    }
                }

                // Check 3.2: Literal contextra-* package references in shell loops/lists
                if line.contains("for ") || line.contains("cargo ") || line.contains("CRATES=") || line.contains("crates=") {
                    for caps in crate_ref_regex.captures_iter(line) {
                        if let Some(m) = caps.get(1) {
                            let pkg = m.as_str();
                            if !ws_packages.contains(pkg) {
                                // Skip if part of a directory path like crates/contextra-candle
                                if !line.contains(&format!("crates/{}", pkg))
                                    && !line.contains(&format!("/{}", pkg))
                                {
                                    findings.push(WiringFinding {
                                        check: "check_3_package_references".to_string(),
                                        file: rel.clone(),
                                        line: line_num,
                                        message: format!(
                                            "Referenced package '{}' does not exist in workspace",
                                            pkg
                                        ),
                                        severity: "error".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    findings
}

pub fn get_workspace_package_names(root: &Path) -> HashSet<String> {
    let mut names = HashSet::new();

    if let Ok(output) = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(root)
        .output()
    {
        if output.status.success() {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(packages) = json.get("packages").and_then(|p| p.as_array()) {
                    for pkg in packages {
                        if let Some(name) = pkg.get("name").and_then(|n| n.as_str()) {
                            names.insert(name.to_string());
                        }
                    }
                }
            }
        }
    }

    if names.is_empty() {
        // Fallback: parse workspace crates via xtask helper
        for c in crate::get_workspace_crates() {
            names.insert(c.name);
        }
        names.insert("xtask".to_string());
        names.insert("xtask-heavy".to_string());
    }

    names
}

// -----------------------------------------------------------------------------
// Check 4: Feature References (--features)
// -----------------------------------------------------------------------------

pub fn check4_feature_references(root: &Path) -> Vec<WiringFinding> {
    let mut findings = Vec::new();
    let pkg_features = get_workspace_package_features(root);
    let files = get_all_scannable_files(root);

    let pkg_flag_regex = match regex::Regex::new(r"(?:^|\s)(?:-p|--package)(?:\s+|=)([a-zA-Z0-9_-]+)") {
        Ok(re) => re,
        Err(_) => return findings,
    };
    let feats_regex = match regex::Regex::new(r#"--features(?:\s+|=)(["a-zA-Z0-9_,\s-]+)"#) {
        Ok(re) => re,
        Err(_) => return findings,
    };

    for file in &files {
        let rel = rel_path(root, file);
        if let Ok(content) = fs::read_to_string(file) {
            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;

                if line.contains("--features") {
                    let pkgs: Vec<String> = pkg_flag_regex
                        .captures_iter(line)
                        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
                        .collect();

                    if let Some(caps) = feats_regex.captures(line) {
                        if let Some(feats_match) = caps.get(1) {
                            let raw_feats = feats_match.as_str().trim_matches('"').trim_matches('\'');
                            let feats: Vec<&str> = raw_feats.split(',').map(|s| s.trim()).collect();

                            for pkg in &pkgs {
                                if let Some(declared_features) = pkg_features.get(pkg) {
                                    for feat in &feats {
                                        if !feat.is_empty()
                                            && !feat.starts_with('$')
                                            && !feat.starts_with('{')
                                            && !declared_features.contains(*feat)
                                        {
                                            findings.push(WiringFinding {
                                                check: "check_4_feature_references".to_string(),
                                                file: rel.clone(),
                                                line: line_num,
                                                message: format!(
                                                    "Feature '{}' does not exist in package '{}'",
                                                    feat, pkg
                                                ),
                                                severity: "error".to_string(),
                                            });
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

    findings
}

pub fn get_workspace_package_features(root: &Path) -> HashMap<String, HashSet<String>> {
    let mut map = HashMap::new();

    if let Ok(output) = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(root)
        .output()
    {
        if output.status.success() {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(packages) = json.get("packages").and_then(|p| p.as_array()) {
                    for pkg in packages {
                        if let Some(name) = pkg.get("name").and_then(|n| n.as_str()) {
                            let mut feats = HashSet::new();
                            if let Some(f_table) = pkg.get("features").and_then(|f| f.as_object()) {
                                for key in f_table.keys() {
                                    feats.insert(key.clone());
                                }
                            }
                            feats.insert("default".to_string());
                            map.insert(name.to_string(), feats);
                        }
                    }
                }
            }
        }
    }

    map
}

// -----------------------------------------------------------------------------
// Check 5: Fuzz Targets
// -----------------------------------------------------------------------------

pub fn check5_fuzz_targets(root: &Path) -> Vec<WiringFinding> {
    let mut findings = Vec::new();
    let fuzz_map = get_crate_fuzz_targets(root);

    let mut scannable = get_workflow_files(root);
    let justfile = root.join("justfile");
    if justfile.is_file() {
        scannable.push(justfile);
    }

    let matrix_regex = match regex::Regex::new(r"crate:\s*([a-zA-Z0-9_-]+),\s*target:\s*([a-zA-Z0-9_-]+)") {
        Ok(re) => re,
        Err(_) => return findings,
    };
    let justfile_target_regex =
        match regex::Regex::new(r#""(contextra-[a-zA-Z0-9_-]+):([a-zA-Z0-9_-]+)""#) {
            Ok(re) => re,
            Err(_) => return findings,
        };

    for file in &scannable {
        let rel = rel_path(root, file);
        if let Ok(content) = fs::read_to_string(file) {
            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;

                // Matrix entries
                for caps in matrix_regex.captures_iter(line) {
                    let crate_name = caps.get(1).map_or("", |m| m.as_str());
                    let target_name = caps.get(2).map_or("", |m| m.as_str());

                    if let Some(targets) = fuzz_map.get(crate_name) {
                        if !targets.contains(target_name) {
                            findings.push(WiringFinding {
                                check: "check_5_fuzz_targets".to_string(),
                                file: rel.clone(),
                                line: line_num,
                                message: format!(
                                    "Fuzz target '{}' does not exist in crate '{}'",
                                    target_name, crate_name
                                ),
                                severity: "error".to_string(),
                            });
                        }
                    } else {
                        findings.push(WiringFinding {
                            check: "check_5_fuzz_targets".to_string(),
                            file: rel.clone(),
                            line: line_num,
                            message: format!(
                                "Crate '{}' has no fuzz targets defined in crates/{}/fuzz/Cargo.toml",
                                crate_name, crate_name
                            ),
                            severity: "error".to_string(),
                        });
                    }
                }

                // Justfile "crate:target" entries
                for caps in justfile_target_regex.captures_iter(line) {
                    let crate_name = caps.get(1).map_or("", |m| m.as_str());
                    let target_name = caps.get(2).map_or("", |m| m.as_str());

                    if let Some(targets) = fuzz_map.get(crate_name) {
                        if !targets.contains(target_name) {
                            findings.push(WiringFinding {
                                check: "check_5_fuzz_targets".to_string(),
                                file: rel.clone(),
                                line: line_num,
                                message: format!(
                                    "Fuzz target '{}' in 'fuzz-all' does not exist in crate '{}'",
                                    target_name, crate_name
                                ),
                                severity: "error".to_string(),
                            });
                        }
                    } else {
                        findings.push(WiringFinding {
                            check: "check_5_fuzz_targets".to_string(),
                            file: rel.clone(),
                            line: line_num,
                            message: format!(
                                "Crate '{}' referenced in 'fuzz-all' has no fuzz/Cargo.toml",
                                crate_name
                            ),
                            severity: "error".to_string(),
                        });
                    }
                }
            }
        }
    }

    findings
}

pub fn get_crate_fuzz_targets(root: &Path) -> HashMap<String, HashSet<String>> {
    let mut map: HashMap<String, HashSet<String>> = HashMap::new();
    let crates_dir = root.join("crates");

    let bin_name_regex = match regex::Regex::new(r#"name\s*=\s*"([^"]+)""#) {
        Ok(re) => re,
        Err(_) => return map,
    };

    if let Ok(entries) = fs::read_dir(&crates_dir) {
        for entry in entries.flatten() {
            let crate_dir = entry.path();
            if crate_dir.is_dir() {
                let fuzz_cargo = crate_dir.join("fuzz/Cargo.toml");
                if fuzz_cargo.is_file() {
                    let crate_name = crate_dir
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default()
                        .to_string();

                    if let Ok(content) = fs::read_to_string(&fuzz_cargo) {
                        let mut targets = HashSet::new();
                        // Parse [[bin]] blocks
                        for block in content.split("[[bin]]").skip(1) {
                            if let Some(caps) = bin_name_regex.captures(block) {
                                if let Some(m) = caps.get(1) {
                                    targets.insert(m.as_str().to_string());
                                }
                            }
                        }
                        map.insert(crate_name, targets);
                    }
                }
            }
        }
    }

    map
}

// -----------------------------------------------------------------------------
// Check 6: Required Status Check Names in Rulesets
// -----------------------------------------------------------------------------

pub fn check6_required_checks(root: &Path) -> Vec<WiringFinding> {
    let mut findings = Vec::new();
    let ruleset_path = root.join(".github/rulesets/protect-main.json");

    if !ruleset_path.is_file() {
        return findings;
    }

    let rel_ruleset = rel_path(root, &ruleset_path);
    let required_checks = parse_protect_main_required_checks(&ruleset_path);

    let workflow_job_names = get_all_workflow_job_names(root);

    for (check_name, line_num) in required_checks {
        if !workflow_job_names.contains(&check_name) {
            findings.push(WiringFinding {
                check: "check_6_required_checks".to_string(),
                file: rel_ruleset.clone(),
                line: line_num,
                message: format!(
                    "Required check '{}' does not match any workflow job 'name:' or job ID",
                    check_name
                ),
                severity: "error".to_string(),
            });
        }
    }

    findings
}

fn parse_protect_main_required_checks(path: &Path) -> Vec<(String, usize)> {
    let mut res = Vec::new();
    if let Ok(content) = fs::read_to_string(path) {
        let lines: Vec<&str> = content.lines().collect();
        let context_regex = match regex::Regex::new(r#""context"\s*:\s*"([^"]+)""#) {
            Ok(re) => re,
            Err(_) => return res,
        };

        for (idx, line) in lines.iter().enumerate() {
            if let Some(caps) = context_regex.captures(line) {
                if let Some(m) = caps.get(1) {
                    res.push((m.as_str().to_string(), idx + 1));
                }
            }
        }
    }
    res
}

fn get_all_workflow_job_names(root: &Path) -> HashSet<String> {
    let mut names = HashSet::new();
    let files = get_workflow_files(root);

    let name_regex = match regex::Regex::new(r"^\s*name:\s*(.+)$") {
        Ok(re) => re,
        Err(_) => return names,
    };
    let job_id_regex = match regex::Regex::new(r"^  ([a-zA-Z0-9_-]+):\s*$") {
        Ok(re) => re,
        Err(_) => return names,
    };

    for file in &files {
        if let Ok(content) = fs::read_to_string(file) {
            for line in content.lines() {
                if let Some(caps) = name_regex.captures(line) {
                    if let Some(m) = caps.get(1) {
                        let clean = m.as_str().trim().trim_matches('"').trim_matches('\'');
                        names.insert(clean.to_string());
                    }
                }
                if let Some(caps) = job_id_regex.captures(line) {
                    if let Some(m) = caps.get(1) {
                        names.insert(m.as_str().to_string());
                    }
                }
            }
        }
    }

    names
}

// -----------------------------------------------------------------------------
// Check 7: Verdict Blocking Gates Producer Artifacts
// -----------------------------------------------------------------------------

pub fn check7_verdict_producers(root: &Path) -> Vec<WiringFinding> {
    let mut findings = Vec::new();
    let verdict_req_path = root.join("governance/verdict-required.toml");

    if !verdict_req_path.is_file() {
        return findings;
    }

    let rel_verdict = rel_path(root, &verdict_req_path);
    let blocking_gates = parse_verdict_blocking_gates(&verdict_req_path);
    let uploaded_artifacts = get_all_uploaded_artifact_names(root);

    for (gate_name, line_num) in blocking_gates {
        let norm_gate = if let Some(stripped) = gate_name.strip_prefix("gate-") {
            stripped
        } else {
            &gate_name
        };

        let expected_artifact1 = format!("gate-{}", norm_gate);
        let expected_artifact2 = norm_gate.to_string();

        if !uploaded_artifacts.contains(&expected_artifact1)
            && !uploaded_artifacts.contains(&expected_artifact2)
        {
            findings.push(WiringFinding {
                check: "check_7_verdict_producers".to_string(),
                file: rel_verdict.clone(),
                line: line_num,
                message: format!(
                    "Blocking gate '{}' has no workflow job uploading artifact '{}'",
                    gate_name, expected_artifact1
                ),
                severity: "error".to_string(),
            });
        }
    }

    findings
}

fn parse_verdict_blocking_gates(path: &Path) -> Vec<(String, usize)> {
    let mut res = Vec::new();
    if let Ok(content) = fs::read_to_string(path) {
        let lines: Vec<&str> = content.lines().collect();

        let mut current_name: Option<String> = None;
        let mut name_line = 0;

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("[[gate]]") {
                current_name = None;
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("name =") {
                let name = rest.trim().trim_matches('"').trim_matches('\'');
                current_name = Some(name.to_string());
                name_line = line_num;
            }

            if trimmed == "blocking = true" {
                if let Some(ref name) = current_name {
                    res.push((name.clone(), name_line));
                }
            }
        }
    }
    res
}

fn get_all_uploaded_artifact_names(root: &Path) -> HashSet<String> {
    let mut artifacts = HashSet::new();
    let files = get_workflow_files(root);

    let name_regex = match regex::Regex::new(r"^\s*name:\s*(.+)$") {
        Ok(re) => re,
        Err(_) => return artifacts,
    };

    for file in &files {
        if let Ok(content) = fs::read_to_string(file) {
            let lines: Vec<&str> = content.lines().collect();
            for (idx, line) in lines.iter().enumerate() {
                if line.contains("uses: actions/upload-artifact@") {
                    // Search subsequent lines for name:
                    for next_line in lines.iter().skip(idx + 1).take(10) {
                        if let Some(caps) = name_regex.captures(next_line) {
                            if let Some(m) = caps.get(1) {
                                let clean = m.as_str().trim().trim_matches('"').trim_matches('\'');
                                artifacts.insert(clean.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    artifacts
}

// -----------------------------------------------------------------------------
// Check 8: Runner Labels (Forbidden: ubuntu-latest, macos-13)
// -----------------------------------------------------------------------------

pub const FORBIDDEN_RUNNERS: &[&str] = &["ubuntu-latest", "macos-13"];

pub fn check8_runner_labels(root: &Path) -> Vec<WiringFinding> {
    let mut findings = Vec::new();
    let workflow_files = get_workflow_files(root);

    for wf in &workflow_files {
        let rel = rel_path(root, wf);
        if let Ok(content) = fs::read_to_string(wf) {
            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;
                if line.trim_start().starts_with("runs-on:") || line.contains("os:") {
                    for forbidden in FORBIDDEN_RUNNERS {
                        if line.contains(forbidden) {
                            findings.push(WiringFinding {
                                check: "check_8_runner_labels".to_string(),
                                file: rel.clone(),
                                line: line_num,
                                message: format!("Forbidden runner label '{}' in workflow", forbidden),
                                severity: "error".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    findings
}

// -----------------------------------------------------------------------------
// Check 9: Security Warnings
// -----------------------------------------------------------------------------

pub fn check9_security_warnings(root: &Path) -> Vec<WiringFinding> {
    let mut findings = Vec::new();
    let workflow_files = get_workflow_files(root);

    let sha_regex = match regex::Regex::new(r"^[0-9a-f]{40}$") {
        Ok(re) => re,
        Err(_) => return findings,
    };
    let script_interp_regex = match regex::Regex::new(r"\$\{\{\s*(?:github\.event\.|github\.head_ref|inputs\.)") {
        Ok(re) => re,
        Err(_) => return findings,
    };

    for wf in &workflow_files {
        let rel = rel_path(root, wf);
        if let Ok(content) = fs::read_to_string(wf) {
            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;
                let trimmed = line.trim();

                // 9.1: Actions with non-SHA ref
                if trimmed.starts_with("uses:") || trimmed.starts_with("- uses:") {
                    let val = trimmed.split("uses:").nth(1).unwrap_or("").trim();
                    if val.contains('@') && !val.starts_with("./") {
                        let parts: Vec<&str> = val.split('@').collect();
                        let action = parts[0];
                        let ref_str = parts[1].split('#').next().unwrap_or("").trim();

                        if !sha_regex.is_match(ref_str) {
                            findings.push(WiringFinding {
                                check: "check_9_security_warnings".to_string(),
                                file: rel.clone(),
                                line: line_num,
                                message: format!(
                                    "Action '{}' uses non-SHA ref '{}'",
                                    action, ref_str
                                ),
                                severity: "warning".to_string(),
                            });
                        }
                    }
                }

                // 9.2: Script interpolation in run: block
                if script_interp_regex.is_match(line) {
                    findings.push(WiringFinding {
                        check: "check_9_security_warnings".to_string(),
                        file: rel.clone(),
                        line: line_num,
                        message:
                            "Direct script interpolation of '${{ ... }}' in run block can allow script injection"
                                .to_string(),
                        severity: "warning".to_string(),
                    });
                }
            }
        }
    }

    findings
}

// -----------------------------------------------------------------------------
// Unit Tests on Fixtures
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_check1_yaml_fallback_fixture() {
        let yaml_valid = r#"
jobs:
  build:
    runs-on: ubuntu-24.04
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
"#;
        let findings1 = check_yaml_content_fallback("test.yml", yaml_valid);
        assert!(findings1.is_empty());

        let yaml_invalid = r#"
jobs:
  deep-gates:
    uses: ./.github/workflows/deep-gates.yml
    timeout-minutes: 10
"#;
        let findings2 = check_yaml_content_fallback("test.yml", yaml_invalid);
        assert_eq!(findings2.len(), 1);
        assert_eq!(findings2[0].check, "check_1_yaml_validity");
        assert_eq!(findings2[0].severity, "error");
    }

    #[test]
    fn test_check2_xtask_commands_fixture() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();

        let wf_dir = root.join(".github/workflows");
        fs::create_dir_all(&wf_dir).expect("create_dir_all");
        let wf_content = r#"
name: Test
jobs:
  test:
    steps:
      - run: cargo xtask check-compile
      - run: cargo xtask invalid-nonexistent-command
"#;
        fs::write(wf_dir.join("test.yml"), wf_content).expect("write");

        let findings = check2_xtask_commands(root);
        assert!(findings.iter().any(|f| f.message.contains("invalid-nonexistent-command")));
    }

    #[test]
    fn test_check3_package_references_fixture() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();

        let wf_dir = root.join(".github/workflows");
        fs::create_dir_all(&wf_dir).expect("create_dir_all");
        let wf_content = r#"
name: Test
jobs:
  test:
    steps:
      - run: cargo test -p contextra-nonexistent-crate
"#;
        fs::write(wf_dir.join("test.yml"), wf_content).expect("write");

        let findings = check3_package_references(root);
        assert!(findings.iter().any(|f| f.message.contains("contextra-nonexistent-crate")));
    }

    #[test]
    fn test_check4_feature_references_fixture() {
        let root = crate::find_root_dir();

        let dir = tempdir().expect("tempdir");
        let temp_root = dir.path();

        let wf_dir = temp_root.join(".github/workflows");
        fs::create_dir_all(&wf_dir).expect("create_dir_all");
        let wf_content = r#"
name: Test
jobs:
  test:
    steps:
      - run: cargo check -p xtask --features non-existent-feature
"#;
        fs::write(wf_dir.join("test.yml"), wf_content).expect("write");

        // Use real workspace features + fixture workflow file
        let pkg_features = get_workspace_package_features(&root);
        let files = get_all_scannable_files(temp_root);

        let pkg_flag_regex = regex::Regex::new(r"(?:^|\s)(?:-p|--package)(?:\s+|=)([a-zA-Z0-9_-]+)").unwrap();
        let feats_regex = regex::Regex::new(r#"--features(?:\s+|=)(["a-zA-Z0-9_,\s-]+)"#).unwrap();

        let mut findings = Vec::new();
        for file in &files {
            let rel = rel_path(temp_root, file);
            if let Ok(content) = fs::read_to_string(file) {
                for (idx, line) in content.lines().enumerate() {
                    if line.contains("--features") {
                        let pkgs: Vec<String> = pkg_flag_regex
                            .captures_iter(line)
                            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
                            .collect();
                        if let Some(caps) = feats_regex.captures(line) {
                            if let Some(feats_match) = caps.get(1) {
                                let raw = feats_match.as_str().trim_matches('"').trim_matches('\'');
                                let feats: Vec<&str> = raw.split(',').map(|s| s.trim()).collect();
                                for pkg in &pkgs {
                                    if let Some(declared) = pkg_features.get(pkg) {
                                        for feat in &feats {
                                            if !feat.is_empty() && !declared.contains(*feat) {
                                                findings.push(WiringFinding {
                                                    check: "check_4_feature_references".to_string(),
                                                    file: rel.clone(),
                                                    line: idx + 1,
                                                    message: format!("Feature '{}' does not exist in package '{}'", feat, pkg),
                                                    severity: "error".to_string(),
                                                });
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

        assert!(findings.iter().any(|f| f.message.contains("non-existent-feature")));
    }

    #[test]
    fn test_check5_fuzz_targets_fixture() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();

        let crate_fuzz_dir = root.join("crates/contextra-test/fuzz");
        fs::create_dir_all(&crate_fuzz_dir).expect("create_dir_all");
        fs::write(
            crate_fuzz_dir.join("Cargo.toml"),
            "[[bin]]\nname = \"valid_fuzz_target\"\n",
        )
        .expect("write");

        let wf_dir = root.join(".github/workflows");
        fs::create_dir_all(&wf_dir).expect("create_dir_all");
        let wf_content = r#"
name: Test
jobs:
  fuzz:
    strategy:
      matrix:
        target:
          - { crate: contextra-test, target: invalid_fuzz_target }
"#;
        fs::write(wf_dir.join("fuzz.yml"), wf_content).expect("write");

        let findings = check5_fuzz_targets(root);
        assert!(findings.iter().any(|f| f.message.contains("invalid_fuzz_target")));
    }

    #[test]
    fn test_check6_required_checks_fixture() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();

        let ruleset_dir = root.join(".github/rulesets");
        fs::create_dir_all(&ruleset_dir).expect("create_dir_all");
        let ruleset_content = r#"{
  "rules": [
    {}, {}, {},
    {
      "type": "required_status_checks",
      "parameters": {
        "required_status_checks": [
          { "context": "Missing Required Job Name" }
        ]
      }
    }
  ]
}"#;
        fs::write(ruleset_dir.join("protect-main.json"), ruleset_content).expect("write");

        let findings = check6_required_checks(root);
        assert!(findings.iter().any(|f| f.message.contains("Missing Required Job Name")));
    }

    #[test]
    fn test_check7_verdict_producers_fixture() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();

        let gov_dir = root.join("governance");
        fs::create_dir_all(&gov_dir).expect("create_dir_all");
        let verdict_req = r#"
[[gate]]
name = "nonexistent-producer-gate"
blocking = true
"#;
        fs::write(gov_dir.join("verdict-required.toml"), verdict_req).expect("write");

        let findings = check7_verdict_producers(root);
        assert!(findings.iter().any(|f| f.message.contains("nonexistent-producer-gate")));
    }

    #[test]
    fn test_check8_runner_labels_fixture() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();

        let wf_dir = root.join(".github/workflows");
        fs::create_dir_all(&wf_dir).expect("create_dir_all");
        let wf_content = r#"
name: Test
jobs:
  test:
    runs-on: ubuntu-latest
"#;
        fs::write(wf_dir.join("test.yml"), wf_content).expect("write");

        let findings = check8_runner_labels(root);
        assert!(findings.iter().any(|f| f.message.contains("ubuntu-latest")));
    }

    #[test]
    fn test_check9_security_warnings_fixture() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();

        let wf_dir = root.join(".github/workflows");
        fs::create_dir_all(&wf_dir).expect("create_dir_all");
        let wf_content = r#"
name: Test
jobs:
  test:
    steps:
      - uses: actions/checkout@v4
      - run: echo "${{ github.event.pull_request.body }}"
"#;
        fs::write(wf_dir.join("test.yml"), wf_content).expect("write");

        let findings = check9_security_warnings(root);
        assert!(findings.iter().any(|f| f.severity == "warning"));
    }
}
