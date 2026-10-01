//! Harness Modul zur Risikobewertung von Änderungen (risk).

use serde_json::json;
use std::path::{Path, PathBuf};

// Gewichte & Schwellenwerte für deterministischen Risiko-Score (0-100)
// Die Werte sind per Test fixiert und duerfen nicht zufaellig variieren.
const SCORE_TIER_SEC: u32 = 40;
const SCORE_TIER_CRASH: u32 = 30;
const SCORE_TIER_SIMD: u32 = 20;
const SCORE_WAL_CRYPTO_PATH: u32 = 20;
const SCORE_UNSAFE_ISLAND: u32 = 25;
const SCORE_CARGO_TOML: u32 = 15;
const SCORE_PROTECTED_PATH: u32 = 30;
const SCORE_LARGE_DIFF: u32 = 10;

pub fn run_risk(args: &[String]) -> i32 {
    let mut root_dir = risk_find_repo_root();
    let mut base_rev = "origin/main".to_string();
    let mut head_rev = "HEAD".to_string();
    let mut use_json = false;

    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                if idx + 1 < args.len() {
                    root_dir = PathBuf::from(&args[idx + 1]);
                    idx += 1;
                }
            }
            "--base" => {
                if idx + 1 < args.len() {
                    base_rev = args[idx + 1].clone();
                    idx += 1;
                }
            }
            "--head" => {
                if idx + 1 < args.len() {
                    head_rev = args[idx + 1].clone();
                    idx += 1;
                }
            }
            "--json" => {
                use_json = true;
            }
            _ => {}
        }
        idx += 1;
    }

    let changed_files = risk_get_changed_files(&root_dir, &base_rev, &head_rev);
    let mut score: u32 = 0;
    let mut required_gates = Vec::new();
    let mut max_tier = "none";

    let mut touched_wal_crypto = false;
    let mut touched_unsafe = false;
    let mut touched_cargo = false;
    let mut touched_protected = false;

    for file in &changed_files {
        let f_lower = file.to_lowercase();

        if f_lower.contains("crypto") || f_lower.contains("privacy") {
            max_tier = "sec";
            touched_wal_crypto = true;
        } else if f_lower.contains("wal")
            || f_lower.contains("store")
            || f_lower.contains("checkpoint")
        {
            if max_tier != "sec" {
                max_tier = "crash";
            }
            touched_wal_crypto = true;
        } else if f_lower.contains("simd") || f_lower.contains("vector") {
            if max_tier != "sec" && max_tier != "crash" {
                max_tier = "simd";
            }
        }

        if f_lower.contains("unsafe") || f_lower.contains("mmap") {
            touched_unsafe = true;
        }

        if f_lower.ends_with("cargo.toml") || f_lower.ends_with("cargo.lock") {
            touched_cargo = true;
        }

        if f_lower.starts_with(".github/") || f_lower == "constitution.md" || f_lower == "deny.toml"
        {
            touched_protected = true;
        }
    }

    match max_tier {
        "sec" => {
            score += SCORE_TIER_SEC;
            required_gates.push("mutants-diff".to_string());
            required_gates.push("fuzz-smoke".to_string());
        }
        "crash" => {
            score += SCORE_TIER_CRASH;
            required_gates.push("wal-replay-verify".to_string());
        }
        "simd" => {
            score += SCORE_TIER_SIMD;
        }
        _ => {}
    }

    if touched_wal_crypto {
        score += SCORE_WAL_CRYPTO_PATH;
    }

    if touched_unsafe {
        score += SCORE_UNSAFE_ISLAND;
        required_gates.push("unsafe-audit".to_string());
    }

    if touched_cargo {
        score += SCORE_CARGO_TOML;
        required_gates.push("dependency-gate".to_string());
    }

    if touched_protected {
        score += SCORE_PROTECTED_PATH;
        required_gates.push("protected-paths".to_string());
    }

    if changed_files.len() > 10 {
        score += SCORE_LARGE_DIFF;
        required_gates.push("diff-budget".to_string());
    }

    score = score.min(100);

    let level = if score >= 80 {
        "kritisch"
    } else if score >= 50 {
        "hoch"
    } else if score >= 20 {
        "mittel"
    } else {
        "niedrig"
    };

    required_gates.sort();
    required_gates.dedup();

    if use_json {
        println!(
            "{}",
            json!({
                "gate": "risk",
                "status": "pass",
                "summary": format!("Risiko-Score: {}/100 ({})", score, level),
                "findings": [{
                    "id": "risk-assessment",
                    "severity": if score >= 50 { "warn" } else { "info" },
                    "file": "",
                    "line": 0,
                    "message": format!("Score: {}, Stufe: {}, Pflicht-Gates: {}", score, level, required_gates.join(", ")),
                    "fix": "Führe alle geforderten Pflicht-Gates aus."
                }]
            })
        );
    } else {
        println!("=== Risikobewertung ===");
        println!("Score: {}/100", score);
        println!("Stufe: {}", level);
        println!("Pflicht-Gates:");
        for g in &required_gates {
            println!(" - {}", g);
        }
    }

    0
}

fn risk_find_repo_root() -> PathBuf {
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return PathBuf::from(path_str);
            }
        }
    }
    PathBuf::from(".")
}

fn risk_get_changed_files(root: &Path, base: &str, head: &str) -> Vec<String> {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(["diff", "--name-only", base, head])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    Vec::new()
}
