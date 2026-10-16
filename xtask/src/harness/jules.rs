//! 5-phase Jules facade orchestrator (`jules start|check|verify|submit|stop`).

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(clippy::duplicate_mod)]
#[path = "loop_guard.rs"]
mod loop_guard;

#[derive(Debug, Deserialize)]
pub struct JulesFacadeStep {
    pub cmd: Vec<String>,
    pub required: bool,
}

#[derive(Debug, Deserialize)]
pub struct JulesFacadePhaseConfig {
    pub start: Option<Vec<JulesFacadeStep>>,
    pub check: Option<Vec<JulesFacadeStep>>,
    pub verify: Option<Vec<JulesFacadeStep>>,
    pub submit: Option<Vec<JulesFacadeStep>>,
    pub stop: Option<Vec<JulesFacadeStep>>,
}

#[derive(Debug, Deserialize)]
pub struct JulesFacadeConfig {
    pub phase: JulesFacadePhaseConfig,
}

pub fn jules_facade_find_root(start: &Path) -> PathBuf {
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

pub fn jules_facade_get_session_owner(cmd_name: &str) -> &'static str {
    match cmd_name {
        "env-attest" | "ledger" | "loop-guard" | "session-report" => "Session 5",
        "protected-paths" | "gate-weakening" | "ratchet" => "Session 2",
        "scope-guard" | "diff-budget" | "claims-in-pr" | "test-integrity" | "symbol-exists" => {
            "Session 6"
        }
        "determinism-check" | "unsafe-audit" | "lock-audit" => "Session 8",
        "doc-truth" => "Session 3",
        _ => "einer anderen parallelen Session",
    }
}

