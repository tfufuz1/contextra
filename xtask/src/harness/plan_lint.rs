//! Plan linting gate to validate Jules generated execution plans against task cards.

use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct CardDataLocal {
    pub scope: Vec<String>,
    pub forbidden: Vec<String>,
    pub acceptance: Vec<String>,
    pub evidence_required: Vec<String>,
    pub protected_change: Option<String>,
}

pub trait PlanLintHttpTransport {
    fn approve_plan(
        &self,
        url: &str,
        header: &str,
        api_key: &str,
        body: &str,
    ) -> Result<String, String>;
}

pub struct PlanLintCurlTransport;

impl PlanLintHttpTransport for PlanLintCurlTransport {
    fn approve_plan(
        &self,
        url: &str,
        header: &str,
        api_key: &str,
        body: &str,
    ) -> Result<String, String> {
        let auth_header = format!("{}: {}", header, api_key);
        let output = std::process::Command::new("curl")
            .args([
                "-s",
                "-S",
                "-X",
                "POST",
                "-H",
                &auth_header,
                "-H",
                "Content-Type: application/json",
                "-d",
                body,
                url,
            ])
            .output()
            .map_err(|e| e.to_string())?;

        let res = String::from_utf8_lossy(&output.stdout).to_string();
        if !output.status.success() {
            let err_raw = String::from_utf8_lossy(&output.stderr).to_string();
            let redacted = if api_key.is_empty() {
                err_raw
            } else {
                err_raw.replace(api_key, "[REDACTED_API_KEY]")
            };
            return Err(format!("curl error: {}", redacted));
        }
        Ok(res)
    }
}

pub struct PlanLintFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub fix: String,
}

pub fn plan_lint_find_root(start: &Path) -> PathBuf {
    let mut current = start.to_path_buf();
    loop {
        if current.join("Cargo.toml").exists() && current.join("capabilities.toml").exists() {
            return current;
        }
        if !current.pop() {
            return start.to_path_buf();
        }
    }
}

