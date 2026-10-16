//! 5-phase Jules facade orchestrator (`jules start|check|verify|submit|stop`).

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(clippy::duplicate_mod)]
#[path = "loop_guard.rs"]
mod loop_guard;

#[allow(clippy::duplicate_mod)]
#[path = "blast_radius.rs"]
mod blast_radius;

#[allow(clippy::duplicate_mod)]
#[path = "scope_guard.rs"]
mod scope_guard;

#[derive(Debug, Deserialize, Clone)]
pub struct JulesFacadeStep {
    pub cmd: Vec<String>,
    pub required: bool,
    #[serde(default)]
    pub parallel_group: Option<String>,
    #[serde(default)]
    pub when_changed: Option<Vec<String>>,
    #[serde(default)]
    pub serial: bool,
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

    let is_cargo_xtask = step.cmd.len() >= 2 && step.cmd[0] == "cargo" && step.cmd[1] == "xtask";

    let current_exe_opt = if is_cargo_xtask {
        std::env::current_exe().ok().and_then(|exe| {
            let exe_str = exe.to_string_lossy();
            if exe.exists() && !exe_str.contains("/deps/") && !exe_str.contains("\\deps\\") {
                Some(exe_str.to_string())
            } else {
                None
            }
        })
    } else {
        None
    };

    let (program, raw_step_args) = if let Some(ref exe) = current_exe_opt {
        (exe.clone(), step.cmd[2..].to_vec())
    } else {
        (step.cmd[0].clone(), step.cmd[1..].to_vec())
    };

    let mut filtered_extra_args = Vec::new();
    for arg in extra_args {
        if arg != "--full" && arg != "--timings" {
            filtered_extra_args.push(arg.clone());
        }
    }

    let has_pkgs_placeholder = raw_step_args.iter().any(|a| a.contains("{pkgs}"));
    let mut final_args = Vec::new();

    if has_pkgs_placeholder {
        let is_full = extra_args.iter().any(|a| a == "--full")
            || std::env::var("JULES_FULL").as_deref() == Ok("1");

        let (affected_crates, root_changed) =
            blast_radius::blast_radius_get_affected_crates(root, Some("origin/main"), Some("HEAD"));

        let pkgs_expansion = if is_full || root_changed || affected_crates.is_empty() {
            vec!["--workspace".to_string()]
        } else {
            affected_crates
                .iter()
                .flat_map(|c| vec!["-p".to_string(), c.clone()])
                .collect()
        };

        for arg in &raw_step_args {
            if arg == "{pkgs}" {
                final_args.extend(pkgs_expansion.clone());
            } else if arg.contains("{pkgs}") {
                if pkgs_expansion.len() == 1 && pkgs_expansion[0] == "--workspace" {
                    final_args.push(arg.replace("{pkgs}", "--workspace"));
                } else {
                    final_args.extend(pkgs_expansion.clone());
                }
            } else {
                final_args.push(arg.clone());
            }
        }
    } else {
        final_args = raw_step_args;
    }

    final_args.extend(filtered_extra_args);

    let output_res = Command::new(&program).args(&final_args).current_dir(root).output();

