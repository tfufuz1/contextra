//! Harness Modul zur Erkennung von Test-Flakes (flake-report).

use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn run_flake_report(args: &[String]) -> i32 {
    let mut root_dir = flake_report_find_repo_root();
    let mut runs = 10;
    let mut target_crate: Option<String> = None;
    let mut target_tests: Option<String> = None;
    let mut use_json = false;

    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--runs" => {
                if idx + 1 < args.len() {
                    runs = args[idx + 1].parse::<usize>().unwrap_or(10);
                    idx += 1;
                }
            }
            "--crate" => {
                if idx + 1 < args.len() {
                    target_crate = Some(args[idx + 1].clone());
                    idx += 1;
                }
            }
            "--tests" => {
                if idx + 1 < args.len() {
                    target_tests = Some(args[idx + 1].clone());
                    idx += 1;
                }
            }
            "--root" => {
                if idx + 1 < args.len() {
                    root_dir = PathBuf::from(&args[idx + 1]);
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

    let nextest_check = std::process::Command::new("cargo")
        .args(["nextest", "--version"])
        .output();

    let has_nextest = matches!(nextest_check, Ok(out) if out.status.success());
    if !has_nextest {
        let msg = "cargo-nextest ist nicht im PATH verfuegbar.";
        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "flake-report",
                    "status": "error",
                    "summary": msg,
                    "findings": [{
                        "id": "nextest-missing",
                        "severity": "error",
                        "file": "",
                        "line": 0,
                        "message": msg,
                        "fix": "Installiere cargo-nextest via 'cargo install cargo-nextest'."
                    }]
                })
            );
        } else {
            eprintln!("FEHLER: {}\nInstallationshinweis: cargo install cargo-nextest", msg);
        }
        return 2;
    }

    let mut test_success_count: HashMap<String, usize> = HashMap::new();

    for run_idx in 1..=runs {
        let mut cmd = std::process::Command::new("cargo");
        cmd.current_dir(&root_dir);
        cmd.args(["nextest", "run", "--retries", "0", "--no-fail-fast"]);

        if let Some(ref c) = target_crate {
            cmd.args(["-p", c]);
        }
        if let Some(ref t) = target_tests {
            cmd.arg(t);
        }

        let output = cmd.output();
        let success = matches!(output, Ok(out) if out.status.success());

        let run_key = format!("suite_run_{}", run_idx);
        if success {
            *test_success_count.entry(run_key).or_default() += 1;
        }
    }

    let local_dir = root_dir.join(".jules/local");
    let _ = fs::create_dir_all(&local_dir);
    let report_file = local_dir.join("flake-report.json");

    let mut flaky_detected = false;
    let mut findings = Vec::new();

    let total_successful_runs = test_success_count.len();
    let rate = total_successful_runs as f64 / runs as f64;

    if rate > 0.0 && rate < 1.0 {
        flaky_detected = true;
        findings.push(json!({
            "id": "flaky-test-suite",
            "severity": "warn",
            "file": "",
            "line": 0,
            "message": format!("Testsuite zeigte Flakiness: {}/{} Läufe erfolgreich (Erfolgsquote: {:.1}%)", total_successful_runs, runs, rate * 100.0),
            "fix": "Analysiere nicht-deterministische Test-Ursachen (z. B. Race Conditions, Timeouts)."
        }));
    }

    let report_json = json!({
        "runs": runs,
        "successful_runs": total_successful_runs,
        "success_rate": rate,
        "flaky_detected": flaky_detected
    });

    let _ = fs::write(&report_file, serde_json::to_string_pretty(&report_json).unwrap_or_default());

    if use_json {
        println!(
            "{}",
            json!({
                "gate": "flake-report",
                "status": if flaky_detected { "fail" } else { "pass" },
                "summary": format!("Flake-Report abgeschlossen ({} Läufe, Erfolgsquote {:.1}%)", runs, rate * 100.0),
                "findings": findings
            })
        );
    } else {
        println!("=== Flake Report ===");
        println!("Läufe: {}", runs);
        println!("Erfolgreiche Läufe: {}", total_successful_runs);
        println!("Erfolgsquote: {:.1}%", rate * 100.0);
        if flaky_detected {
            println!("WARNUNG: Test-Flakiness erkannt!");
        } else {
            println!("Keine Flakiness festgestellt.");
        }
    }

    0
}

fn flake_report_find_repo_root() -> PathBuf {
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
