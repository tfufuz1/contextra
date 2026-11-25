//! Session report generator for PR body documentation.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionReportGateFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionReportGateResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<SessionReportGateFinding>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionReportFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub fix: String,
}

pub fn session_report_get_diff_files(root: &Path, base: &str, head: &str) -> Vec<String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["diff", "--name-only", base, head])
        .output();

    if let Ok(o) = out {
        if o.status.success() {
            let text = String::from_utf8_lossy(&o.stdout);
            return text
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
        }
    }
    Vec::new()
}

pub fn session_report_generate(
    root: &Path,
    base: &str,
    head: &str,
    card_path: Option<&Path>,
    goal_text: Option<&str>,
    results_dir: Option<&Path>,
    findings_path: Option<&Path>,
) -> String {
    let mut body = String::new();

    // 1. Ziel
    body.push_str("## Ziel\n");
    let mut goal_found = false;
    if let Some(card_p) = card_path {
        if let Ok(card_content) = fs::read_to_string(card_p) {
            body.push_str(card_content.trim());
            body.push_str("\n\n");
            goal_found = true;
        }
    }
    if !goal_found {
        if let Some(goal) = goal_text {
            body.push_str(goal.trim());
            body.push_str("\n\n");
            goal_found = true;
        }
    }
    if !goal_found {
        body.push_str("KEIN BELEG\n\n");
    }

    // 2. Änderungen
    body.push_str("## Änderungen\n");
    let diff_files = session_report_get_diff_files(root, base, head);
    if diff_files.is_empty() {
        body.push_str("KEIN BELEG\n\n");
    } else {
        for f in &diff_files {
            body.push_str(&format!("- {}\n", f));
        }
        body.push('\n');
    }

    // 3. Invarianten berührt
    body.push_str("## Invarianten berührt\n");
    let mut inv_found = false;
    if let Some(card_p) = card_path {
        if let Ok(content) = fs::read_to_string(card_p) {
            let inv_lines: Vec<&str> = content
                .lines()
                .filter(|l| l.contains("INV-") || l.contains("ADR-"))
                .collect();
            if !inv_lines.is_empty() {
                for line in inv_lines {
                    body.push_str(line.trim());
                    body.push('\n');
                }
                body.push('\n');
                inv_found = true;
            }
        }
    }
    if !inv_found {
        body.push_str("Keine spezifischen Invarianten berührt.\n\n");
    }

    // 4. Verifikation
    body.push_str("## Verifikation\n");
    let mut ver_found = false;
    if let Some(res_dir) = results_dir {
        if let Ok(entries) = fs::read_dir(res_dir) {
            let mut results = Vec::new();
            for entry_res in entries.flatten() {
                let p = entry_res.path();
                if p.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(c) = fs::read_to_string(&p) {
                        if let Ok(rep) = serde_json::from_str::<SessionReportGateResult>(&c) {
                            results.push(rep);
                        }
                    }
                }
            }
            if !results.is_empty() {
                results.sort_by(|a, b| a.gate.cmp(&b.gate));
                for r in results {
                    body.push_str(&format!(
                        "- **{}**: {} — {}\n",
                        r.gate,
                        r.status.to_uppercase(),
                        r.summary
                    ));
                }
                body.push('\n');
                ver_found = true;
            }
        }
    }
    if !ver_found {
        body.push_str("KEIN BELEG\n\n");
    }

    // 5. Out-of-scope Findings
    body.push_str("## Out-of-scope Findings\n");
    let mut findings_found = false;
    if let Some(find_p) = findings_path {
        if let Ok(content) = fs::read_to_string(find_p) {
            if !content.trim().is_empty() {
                body.push_str(content.trim());
                body.push_str("\n\n");
                findings_found = true;
            }
        }
    }
    if !findings_found {
        body.push_str("Keine Out-of-scope Findings identifiziert.\n\n");
    }

    // 6. Risiken/Rollback
    body.push_str("## Risiken/Rollback\n");
    body.push_str(
        "- **Risiko**: Niedrig (Rein isolierte Harness- und Pre-Submit-Gate-Erweiterungen).\n",
    );
    body.push_str("- **Rollback-Strategie**: `git revert` des Commits stellt den vorherigen Zustand vollständig wieder her.\n\n");

    // 7. ADR/Spec-Sync
    body.push_str("## ADR/Spec-Sync\n");
    body.push_str("Kein ADR- / Spec-Update erforderlich. Harness-Anpassungen entsprechen dem allgemeinen Harness-Vertrag.\n");

    body
}

pub fn run_session_report(args: &[String]) -> i32 {
    let mut root_path = match Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        Ok(out) if out.status.success() => {
            PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string())
        }
        _ => PathBuf::from("."),
    };

    let mut base_rev = match Command::new("git")
        .args(["merge-base", "HEAD", "origin/main"])
        .output()
    {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "HEAD~1".to_string(),
    };
    if base_rev.is_empty() {
        base_rev = "HEAD~1".to_string();
    }

    let mut head_rev = "HEAD".to_string();
    let mut json_output = false;
    let mut card_path = None;
    let mut goal_text = None;
    let mut results_dir = None;
    let mut findings_path = None;

    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        match arg.as_str() {
            "--root" => {
                if idx + 1 < args.len() {
                    root_path = PathBuf::from(&args[idx + 1]);
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
                json_output = true;
            }
            "--card" => {
                if idx + 1 < args.len() {
                    card_path = Some(PathBuf::from(&args[idx + 1]));
                    idx += 1;
                }
            }
            "--goal" => {
                if idx + 1 < args.len() {
                    goal_text = Some(args[idx + 1].clone());
                    idx += 1;
                }
            }
            "--results-dir" => {
                if idx + 1 < args.len() {
                    results_dir = Some(PathBuf::from(&args[idx + 1]));
                    idx += 1;
                }
            }
            "--findings" => {
                if idx + 1 < args.len() {
                    findings_path = Some(PathBuf::from(&args[idx + 1]));
                    idx += 1;
                }
            }
            _ => {}
        }
        idx += 1;
    }

    let pr_body = session_report_generate(
        &root_path,
        &base_rev,
        &head_rev,
        card_path.as_deref(),
        goal_text.as_deref(),
        results_dir.as_deref(),
        findings_path.as_deref(),
    );

    let local_dir = root_path.join(".jules/local");
    let _ = fs::create_dir_all(&local_dir);
    let out_file = local_dir.join("PR_BODY.md");

    if let Err(e) = fs::write(&out_file, &pr_body) {
        if json_output {
            let out = serde_json::json!({
                "gate": "session-report",
                "status": "error",
                "summary": format!("Failed to write PR_BODY.md: {e}"),
                "findings": []
            });
            println!("{}", out);
        } else {
            eprintln!("WAS: Schreiben von PR_BODY.md fehlgeschlagen.\nWARUM: {e}\nFIX: Prüfe Schreibrechte in .jules/local/.");
        }
        return 2;
    }

    if json_output {
        let out = serde_json::json!({
            "gate": "session-report",
            "status": "pass",
            "summary": format!("PR_BODY.md generiert unter {}", out_file.display()),
            "findings": []
        });
        println!("{}", out);
    } else {
        println!("PR_BODY.md generiert unter {}", out_file.display());
    }

    0
}
