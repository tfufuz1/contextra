//! Single non-negotiable merge verdict engine.

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct VerdictRequiredConfig {
    #[serde(default)]
    gate: Vec<VerdictGateConfig>,
}

#[derive(Debug, Deserialize)]
struct VerdictGateConfig {
    name: String,
    blocking: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct VerdictFinding {
    id: String,
    severity: String,
    file: String,
    line: usize,
    message: String,
    fix: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct VerdictGateReport {
    gate: String,
    status: String,
    summary: String,
    #[serde(default)]
    findings: Vec<VerdictFinding>,
}

#[derive(Debug, Clone)]
struct VerdictFailureReason {
    gate_name: String,
    severity: String, // "error" or "warn"
    message: String,
    fix: String,
}

#[derive(Debug, Serialize)]
struct VerdictOutputJson {
    gate: String,
    status: String,
    summary: String,
    findings: Vec<VerdictFinding>,
}

pub fn run_verdict(args: &[String]) -> i32 {
    let mut results_dir = PathBuf::from("gate-results");
    let mut required_toml = PathBuf::from("governance/verdict-required.toml");
    let mut is_json = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--results-dir" => {
                if i + 1 < args.len() {
                    results_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--required" => {
                if i + 1 < args.len() {
                    required_toml = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--root" | "--base" | "--head" => {
                if i + 1 < args.len() {
                    i += 1;
                }
            }
            "--json" => {
                is_json = true;
            }
            _ => {}
        }
        i += 1;
    }

    let config_content = match fs::read_to_string(&required_toml) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "VERDICT FEHLER: Required-TOML konnte nicht gelesen werden ({}): {}",
                required_toml.display(),
                e
            );
            return 2;
        }
    };

    let config: VerdictRequiredConfig = match toml::from_str(&config_content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "VERDICT FEHLER: Required-TOML konnte nicht parsiert werden ({}): {}",
                required_toml.display(),
                e
            );
            return 2;
        }
    };

    let mut failure_reasons: Vec<VerdictFailureReason> = Vec::new();
    let mut warning_reasons: Vec<VerdictFailureReason> = Vec::new();
    let mut passed_count = 0;
    let mut total_blocking = 0;

    for gate_cfg in &config.gate {
        if gate_cfg.blocking {
            total_blocking += 1;
        }

        let json_file_name = format!("{}.json", gate_cfg.name);
        let found_artifact = verdict_find_artifact(&results_dir, &json_file_name);

        match found_artifact {
            None => {
                let reason = VerdictFailureReason {
                    gate_name: gate_cfg.name.clone(),
                    severity: "error".to_string(),
                    message: format!("Artefakt '{}.json' fehlt im Ordner '{}'", gate_cfg.name, results_dir.display()),
                    fix: format!("Stelle sicher, dass der Job für '{}' ausgeführt wurde und das Artefakt hochlädt.", gate_cfg.name),
                };
                if gate_cfg.blocking {
                    failure_reasons.push(reason);
                } else {
                    warning_reasons.push(reason);
                }
            }
            Some(artifact_path) => match fs::read_to_string(&artifact_path) {
                Err(e) => {
                    let reason = VerdictFailureReason {
                        gate_name: gate_cfg.name.clone(),
                        severity: "error".to_string(),
                        message: format!(
                            "Artefakt '{}' konnte nicht gelesen werden: {}",
                            artifact_path.display(),
                            e
                        ),
                        fix: format!("Prüfe die Job-Ausgabe für '{}'.", gate_cfg.name),
                    };
                    if gate_cfg.blocking {
                        failure_reasons.push(reason);
                    } else {
                        warning_reasons.push(reason);
                    }
                }
                Ok(content) => match serde_json::from_str::<VerdictGateReport>(&content) {
                    Err(e) => {
                        let reason = VerdictFailureReason {
                            gate_name: gate_cfg.name.clone(),
                            severity: "error".to_string(),
                            message: format!(
                                "Ungültiges JSON-Schema in '{}': {}",
                                artifact_path.display(),
                                e
                            ),
                            fix: format!(
                                "Verifiziere, dass '{}' valide JSON-Ergebnisse liefert.",
                                gate_cfg.name
                            ),
                        };
                        if gate_cfg.blocking {
                            failure_reasons.push(reason);
                        } else {
                            warning_reasons.push(reason);
                        }
                    }
                    Ok(report) => {
                        if report.gate != gate_cfg.name {
                            let reason = VerdictFailureReason {
                                        gate_name: gate_cfg.name.clone(),
                                        severity: "error".to_string(),
                                        message: format!("Gate-Name im JSON ('{}') stimmt nicht mit Dateiname ('{}') überein.", report.gate, gate_cfg.name),
                                        fix: format!("Korrigiere den übergebenen Gate-Namen beim Aufruf von '{}'.", gate_cfg.name),
                                    };
                            if gate_cfg.blocking {
                                failure_reasons.push(reason);
                            } else {
                                warning_reasons.push(reason);
                            }
                        } else {
                            match report.status.as_str() {
                                "pass" | "not_applicable" => {
                                    if gate_cfg.blocking {
                                        passed_count += 1;
                                    }
                                }
                                _ => {
                                    let first_msg = report
                                        .findings
                                        .first()
                                        .map(|f| f.message.clone())
                                        .unwrap_or_else(|| report.summary.clone());
                                    let first_fix = report
                                        .findings
                                        .first()
                                        .map(|f| f.fix.clone())
                                        .unwrap_or_else(|| "Siehe Details im Job-Log.".to_string());
                                    let reason = VerdictFailureReason {
                                        gate_name: gate_cfg.name.clone(),
                                        severity: "error".to_string(),
                                        message: first_msg,
                                        fix: first_fix,
                                    };
                                    if gate_cfg.blocking {
                                        failure_reasons.push(reason);
                                    } else {
                                        warning_reasons.push(reason);
                                    }
                                }
                            }
                        }
                    }
                },
            },
        }
    }

    let is_green = failure_reasons.is_empty();

    // Sort reasons deterministically: severity descending ("error" before "warn"), then gate_name ascending
    failure_reasons.sort_by(|a, b| {
        a.severity
            .cmp(&b.severity)
            .reverse()
            .then_with(|| a.gate_name.cmp(&b.gate_name))
    });
    warning_reasons.sort_by(|a, b| {
        a.severity
            .cmp(&b.severity)
            .reverse()
            .then_with(|| a.gate_name.cmp(&b.gate_name))
    });

    let top_reasons = failure_reasons.iter().take(3).cloned().collect::<Vec<_>>();

    if is_json {
        let findings = top_reasons
            .iter()
            .enumerate()
            .map(|(idx, r)| VerdictFinding {
                id: format!("VERDICT-{}", idx + 1),
                severity: "error".to_string(),
                file: r.gate_name.clone(),
                line: 0,
                message: r.message.clone(),
                fix: r.fix.clone(),
            })
            .collect::<Vec<_>>();

        let output = VerdictOutputJson {
            gate: "verdict".to_string(),
            status: if is_green {
                "pass".to_string()
            } else {
                "fail".to_string()
            },
            summary: if is_green {
                format!(
                    "GRÜN: Alle {} blockierenden Gates bestanden ({} passed).",
                    total_blocking, passed_count
                )
            } else {
                format!(
                    "ROT: {} blockierende Gates fehlgeschlagen.",
                    failure_reasons.len()
                )
            },
            findings,
        };

        if let Ok(json_str) = serde_json::to_string_pretty(&output) {
            println!("{}", json_str);
        }
    } else {
        println!(
            "=== MERGE VERDICT: {} ===",
            if is_green { "GRÜN" } else { "ROT" }
        );
        if is_green {
            println!(
                "Alle {} blockierenden Gates erfolgreich bestanden.",
                total_blocking
            );
            if !warning_reasons.is_empty() {
                println!(
                    "\nHinweis: {} nicht-blockierende Gates haben Fehler/Warnungen gemeldet:",
                    warning_reasons.len()
                );
                for w in &warning_reasons {
                    println!("  - Gate [{}]: {}", w.gate_name, w.message);
                }
            }
        } else {
            println!(
                "Fehlgeschlagene blockierende Gates ({} insg.):",
                failure_reasons.len()
            );
            for (idx, r) in top_reasons.iter().enumerate() {
                println!("\nReason #{}:", idx + 1);
                println!("  WAS: Gate [{}] schlug fehl", r.gate_name);
                println!("  WARUM: {}", r.message);
                println!("  FIX: {}", r.fix);
            }
            if failure_reasons.len() > 3 {
                println!(
                    "\n... und {} weitere blockierende Fehler.",
                    failure_reasons.len() - 3
                );
            }
        }
    }

    // Write to $GITHUB_STEP_SUMMARY if set
    if let Ok(summary_path) = env::var("GITHUB_STEP_SUMMARY") {
        let mut md = String::new();
        md.push_str(&format!(
            "# Merge Verdict: {}\n\n",
            if is_green { "🟢 GRÜN" } else { "🔴 ROT" }
        ));
        if is_green {
            md.push_str(&format!(
                "Alle **{}** blockierenden Quality Gates sind erfolgreich gelaufen.\n",
                total_blocking
            ));
            if !warning_reasons.is_empty() {
                md.push_str("\n### Nicht-blockierende Hinweise\n");
                for w in &warning_reasons {
                    md.push_str(&format!("- **{}**: {}\n", w.gate_name, w.message));
                }
            }
        } else {
            md.push_str(&format!(
                "Es sind **{}** blockierende Gate-Fehler aufgetreten.\n\n",
                failure_reasons.len()
            ));
            md.push_str("### Hauptgründe\n\n");
            for (idx, r) in top_reasons.iter().enumerate() {
                md.push_str(&format!("#### {}. Gate `{}`\n", idx + 1, r.gate_name));
                md.push_str(&format!("- **WAS**: Gate `{}` schlug fehl\n", r.gate_name));
                md.push_str(&format!("- **WARUM**: {}\n", r.message));
                md.push_str(&format!("- **FIX**: {}\n\n", r.fix));
            }
        }
        let _ = fs::write(summary_path, md);
    }

    if is_green {
        0
    } else {
        1
    }
}

fn verdict_find_artifact(dir: &Path, file_name: &str) -> Option<PathBuf> {
    if !dir.exists() {
        return None;
    }
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.file_name() == file_name {
            return Some(entry.path().to_path_buf());
        }
    }
    None
}