pub fn plan_lint_check_plan_text(
    plan_text: &str,
    card_path: &Path,
) -> (bool, Vec<PlanLintFinding>) {
    let mut findings = Vec::new();
    let card_str = card_path.to_string_lossy().to_string();

    let card_content = match fs::read_to_string(card_path) {
        Ok(c) => c,
        Err(e) => {
            findings.push(PlanLintFinding {
                id: "card-read-error".to_string(),
                severity: "error".to_string(),
                file: card_str,
                line: 0,
                message: format!("Task-Karte konnte nicht gelesen werden: {}", e),
                fix: "Pfad prüfen".to_string(),
            });
            return (false, findings);
        }
    };

    let card: CardDataLocal = match toml::from_str(&card_content) {
        Ok(c) => c,
        Err(e) => {
            findings.push(PlanLintFinding {
                id: "card-parse-error".to_string(),
                severity: "error".to_string(),
                file: card_str,
                line: 0,
                message: format!("Task-Karte TOML-Fehler: {}", e),
                fix: "TOML korrigieren".to_string(),
            });
            return (false, findings);
        }
    };

    // Extract paths and commands from plan
    let mut paths_in_plan = HashSet::new();
    let mut commands_in_plan = Vec::new();

    for line in plan_text.lines() {
        let trimmed = line.trim();
        if trimmed.contains('`') {
            let parts: Vec<&str> = trimmed.split('`').collect();
            for (idx, part) in parts.iter().enumerate() {
                if idx % 2 == 1 {
                    let p = part.trim();
                    if p.contains('/') || p.ends_with(".rs") || p.ends_with(".toml") {
                        paths_in_plan.insert(p.to_string());
                    }
                    if p.starts_with("cargo ") || p.starts_with("just ") {
                        commands_in_plan.push(p.to_string());
                    }
                }
            }
        }
    }

    // Rule 1: Scope check
    let scope_patterns = &card.scope;
    for path in &paths_in_plan {
        let mut in_scope = false;
        for pattern in scope_patterns {
            let prefix = pattern.trim_end_matches('*').trim_end_matches('/');
            if path.starts_with(prefix) || pattern == path {
                in_scope = true;
                break;
            }
        }
        if !in_scope {
            findings.push(PlanLintFinding {
                id: "path-out-of-scope".to_string(),
                severity: "error".to_string(),
                file: path.clone(),
                line: 0,
                message: format!("Pfad '{}' im Plan liegt außerhalb des Katen-Scopes", path),
                fix: "Pfad aus dem Plan entfernen oder Scope der Karte erweitern".to_string(),
            });
        }
    }

    // Rule 2: Forbidden paths
    let forbidden_patterns = &card.forbidden;
    for path in &paths_in_plan {
        for f in forbidden_patterns {
            let prefix = f.trim_end_matches('*').trim_end_matches('/');
            if (path.starts_with(prefix) || f == path) && card.protected_change.is_none() {
                findings.push(PlanLintFinding {
                    id: "forbidden-path-in-plan".to_string(),
                    severity: "error".to_string(),
                    file: path.clone(),
                    line: 0,
                    message: format!("Plan berührt verbotenen Pfad '{}'", path),
                    fix: "Verbotenen Pfad aus dem Plan entfernen".to_string(),
                });
            }
        }
    }

    // Rule 3: Acceptance commands check
    for req_cmd in &card.acceptance {
        let cmd_found =
            plan_text.contains(req_cmd) || commands_in_plan.iter().any(|c| c == req_cmd);
        if !cmd_found {
            findings.push(PlanLintFinding {
                id: "missing-acceptance-command".to_string(),
                severity: "error".to_string(),
                file: card_str.clone(),
                line: 0,
                message: format!("Akzeptanzbefehl '{}' fehlt im Plan", req_cmd),
                fix: format!("Befehl '{}' explizit in den Plan aufnehmen", req_cmd),
            });
        }
    }

    // Rule 4: Dependency justification check
    let is_dep_change = plan_text.contains("Cargo.toml")
        || plan_text.to_lowercase().contains("add dependency")
        || plan_text.to_lowercase().contains("add crate");
    if is_dep_change
        && !plan_text.contains("Begründung")
        && !plan_text.to_lowercase().contains("justification")
    {
        findings.push(PlanLintFinding {
            id: "unjustified-dependency-change".to_string(),
            severity: "error".to_string(),
            file: "Cargo.toml".to_string(),
            line: 0,
            message: "Neue Dependency oder Cargo.toml-Änderung geplant ohne Begründungsabschnitt"
                .to_string(),
            fix: "Begründungsabschnitt im Plan ergänzen".to_string(),
        });
    }

    // Rule 5: Test-first check
    if !card.evidence_required.is_empty() {
        let has_test_step = plan_text.to_lowercase().contains("test")
            || plan_text.to_lowercase().contains("rot")
            || plan_text.to_lowercase().contains("failing");
        if !has_test_step {
            findings.push(PlanLintFinding {
                id: "missing-test-first-step".to_string(),
                severity: "error".to_string(),
                file: card_str.clone(),
                line: 0,
                message:
                    "evidence_required gefordert, aber kein Test-zuerst-Schritt im Plan gefunden"
                        .to_string(),
                fix: "Test-zuerst Schritt im Plan aufnehmen".to_string(),
            });
        }
    }

    let has_errors = findings.iter().any(|f| f.severity == "error");
    (!has_errors, findings)
}

