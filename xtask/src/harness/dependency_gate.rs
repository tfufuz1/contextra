//! Harness Modul zur Prüfung neuer Abhängigkeiten (dependency-gate).

use serde_json::json;
use std::path::{Path, PathBuf};

pub fn run_dependency_gate(args: &[String]) -> i32 {
    let mut root_dir = dependency_gate_find_repo_root();
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

    let changed_files = dependency_gate_get_changed_files(&root_dir, &base_rev, &head_rev);
    let cargo_changed = changed_files
        .iter()
        .any(|f| f.ends_with("Cargo.toml") || f.ends_with("Cargo.lock"));

    if !cargo_changed {
        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "dependency-gate",
                    "status": "not_applicable",
                    "summary": "Keine Änderungen an Cargo.toml / Cargo.lock",
                    "findings": []
                })
            );
        } else {
            println!("N/A: Keine Änderungen an Cargo.toml / Cargo.lock");
        }
        return 0;
    }

    let commit_msg = dependency_gate_get_commit_message(&root_dir, &head_rev);
    let has_reason_trailer = commit_msg
        .lines()
        .any(|line| line.trim().starts_with("Dependency-Reason:"));

    if !has_reason_trailer {
        let msg = "Fehlender Begründungstrailer 'Dependency-Reason: <text>' im Commit.";
        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "dependency-gate",
                    "status": "fail",
                    "summary": msg,
                    "findings": [{
                        "id": "dependency-missing-trailer",
                        "severity": "error",
                        "file": "Cargo.toml",
                        "line": 0,
                        "message": msg,
                        "fix": "Füge die Zeile 'Dependency-Reason: <Begründung>' zu deiner Commit-Message hinzu."
                    }]
                })
            );
        } else {
            eprintln!("VERSTOß: {}", msg);
        }
        return 1;
    }

    if use_json {
        println!(
            "{}",
            json!({
                "gate": "dependency-gate",
                "status": "pass",
                "summary": "Abhängigkeitsprüfung erfolgreich bestanden.",
                "findings": []
            })
        );
    } else {
        println!("Pass: Abhängigkeitsprüfung erfolgreich.");
    }

    0
}

fn dependency_gate_find_repo_root() -> PathBuf {
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

fn dependency_gate_get_changed_files(root: &Path, base: &str, head: &str) -> Vec<String> {
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

fn dependency_gate_get_commit_message(root: &Path, rev: &str) -> String {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(["log", "-1", "--format=%B", rev])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).to_string();
        }
    }
    String::new()
}