pub fn jules_facade_run_step(
    step: &JulesFacadeStep,
    extra_args: &[String],
    root: &Path,
) -> (i32, String, String) {
    if step.cmd.is_empty() {
        return (2, "".to_string(), "Leeres Kommando".to_string());
    }

    let program = &step.cmd[0];
    let mut args: Vec<String> = step.cmd[1..].to_vec();
    args.extend_from_slice(extra_args);

    let output = match Command::new(program).args(&args).current_dir(root).output() {
        Ok(o) => o,
        Err(e) => return (2, "".to_string(), e.to_string()),
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let exit_code = output.status.code().unwrap_or(2);
    (exit_code, stdout, stderr)
}

pub fn jules_facade_execute_phase(
    root: &Path,
    phase_name: &str,
    extra_args: &[String],
    config_override: Option<&JulesFacadeConfig>,
) -> (i32, serde_json::Value) {
    let default_config_path = root.join(".jules/harness-phases.toml");
    let loaded_config: Option<JulesFacadeConfig> =
        if config_override.is_none() && default_config_path.exists() {
            if let Ok(content) = fs::read_to_string(&default_config_path) {
                toml::from_str(&content).ok()
            } else {
                None
            }
        } else {
            None
        };

    let config = config_override.or(loaded_config.as_ref());
    let steps_opt = config.and_then(|c| match phase_name {
        "start" => c.phase.start.as_ref(),
        "check" => c.phase.check.as_ref(),
        "verify" => c.phase.verify.as_ref(),
        "submit" => c.phase.submit.as_ref(),
        "stop" => c.phase.stop.as_ref(),
        _ => None,
    });

    let steps = match steps_opt {
        Some(s) => s,
        None => {
            let json = serde_json::json!({
                "gate": "jules-facade",
                "status": "error",
                "summary": format!("Unbekannte Phase oder fehlende Konfiguration: {}", phase_name),
                "findings": []
            });
            return (2, json);
        }
    };

    let mut overall_status = 0;
    let mut step_results = Vec::new();
    let mut reasons = Vec::new();

    for step in steps {
        let cmd_str = step.cmd.join(" ");
        let sub_cmd = step
            .cmd
            .iter()
            .filter(|a| !a.starts_with('-'))
            .next_back()
            .map(|s| s.as_str())
            .unwrap_or("");

        let (code, stdout, stderr) = jules_facade_run_step(step, extra_args, root);
        let combined = format!("{}\n{}", stdout, stderr);
        let is_unknown = combined.contains("Unknown xtask command")
            || combined.contains("Unbekannter Unterbefehl")
            || combined.contains("unrecognized subcommand")
            || (code != 0 && combined.contains("Unknown"));

        if is_unknown {
            if step.required {
                let owner = jules_facade_get_session_owner(sub_cmd);
                let msg = format!(
                    "Pflichtkommando '{}' existiert noch nicht (gehört zu {})",
                    sub_cmd, owner
                );
                reasons.push(msg.clone());
                step_results.push(serde_json::json!({
                    "command": cmd_str,
                    "required": step.required,
                    "status": "error",
                    "message": msg
                }));
                overall_status = 2;
                break;
            } else {
                let msg = format!("Optionales Kommando '{}' noch nicht verfügbar", sub_cmd);
                step_results.push(serde_json::json!({
                    "command": cmd_str,
                    "required": step.required,
                    "status": "not_applicable",
                    "message": msg
                }));
            }
        } else if code == 0 {
            step_results.push(serde_json::json!({
                "command": cmd_str,
                "required": step.required,
                "status": "pass",
                "message": "Erfolgreich"
            }));
        } else {
            let msg = format!(
                "Kommando '{}' fehlgeschlagen mit Exit-Code {}",
                cmd_str, code
            );
            if reasons.len() < 3 {
                reasons.push(msg.clone());
            }

            let (hash, count, escalated, lg_err) =
                match loop_guard::loop_guard_record_text(root, Some(sub_cmd), &combined, None) {
                    Ok((h, c)) => {
                        let esc = c >= 2;
                        if esc {
                            let reason = format!(
                                "Wiederholter Fehler beim Gate '{}': Hash {} ({})",
                                sub_cmd, h, c
                            );
                            let _ = loop_guard::loop_guard_stop(root, &reason);
                        }
                        (h, c, esc, None)
                    }
                    Err(e) => (String::new(), 0, false, Some(e)),
                };

            let mut step_json = serde_json::json!({
                "command": cmd_str,
                "required": step.required,
                "status": "fail",
                "message": msg,
                "gate": sub_cmd,
                "hash": hash,
                "count": count,
                "escalated": escalated
            });

            if let Some(err) = lg_err {
                let state_msg = format!("Loop-Guard Zustandsfehler: {}", err);
                step_json["loop_guard_error"] = serde_json::json!(state_msg.clone());
                reasons.push(state_msg);
                overall_status = 2;
            } else if step.required {
                overall_status = 1;
            }

            step_results.push(step_json);

            if escalated {
                reasons.push(format!(
                    "Loop-Guard Eskalation für Gate '{}': Hash {} (Count {})",
                    sub_cmd, hash, count
                ));
                overall_status = 1;
                break;
            }
        }
    }

    let summary = if overall_status == 0 {
        format!("Phase '{}' erfolgreich abgeschlossen", phase_name)
    } else if overall_status == 1 {
        format!(
            "Phase '{}' fehlgeschlagen: {}",
            phase_name,
            reasons.join("; ")
        )
    } else {
        format!(
            "Phase '{}' abgebrochen (Fehler/Not Available): {}",
            phase_name,
            reasons.join("; ")
        )
    };

    let status_str = if overall_status == 0 {
        "pass"
    } else if overall_status == 1 {
        "fail"
    } else {
        "error"
    };

    let json = serde_json::json!({
        "gate": "jules-facade",
        "phase": phase_name,
        "status": status_str,
        "summary": summary,
        "steps": step_results,
        "reasons": reasons
    });

    (overall_status, json)
}

pub fn run_jules(args: &[String]) -> i32 {
    let mut root_dir: Option<PathBuf> = None;
    let mut json_output = false;
    let mut phase_opt: Option<String> = None;
    let mut extra_args = Vec::new();
    let mut i = 0;

    while i < args.len() {
        if args[i] == "--root" && i + 1 < args.len() {
            root_dir = Some(PathBuf::from(&args[i + 1]));
            i += 2;
        } else if args[i] == "--json" {
            json_output = true;
            i += 1;
        } else if phase_opt.is_none() && !args[i].starts_with("--") {
            phase_opt = Some(args[i].clone());
            i += 1;
        } else {
            extra_args.push(args[i].clone());
            i += 1;
        }
    }

    let root = root_dir.unwrap_or_else(|| jules_facade_find_root(Path::new(".")));
    let phase_name = match phase_opt {
        Some(p) => p,
        None => {
            eprintln!("Usage: jules <start|check|verify|submit|stop> [args]");
            return 2;
        }
    };

    let (code, json_res) = jules_facade_execute_phase(&root, &phase_name, &extra_args, None);

    if json_output {
        println!("{}", json_res);
    } else {
        let summary = json_res
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or("");
        if code == 0 {
            println!("🟢 {}", summary);
        } else if code == 1 {
            println!("🟡 {}", summary);
        } else {
            eprintln!("🔴 {}", summary);
        }
    }

    code
}