pub fn plan_lint_process_session<T: PlanLintHttpTransport>(
    root: &Path,
    session_id_opt: Option<&str>,
    plan_text_arg: &str,
    card_path: &Path,
    approve: bool,
    transport: &T,
) -> (i32, serde_json::Value) {
    let session_id = session_id_opt.unwrap_or("sess-local");
    let session_file = root.join(format!(".jules/local/sessions/{}.json", session_id));
    let mut revision_counter = 0;
    let mut effective_plan_text = plan_text_arg.to_string();

    if session_file.exists() {
        if let Ok(content) = fs::read_to_string(&session_file) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                revision_counter = json
                    .get("revision_counter")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                if effective_plan_text.trim().is_empty() {
                    if let Some(p) = json.get("plan").and_then(|v| v.as_str()) {
                        effective_plan_text = p.to_string();
                    }
                }
            }
        }
    }

    let (success, findings) = plan_lint_check_plan_text(&effective_plan_text, card_path);

    if !success {
        revision_counter += 1;
        if session_file.exists() {
            if let Ok(content) = fs::read_to_string(&session_file) {
                if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                    json["revision_counter"] = serde_json::json!(revision_counter);
                    let _ = fs::write(
                        &session_file,
                        serde_json::to_string_pretty(&json).unwrap_or_default(),
                    );
                }
            }
        }

        if revision_counter >= 3 {
            let json = serde_json::json!({
                "gate": "plan-lint",
                "status": "error",
                "summary": "Eskalation: Karte zu unklar/zu groß — Task teilen (3. Ablehnung)",
                "revision_counter": revision_counter,
                "findings": []
            });
            return (2, json);
        }

        let json_findings: Vec<serde_json::Value> = findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "id": f.id,
                    "severity": f.severity,
                    "file": f.file,
                    "line": f.line,
                    "message": f.message,
                    "fix": f.fix
                })
            })
            .collect();

        let json = serde_json::json!({
            "gate": "plan-lint",
            "status": "fail",
            "summary": format!("Plan-Linting fehlgeschlagen (Revision {})", revision_counter),
            "revision_counter": revision_counter,
            "findings": json_findings
        });
        return (1, json);
    }

    if approve {
        if session_id_opt.is_none() {
            let json = serde_json::json!({
                "gate": "plan-lint",
                "status": "error",
                "summary": "--approve erfordert eine explizite --session <id>",
                "findings": []
            });
            return (2, json);
        }

        let api_key = match std::env::var("JULES_API_KEY") {
            Ok(k) if !k.trim().is_empty() => k,
            _ => {
                let json = serde_json::json!({
                    "gate": "plan-lint",
                    "status": "error",
                    "summary": "JULES_API_KEY fehlt für --approve",
                    "findings": []
                });
                return (2, json);
            }
        };

        let url = format!(
            "https://jules.googleapis.com/v1alpha/sessions/{}:approvePlan",
            session_id
        );
        match transport.approve_plan(&url, "X-Goog-Api-Key", &api_key, "{}") {
            Ok(_) => {
                let json = serde_json::json!({
                    "gate": "plan-lint",
                    "status": "pass",
                    "summary": format!("Plan genehmigt für Session {}", session_id),
                    "revision_counter": revision_counter,
                    "findings": []
                });
                (0, json)
            }
            Err(e) => {
                let json = serde_json::json!({
                    "gate": "plan-lint",
                    "status": "error",
                    "summary": format!("API-Approval-Fehler: {}", e),
                    "findings": []
                });
                (2, json)
            }
        }
    } else {
        let json = serde_json::json!({
            "gate": "plan-lint",
            "status": "pass",
            "summary": "Plan-Linting erfolgreich",
            "revision_counter": revision_counter,
            "findings": []
        });
        (0, json)
    }
}

pub fn run_plan_lint(args: &[String]) -> i32 {
    let mut root_dir: Option<PathBuf> = None;
    let mut json_output = false;
    let mut plan_file_opt: Option<PathBuf> = None;
    let mut card_path_opt: Option<PathBuf> = None;
    let mut session_id_opt: Option<String> = None;
    let mut approve = false;
    let mut i = 0;

    while i < args.len() {
        if args[i] == "--root" && i + 1 < args.len() {
            root_dir = Some(PathBuf::from(&args[i + 1]));
            i += 2;
        } else if args[i] == "--plan-file" && i + 1 < args.len() {
            plan_file_opt = Some(PathBuf::from(&args[i + 1]));
            i += 2;
        } else if args[i] == "--card" && i + 1 < args.len() {
            card_path_opt = Some(PathBuf::from(&args[i + 1]));
            i += 2;
        } else if args[i] == "--session" && i + 1 < args.len() {
            session_id_opt = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "--approve" {
            approve = true;
            i += 1;
        } else if args[i] == "--json" {
            json_output = true;
            i += 1;
        } else {
            i += 1;
        }
    }

    let root = root_dir.unwrap_or_else(|| plan_lint_find_root(Path::new(".")));
    let card_path = match card_path_opt {
        Some(p) => p,
        None => {
            eprintln!("Usage: plan-lint --plan-file <file> --card <card> [--approve]");
            return 2;
        }
    };

    let plan_text = if let Some(plan_file) = plan_file_opt {
        fs::read_to_string(&plan_file).unwrap_or_default()
    } else {
        "".to_string()
    };

    let transport = PlanLintCurlTransport;

    let (code, json_res) = plan_lint_process_session(
        &root,
        session_id_opt.as_deref(),
        &plan_text,
        &card_path,
        approve,
        &transport,
    );

    if json_output {
        println!("{}", json_res);
    } else {
        let summary = json_res
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or("");
        if code == 0 {
            println!("🟢 {}", summary);
        } else {
            eprintln!("🔴 {}", summary);
        }
    }

    code
}
