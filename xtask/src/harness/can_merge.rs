//! Harness Modul zur Merge-Entscheidungsfindung (can-merge).

use serde_json::json;
use std::fs;
use std::path::PathBuf;

pub fn run_can_merge(args: &[String]) -> i32 {
    let mut results_dir: Option<PathBuf> = None;
    let mut pr_nr: Option<String> = None;
    let mut use_json = false;

    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--results-dir" => {
                if idx + 1 < args.len() {
                    results_dir = Some(PathBuf::from(&args[idx + 1]));
                    idx += 1;
                }
            }
            "--pr" => {
                if idx + 1 < args.len() {
                    pr_nr = Some(args[idx + 1].clone());
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

    if results_dir.is_none() && pr_nr.is_none() {
        let msg = "Weder --results-dir noch --pr angegeben (Fail-closed).";
        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "can-merge",
                    "status": "error",
                    "summary": "NEIN",
                    "findings": [{
                        "id": "can-merge-missing-input",
                        "severity": "error",
                        "file": "",
                        "line": 0,
                        "message": msg,
                        "fix": "Gib --results-dir <dir> oder --pr <nr> an."
                    }]
                })
            );
        } else {
            eprintln!("NEIN: {}", msg);
        }
        return 2;
    }

    let mut failed_gates = Vec::new();

    if let Some(dir) = results_dir {
        if !dir.exists() || !dir.is_dir() {
            let msg = format!(
                "Ergebnisverzeichnis {:?} existiert nicht oder ist kein Ordner.",
                dir
            );
            if use_json {
                println!(
                    "{}",
                    json!({
                        "gate": "can-merge",
                        "status": "error",
                        "summary": "NEIN",
                        "findings": [{
                            "id": "can-merge-invalid-dir",
                            "severity": "error",
                            "file": "",
                            "line": 0,
                            "message": msg,
                            "fix": "Pruefe den Pfad des Ergebnisverzeichnisses."
                        }]
                    })
                );
            } else {
                eprintln!("NEIN: {}", msg);
            }
            return 2;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            let status = val
                                .get("status")
                                .and_then(|s| s.as_str())
                                .unwrap_or("error");
                            let gate = val
                                .get("gate")
                                .and_then(|s| s.as_str())
                                .unwrap_or("unknown");
                            let summary = val
                                .get("summary")
                                .and_then(|s| s.as_str())
                                .unwrap_or("Keine Zusammenfassung");

                            if status == "fail" || status == "error" {
                                failed_gates.push((
                                    gate.to_string(),
                                    summary.to_string(),
                                    status.to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    } else if let Some(pr) = pr_nr {
        let output = std::process::Command::new("gh")
            .args(["pr", "checks", &pr, "--json", "name,state,link"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                    if let Some(arr) = val.as_array() {
                        for item in arr {
                            let name = item
                                .get("name")
                                .and_then(|s| s.as_str())
                                .unwrap_or("unknown");
                            let state = item
                                .get("state")
                                .and_then(|s| s.as_str())
                                .unwrap_or("FAILURE");
                            if state != "SUCCESS" && state != "SKIPPED" {
                                failed_gates.push((
                                    name.to_string(),
                                    format!("PR Check State: {}", state),
                                    state.to_string(),
                                ));
                            }
                        }
                    }
                }
            }
            _ => {
                let msg =
                    "gh CLI ist nicht verfuegbar oder PR-Checks konnten nicht abgerufen werden.";
                if use_json {
                    println!(
                        "{}",
                        json!({
                            "gate": "can-merge",
                            "status": "error",
                            "summary": "NEIN",
                            "findings": [{
                                "id": "can-merge-gh-failed",
                                "severity": "error",
                                "file": "",
                                "line": 0,
                                "message": msg,
                                "fix": "Installiere gh CLI oder nutze --results-dir."
                            }]
                        })
                    );
                } else {
                    eprintln!(
                        "NEIN: {}\nInstallationshinweis: https://cli.github.com/",
                        msg
                    );
                }
                return 2;
            }
        }
    }

    if failed_gates.is_empty() {
        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "can-merge",
                    "status": "pass",
                    "summary": "JA",
                    "findings": []
                })
            );
        } else {
            println!("JA");
        }
        0
    } else {
        failed_gates.sort_by(|a, b| a.0.cmp(&b.0));
        let reasons_count = failed_gates.len().min(3);
        let top_reasons = &failed_gates[..reasons_count];

        let findings: Vec<_> = top_reasons
            .iter()
            .map(|(gate, cause, status)| {
                json!({
                    "id": gate,
                    "severity": if status == "error" { "error" } else { "warn" },
                    "file": "",
                    "line": 0,
                    "message": format!("Gate {} fehlgeschlagen: {}", gate, cause),
                    "fix": format!("cargo run --manifest-path xtask/Cargo.toml -- explain {}", gate)
                })
            })
            .collect();

        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "can-merge",
                    "status": "fail",
                    "summary": "NEIN",
                    "findings": findings
                })
            );
        } else {
            println!("NEIN");
            println!("Gruende (max. 3):");
            for (gate, cause, _) in top_reasons {
                println!(" - Gate {}: {}", gate, cause);
                println!(
                    "   Fix: cargo run --manifest-path xtask/Cargo.toml -- explain {}",
                    gate
                );
            }
        }
        1
    }
}