    let output = match output_res {
        Ok(o) => o,
        Err(e) => {
            if current_exe_opt.is_some() {
                // Fallback to cargo xtask
                let fallback_prog = "cargo";
                let mut fallback_args = vec!["xtask".to_string()];
                fallback_args.extend(final_args);
                match Command::new(fallback_prog)
                    .args(&fallback_args)
                    .current_dir(root)
                    .output()
                {
                    Ok(o) => o,
                    Err(e2) => return (2, "".to_string(), e2.to_string()),
                }
            } else {
                return (2, "".to_string(), e.to_string());
            }
        }
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

    let show_timings = extra_args.iter().any(|a| a == "--timings");
    let phase_start = std::time::Instant::now();

    let changed_files_opt = blast_radius::blast_radius_get_changed_files_worktree(root, "origin/main", "HEAD");

    let mut overall_status = 0;
    let mut step_results = Vec::new();
    let mut reasons = Vec::new();

    // Partition steps into sequential items and parallel groups
    let mut step_batches: Vec<Vec<&JulesFacadeStep>> = Vec::new();
    let mut current_batch: Vec<&JulesFacadeStep> = Vec::new();
    let mut current_group: Option<String> = None;

    for step in steps {
        if step.serial || step.parallel_group.is_none() {
            if !current_batch.is_empty() {
                step_batches.push(current_batch);
                current_batch = Vec::new();
                current_group = None;
            }
            step_batches.push(vec![step]);
        } else {
            let group_name = step.parallel_group.as_ref().unwrap();
            if let Some(ref cg) = current_group {
                if cg == group_name {
                    current_batch.push(step);
                } else {
                    step_batches.push(current_batch);
                    current_batch = vec![step];
                    current_group = Some(group_name.clone());
                }
            } else {
                current_batch = vec![step];
                current_group = Some(group_name.clone());
            }
        }
    }
    if !current_batch.is_empty() {
        step_batches.push(current_batch);
    }

    'batch_loop: for batch in step_batches {
        let batch_outputs = if batch.len() == 1 {
            let step = batch[0];
            let should_skip = if let Some(ref globs) = step.when_changed {
                if let Some(ref files) = changed_files_opt {
                    let matched = files.iter().any(|f| globs.iter().any(|g| scope_guard::scope_guard_matches_glob(f, g)));
                    !matched
                } else {
                    false
                }
            } else {
                false
            };

            if should_skip {
                vec![(step, None)]
            } else {
                let start_inst = std::time::Instant::now();
                let res = jules_facade_run_step(step, extra_args, root);
                let dur = start_inst.elapsed();
                vec![(step, Some((res, dur)))]
            }
        } else {
            std::thread::scope(|s| {
                let mut handles = Vec::new();
                for &step in &batch {
                    let should_skip = if let Some(ref globs) = step.when_changed {
                        if let Some(ref files) = changed_files_opt {
                            let matched = files.iter().any(|f| globs.iter().any(|g| scope_guard::scope_guard_matches_glob(f, g)));
                            !matched
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if should_skip {
                        handles.push((step, None));
                    } else {
                        let handle = s.spawn(move || {
                            let start_inst = std::time::Instant::now();
                            let res = jules_facade_run_step(step, extra_args, root);
                            let dur = start_inst.elapsed();
                            (res, dur)
                        });
                        handles.push((step, Some(handle)));
                    }
                }

                handles
                    .into_iter()
                    .map(|(step, h_opt)| {
                        let res_opt = h_opt.map(|h| {
                            h.join().unwrap_or((
                                (2, "".to_string(), "Thread panic".to_string()),
                                std::time::Duration::from_millis(0),
                            ))
                        });
                        (step, res_opt)
                    })
                    .collect::<Vec<_>>()
            })
        };

        let mut batch_stop = false;

        for (step, exec_res) in batch_outputs {
            let cmd_str = step.cmd.join(" ");
            let sub_cmd = step
                .cmd
                .iter()
                .filter(|a| !a.starts_with('-'))
                .next_back()
                .map(|s| s.as_str())
                .unwrap_or("");

            if exec_res.is_none() {
                let msg = "SKIPPED(kein passender Pfad im Diff)";
                let mut json_val = serde_json::json!({
                    "command": cmd_str,
                    "required": step.required,
                    "status": "skipped",
                    "message": msg
                });
                if show_timings {
                    json_val["duration_ms"] = serde_json::json!(0);
                    json_val["duration_str"] = serde_json::json!("0ms");
                }
                step_results.push(json_val);
                continue;
            }

            let ((code, stdout, stderr), duration) = exec_res.unwrap();
            let duration_ms = duration.as_millis() as u64;
            let duration_str = format!("{:.2?}", duration);

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
                    let mut json_val = serde_json::json!({
                        "command": cmd_str,
                        "required": step.required,
                        "status": "error",
                        "message": msg
                    });
                    if show_timings {
                        json_val["duration_ms"] = serde_json::json!(duration_ms);
                        json_val["duration_str"] = serde_json::json!(duration_str);
                    }
                    step_results.push(json_val);
                    overall_status = 2;
                    batch_stop = true;
                } else {
                    let msg = format!("Optionales Kommando '{}' noch nicht verfügbar", sub_cmd);
                    let mut json_val = serde_json::json!({
                        "command": cmd_str,
                        "required": step.required,
                        "status": "not_applicable",
                        "message": msg
                    });
                    if show_timings {
                        json_val["duration_ms"] = serde_json::json!(duration_ms);
                        json_val["duration_str"] = serde_json::json!(duration_str);
                    }
                    step_results.push(json_val);
                }
            } else if code == 0 {
                let mut json_val = serde_json::json!({
                    "command": cmd_str,
                    "required": step.required,
                    "status": "pass",
                    "message": "Erfolgreich"
                });
                if show_timings {
                    json_val["duration_ms"] = serde_json::json!(duration_ms);
                    json_val["duration_str"] = serde_json::json!(duration_str);
                }
                step_results.push(json_val);
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
                if show_timings {
                    step_json["duration_ms"] = serde_json::json!(duration_ms);
                    step_json["duration_str"] = serde_json::json!(duration_str);
                }

                if let Some(err) = lg_err {
                    let state_msg = format!("Loop-Guard Zustandsfehler: {}", err);
                    step_json["loop_guard_error"] = serde_json::json!(state_msg.clone());
                    reasons.push(state_msg);
                    overall_status = 2;
                } else if step.required && overall_status == 0 {
                    overall_status = 1;
                }

                step_results.push(step_json);

                if escalated {
                    reasons.push(format!(
                        "Loop-Guard Eskalation für Gate '{}': Hash {} (Count {})",
                        sub_cmd, hash, count
                    ));
                    overall_status = 1;
                    batch_stop = true;
                }
            }
        }

        if batch_stop {
            break 'batch_loop;
        }
    }

    let phase_duration = phase_start.elapsed();
    let phase_duration_ms = phase_duration.as_millis() as u64;
    let phase_duration_str = format!("{:.2?}", phase_duration);

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

    let mut json = serde_json::json!({
        "gate": "jules-facade",
        "phase": phase_name,
        "status": status_str,
        "summary": summary,
        "steps": step_results,
        "reasons": reasons
    });

    if show_timings {
        json["duration_ms"] = serde_json::json!(phase_duration_ms);
        json["duration_str"] = serde_json::json!(phase_duration_str);
    }

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
