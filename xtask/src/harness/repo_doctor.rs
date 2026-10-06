//! Harness Modul: gebündeltes Session-Start-Dashboard (Git-Zustand, Workspace-Konsistenz, Unsafe/Panic-Kennzahlen, Branch-Überschneidung) (repo-doctor).

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepoDoctorFinding {
    pub severity: String,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoDoctorGateResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<RepoDoctorFinding>,
}

pub fn run_repo_doctor(args: &[String]) -> i32 {
    let mut root_dir = find_repo_root();
    let mut use_json = false;
    let mut quick = false;

    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                if idx + 1 < args.len() {
                    root_dir = PathBuf::from(&args[idx + 1]);
                    idx += 1;
                }
            }
            "--json" => {
                use_json = true;
            }
            "--quick" => {
                quick = true;
            }
            _ => {}
        }
        idx += 1;
    }

    let mut findings: Vec<RepoDoctorFinding> = Vec::new();

    // 1. Git-Arbeitsbaum-Status
    let is_git_repo = run_git_status_check(&root_dir, &mut findings);

    // 2. Workspace-Konsistenz
    run_workspace_verify_check(&root_dir, &mut findings);

    // 3. Env-Check
    run_env_check(&mut findings);

    if !quick {
        // 4. Branch-Überschneidung
        run_branch_overlap_check(&root_dir, &mut findings);

        // 5. Commit-Health
        run_commit_health_check(&root_dir, &mut findings);
    } else {
        findings.push(RepoDoctorFinding {
            severity: "info".to_string(),
            source: "repo_doctor".to_string(),
            message:
                "Schritte 4-5 (Branch-Überschneidung und Commit-Health) wegen --quick übersprungen."
                    .to_string(),
        });
    }

    // 6. Aggregiere alle Teilergebnisse zu einer EINZIGEN priorisierten Warnliste:
    // kritische Befunde zuerst, dann Warnungen, dann Informationen.
    findings.sort_by_key(|f| match f.severity.as_str() {
        "kritisch" => 0,
        "warnung" => 1,
        "info" => 2,
        _ => 3,
    });

    let has_critical = findings.iter().any(|f| f.severity == "kritisch");
    let status = if has_critical { "fail" } else { "pass" };

    let crit_count = findings.iter().filter(|f| f.severity == "kritisch").count();
    let warn_count = findings.iter().filter(|f| f.severity == "warnung").count();
    let info_count = findings.iter().filter(|f| f.severity == "info").count();

    let summary = format!(
        "Repo Doctor Diagnose abgeschlossen: {} kritisch, {} Warnung(en), {} Info(s)",
        crit_count, warn_count, info_count
    );

    let result = RepoDoctorGateResult {
        gate: "repo-doctor".to_string(),
        status: status.to_string(),
        summary,
        findings,
    };

    if use_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| json!(result).to_string())
        );
    } else {
        println!("=== Gate repo-doctor: {} ===", result.status);
        println!("{}", result.summary);
        println!("Findings:");
        for f in &result.findings {
            println!(
                " [{}] ({}) {}",
                f.severity.to_uppercase(),
                f.source,
                f.message
            );
        }
    }

    if !is_git_repo {
        2
    } else if has_critical {
        1
    } else {
        0
    }
}

