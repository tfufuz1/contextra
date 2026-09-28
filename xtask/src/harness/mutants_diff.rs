//! Gate harness for diff-scoped mutation testing score enforcement.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MutantsDiffFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MutantsDiffOutput {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<MutantsDiffFinding>,
}

pub fn mutants_diff_get_tier1_touched_crates(
    root: &Path,
    base: &str,
    head: &str,
) -> Result<BTreeSet<String>, String> {
    let prompter_path = root.join(".jules/prompter-tiers.toml");
    let mut tier1_crates = BTreeSet::new();

    if prompter_path.exists() {
        if let Ok(content) = fs::read_to_string(&prompter_path) {
            if let Ok(val) = toml::from_str::<serde_json::Value>(&content) {
                if let Some(overrides) = val.get("crate_overrides").and_then(|v| v.as_object()) {
                    for (cname, obj) in overrides {
                        if obj.get("tier").and_then(|t| t.as_str()) == Some("1") {
                            tier1_crates.insert(cname.clone());
                        }
                    }
                }
            }
        }
    }

    let diff_output = Command::new("git")
        .args(["diff", "--name-only", base, head])
        .current_dir(root)
        .output();

    let mut touched = BTreeSet::new();
    if let Ok(o) = diff_output {
        let diff_text = String::from_utf8_lossy(&o.stdout);
        for line in diff_text.lines() {
            if line.starts_with("crates/") {
                let parts: Vec<&str> = line.split('/').collect();
                if parts.len() >= 2 && tier1_crates.contains(parts[1]) {
                    touched.insert(parts[1].to_string());
                }
            }
        }
    }

    Ok(touched)
}

pub fn mutants_diff_get_expected_score(root: &Path, crate_name: &str) -> f64 {
    let history_path = root.join("docs/mutation_score_history.jsonl");
    if history_path.exists() {
        if let Ok(content) = fs::read_to_string(&history_path) {
            let mut last_score = None;
            for line in content.lines() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                    if val.get("crate").and_then(|v| v.as_str()) == Some(crate_name) {
                        if let Some(score) = val.get("score").and_then(|v| v.as_f64()) {
                            last_score = Some(score);
                        }
                    }
                }
            }
            if let Some(s) = last_score {
                return s;
            }
        }
    }
    0.60
}

pub fn run_mutants_diff(args: &[String]) -> i32 {
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut json = false;
    let mut base_rev = None;
    let mut head_rev = None;

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
            "--base" => {
                if i + 1 < args.len() {
                    let val = args[i + 1].trim();
                    if !val.is_empty() {
                        base_rev = Some(val.to_string());
                    }
                    i += 1;
                }
            }
            "--head" => {
                if i + 1 < args.len() {
                    let val = args[i + 1].trim();
                    if !val.is_empty() {
                        head_rev = Some(val.to_string());
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let base = base_rev.unwrap_or_else(|| "HEAD~1".to_string());
    let head = head_rev.unwrap_or_else(|| "HEAD".to_string());

    let touched_crates = match mutants_diff_get_tier1_touched_crates(&root, &base, &head) {
        Ok(c) => c,
        Err(e) => {
            let out = MutantsDiffOutput {
                gate: "mutants-diff".to_string(),
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

    if touched_crates.is_empty() {
        let output = MutantsDiffOutput {
            gate: "mutants-diff".to_string(),
            status: "not_applicable".to_string(),
            summary: "No Tier-1 crates touched in diff".to_string(),
            findings: vec![],
        };
        if json {
            println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
        } else {
            println!("mutants-diff: not_applicable (no Tier-1 crates touched)");
        }
        return 0;
    }

    let diff_file_path = root.join("target/mutants_diff.patch");
    let _ = fs::create_dir_all(root.join("target"));

    let diff_gen = Command::new("git")
        .args(["diff", &base, &head])
        .current_dir(&root)
        .output();

    if let Ok(o) = diff_gen {
        let _ = fs::write(&diff_file_path, o.stdout);
    }

    let mut findings = Vec::new();
    let mut has_error = false;

    for crate_name in &touched_crates {
        let expected_score = mutants_diff_get_expected_score(&root, crate_name);

        let mutants_cmd = Command::new("cargo")
            .args([
                "mutants",
                "--in-diff",
                diff_file_path.to_str().unwrap_or_default(),
                "-p",
                crate_name,
                "--timeout",
                "120",
                "--minimum-test-timeout",
                "30",
                "--in-place",
            ])
            .current_dir(&root)
            .output();

        match mutants_cmd {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);

                if !o.status.success() && (stderr.contains("no such subcommand") || stdout.contains("no such subcommand") || stderr.contains("error:")) {
                    has_error = true;
                    findings.push(MutantsDiffFinding {
                        id: "CARGO_MUTANTS_MISSING".to_string(),
                        severity: "error".to_string(),
                        file: format!("crates/{}", crate_name),
                        line: 0,
                        message: "cargo-mutants toolchain missing or cargo subcommand failed".to_string(),
                        fix: "Install cargo-mutants or verify PATH".to_string(),
                    });
                    continue;
                }

                let mut caught = 0;
                let mut missed = 0;

                for line in stdout.lines() {
                    if line.contains("caught") || line.contains("MISSED") {
                        if line.contains("caught") {
                            caught += 1;
                        }
                        if line.contains("MISSED") {
                            missed += 1;
                        }
                    }
                }

                let total = caught + missed;
                let actual_score = if total > 0 {
                    caught as f64 / total as f64
                } else {
                    1.0
                };

                if actual_score < expected_score {
                    findings.push(MutantsDiffFinding {
                        id: "MUTATION_SCORE_BELOW_THRESHOLD".to_string(),
                        severity: "error".to_string(),
                        file: format!("crates/{}", crate_name),
                        line: 0,
                        message: format!(
                            "Mutation score for {} ({:.2}%) below expected threshold ({:.2}%)",
                            crate_name,
                            actual_score * 100.0,
                            expected_score * 100.0
                        ),
                        fix: "Add targeted unit tests to cover missed AST mutations".to_string(),
                    });
                } else {
                    findings.push(MutantsDiffFinding {
                        id: "MUTATION_SCORE_PASS".to_string(),
                        severity: "info".to_string(),
                        file: format!("crates/{}", crate_name),
                        line: 0,
                        message: format!(
                            "Mutation score for {}: {:.2}% (expected >= {:.2}%)",
                            crate_name,
                            actual_score * 100.0,
                            expected_score * 100.0
                        ),
                        fix: "None required".to_string(),
                    });
                }
            }
            Err(_) => {
                has_error = true;
                findings.push(MutantsDiffFinding {
                    id: "CARGO_MUTANTS_MISSING".to_string(),
                    severity: "error".to_string(),
                    file: format!("crates/{}", crate_name),
                    line: 0,
                    message: "cargo-mutants executable missing or failed to run".to_string(),
                    fix: "Install cargo-mutants or verify PATH".to_string(),
                });
            }
        }
    }

    let status = if has_error {
        "error"
    } else if findings.iter().any(|f| f.severity == "error") {
        "fail"
    } else {
        "pass"
    };

    let summary = format!(
        "Mutants diff testing status: {} with {} findings",
        status,
        findings.len()
    );

    let output = MutantsDiffOutput {
        gate: "mutants-diff".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        findings: findings.clone(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!("{}", summary);
        for f in &findings {
            println!("  [{}] {}: {}", f.severity.to_uppercase(), f.file, f.message);
        }
    }

    match status {
        "pass" => 0,
        "fail" => 1,
        _ => 2,
    }
}