fn find_repo_root() -> PathBuf {
    if let Ok(output) = Command::new("git")
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

fn run_git_status_check(root: &Path, findings: &mut Vec<RepoDoctorFinding>) -> bool {
    let status_out = Command::new("git")
        .current_dir(root)
        .args(["status", "--porcelain"])
        .output();

    let is_repo = match status_out {
        Ok(ref out) if out.status.success() => true,
        _ => false,
    };

    if !is_repo {
        findings.push(RepoDoctorFinding {
            severity: "kritisch".to_string(),
            source: "git".to_string(),
            message: "Verzeichnis ist kein Git-Repository oder 'git status' ist fehlgeschlagen."
                .to_string(),
        });
        return false;
    }

    let stdout_str = String::from_utf8_lossy(&status_out.as_ref().unwrap().stdout);
    let changed_lines: Vec<&str> = stdout_str
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    if !changed_lines.is_empty() {
        findings.push(RepoDoctorFinding {
            severity: "warnung".to_string(),
            source: "git".to_string(),
            message: format!(
                "Uncommitted changes im Arbeitsbaum ({} Datei(en) berührt).",
                changed_lines.len()
            ),
        });
    } else {
        findings.push(RepoDoctorFinding {
            severity: "info".to_string(),
            source: "git".to_string(),
            message: "Arbeitsbaum ist sauber (keine uncommitted changes).".to_string(),
        });
    }

    // Git stash list
    if let Ok(stash_out) = Command::new("git")
        .current_dir(root)
        .args(["stash", "list"])
        .output()
    {
        if stash_out.status.success() {
            let stash_str = String::from_utf8_lossy(&stash_out.stdout);
            let stashes = stash_str.lines().filter(|l| !l.trim().is_empty()).count();
            if stashes > 0 {
                findings.push(RepoDoctorFinding {
                    severity: "info".to_string(),
                    source: "git".to_string(),
                    message: format!("{} Stash(es) im Git-Speicher vorhanden.", stashes),
                });
            }
        }
    }

    // Git rev-list ahead/behind
    if let Ok(rev_out) = Command::new("git")
        .current_dir(root)
        .args(["rev-list", "--left-right", "--count", "origin/main...HEAD"])
        .output()
    {
        if rev_out.status.success() {
            let count_str = String::from_utf8_lossy(&rev_out.stdout).trim().to_string();
            let parts: Vec<&str> = count_str.split_whitespace().collect();
            if parts.len() == 2 {
                let behind = parts[0];
                let ahead = parts[1];
                findings.push(RepoDoctorFinding {
                    severity: "info".to_string(),
                    source: "git".to_string(),
                    message: format!(
                        "Branch-Status gegenüber origin/main: {} dahinter, {} voraus.",
                        behind, ahead
                    ),
                });
            }
        }
    }

    true
}

fn run_workspace_verify_check(root: &Path, findings: &mut Vec<RepoDoctorFinding>) {
    let output_dir = root.join("target/repo_doctor_verify");
    let config = crate::workspace_verify::WorkspaceVerifyConfig {
        mode: crate::workspace_verify::VerifyMode::Fast,
        only: vec![],
        resume_from: None,
        stop_on_fail: false,
        run_clippy: false,
    };

    match crate::workspace_verify::run_workspace_verify_in_root(root, config, &output_dir) {
        Ok(results) => {
            let failed_crates: Vec<_> = results
                .iter()
                .filter(|r| matches!(r.check, crate::workspace_verify::StepResult::Fail(_)))
                .map(|r| r.crate_name.clone())
                .collect();

            if !failed_crates.is_empty() {
                findings.push(RepoDoctorFinding {
                    severity: "kritisch".to_string(),
                    source: "workspace_verify".to_string(),
                    message: format!(
                        "Workspace Check fehlgeschlagen in {} Crate(s): {}",
                        failed_crates.len(),
                        failed_crates.join(", ")
                    ),
                });
            } else {
                findings.push(RepoDoctorFinding {
                    severity: "info".to_string(),
                    source: "workspace_verify".to_string(),
                    message: format!(
                        "Workspace-Konsistenz ok ({} Crates geprüft).",
                        results.len()
                    ),
                });
            }
        }
        Err(err_msg) => {
            findings.push(RepoDoctorFinding {
                severity: "kritisch".to_string(),
                source: "workspace_verify".to_string(),
                message: format!(
                    "Workspace-Konsistenzprüfung ergab Fehler oder Member-Drift: {}",
                    err_msg
                ),
            });
        }
    }
}

fn run_env_check(findings: &mut Vec<RepoDoctorFinding>) {
    let checks = crate::env_validate::run_env_validate();
    let mut missing_req = Vec::new();
    let mut ok_tools = Vec::new();

    for check in checks {
        match check.status {
            crate::env_validate::ToolStatus::Ok(ver) => {
                ok_tools.push(format!("{} ({})", check.name, ver));
            }
            crate::env_validate::ToolStatus::Missing => {
                if check.required {
                    missing_req.push(format!("{} (fehlt)", check.name));
                }
            }
            crate::env_validate::ToolStatus::WrongVersion { found, required } => {
                if check.required {
                    missing_req.push(format!(
                        "{} (gefunden {}, benötigt {})",
                        check.name, found, required
                    ));
                }
            }
        }
    }

    if !missing_req.is_empty() {
        findings.push(RepoDoctorFinding {
            severity: "warnung".to_string(),
            source: "env_validate".to_string(),
            message: format!(
                "Erforderliche Umgebungstools fehlen oder haben falsche Version: {}",
                missing_req.join(", ")
            ),
        });
    } else {
        findings.push(RepoDoctorFinding {
            severity: "info".to_string(),
            source: "env_validate".to_string(),
            message: format!(
                "Umgebungstools geprüft ok: {} Tool(s) bereit.",
                ok_tools.len()
            ),
        });
    }
}

fn run_branch_overlap_check(root: &Path, findings: &mut Vec<RepoDoctorFinding>) {
    let branch_args = vec!["--root".to_string(), root.display().to_string()];
    let overlap_ok =
        crate::agent_lifecycle::branch_overlap::run_check_branch_overlap_with_args(&branch_args);

    if overlap_ok {
        findings.push(RepoDoctorFinding {
            severity: "info".to_string(),
            source: "branch_overlap".to_string(),
            message: "Branch-Überschneidung ok: Keine Konflikte mit claim/* Remote-Branches."
                .to_string(),
        });
    } else {
        findings.push(RepoDoctorFinding {
            severity: "warnung".to_string(),
            source: "branch_overlap".to_string(),
            message: "Branch-Überschneidungs-Check ergab Scope-Überlappung oder Remote-Branch konnte nicht verifiziert werden.".to_string(),
        });
    }
}

fn run_commit_health_check(root: &Path, findings: &mut Vec<RepoDoctorFinding>) {
    match crate::commit_health::run_commit_health_impl(root, 7, None) {
        Ok(_report_md) => {
            findings.push(RepoDoctorFinding {
                severity: "info".to_string(),
                source: "commit_health".to_string(),
                message: "Commit-Gesundheitsbericht der letzten 7 Tage erfolgreich erstellt."
                    .to_string(),
            });
        }
        Err(err) => {
            findings.push(RepoDoctorFinding {
                severity: "warnung".to_string(),
                source: "commit_health".to_string(),
                message: format!("Commit-Gesundheitsbericht fehlgeschlagen: {}", err),
            });
        }
    }
}
