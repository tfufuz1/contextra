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

#[derive(Debug, Deserialize, Default, Clone)]
pub struct JulesFacadeRiskGatesConfig {
    #[serde(default)]
    pub sec: Vec<String>,
    #[serde(default)]
    pub crash: Vec<String>,
    #[serde(default)]
    pub simd: Vec<String>,
    #[serde(default)]
    pub cargo: Vec<String>,
    #[serde(default)]
    pub protected: Vec<String>,
    #[serde(default)]
    pub diff_budget: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct JulesFacadeConfig {
    pub phase: JulesFacadePhaseConfig,
}

#[derive(Debug, Deserialize)]
struct JulesFacadeFullConfig {
    pub phase: JulesFacadePhaseConfig,
    #[serde(default)]
    pub risk_gates: Option<JulesFacadeRiskGatesConfig>,
}

#[derive(Debug, Deserialize)]
struct TaskCardMinimal {
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "crate")]
    pub crate_field: Option<String>,
    pub crate_name: Option<String>,
    pub risk: Option<String>,
    pub scope: Option<Vec<String>>,
    pub acceptance: Option<Vec<String>>,
}

impl TaskCardMinimal {
    fn get_crate(&self) -> String {
        if let Some(ref c) = self.crate_field {
            c.clone()
        } else if let Some(ref c) = self.crate_name {
            c.clone()
        } else {
            String::new()
        }
    }
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
    card_path_opt: Option<&Path>,
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

    let (program, mut raw_step_args) = if let Some(ref exe) = current_exe_opt {
        (exe.clone(), step.cmd[2..].to_vec())
    } else {
        (step.cmd[0].clone(), step.cmd[1..].to_vec())
    };

    let sub_cmd = step
        .cmd
        .iter()
        .filter(|a| !a.starts_with('-'))
        .next_back()
        .map(|s| s.as_str())
        .unwrap_or("");

    // Task 3: symbol-exists im check-Schritt -> übergebe tatsächlich geänderte Dateien
    if sub_cmd == "symbol-exists" || raw_step_args.iter().any(|a| a == "symbol-exists") {
        if let Some(changed) =
            blast_radius::blast_radius_get_changed_files_worktree(root, "origin/main", "HEAD")
        {
            if !changed.is_empty() {
                for f in changed {
                    if !raw_step_args.contains(&f) {
                        raw_step_args.push(f);
                    }
                }
            }
        }
    }

    // Task 2: verify scopen -> ersetze unbegrenztes `cargo test` durch -p <crate>
    if step.cmd == vec!["cargo", "test"] {
        let mut card_opt: Option<TaskCardMinimal> = None;
        if let Some(cp) = card_path_opt {
            if let Ok(content) = fs::read_to_string(cp) {
                card_opt = toml::from_str(&content).ok();
            }
        }

        let changed_files =
            blast_radius::blast_radius_get_changed_files_worktree(root, "origin/main", "HEAD")
                .unwrap_or_default();
        let touched_cargo = changed_files.iter().any(|f| {
            f.to_lowercase().ends_with("cargo.toml") || f.to_lowercase().ends_with("cargo.lock")
        });

        let card_crate = card_opt.as_ref().map(|c| c.get_crate()).unwrap_or_default();
        let card_risk = card_opt
            .as_ref()
            .and_then(|c| c.risk.as_deref())
            .unwrap_or("none");

        if !touched_cargo && card_risk != "concurrency" && !card_crate.is_empty() {
            raw_step_args = vec!["test".to_string(), "-p".to_string(), card_crate];
        }
    }

    let mut filtered_extra_args = Vec::new();
    let mut arg_idx = 0;
    while arg_idx < extra_args.len() {
        let arg = &extra_args[arg_idx];
        if arg == "--card" || arg == "--reason" || arg == "--root" {
            arg_idx += 2;
        } else if arg == "--allow-overlap"
            || arg == "--full"
            || arg == "--timings"
            || arg == "--json"
        {
            arg_idx += 1;
        } else {
            filtered_extra_args.push(arg.clone());
            arg_idx += 1;
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

    let mut cmd_obj = Command::new(&program);
    cmd_obj.args(&final_args).current_dir(root);

    // Task 8: Umgebungsvariable CONTEXTRA_CARD=<pfad> weitergeben
    if let Some(cp) = card_path_opt {
        cmd_obj.env("CONTEXTRA_CARD", cp.to_string_lossy().to_string());
    }

    let output_res = cmd_obj.output();

    let output = match output_res {
        Ok(o) => o,
        Err(e) => {
            if current_exe_opt.is_some() {
                // Fallback to cargo xtask
                let fallback_prog = "cargo";
                let mut fallback_args = vec!["xtask".to_string()];
                fallback_args.extend(final_args);
                let mut fb_cmd = Command::new(fallback_prog);
                fb_cmd.args(&fallback_args).current_dir(root);
                if let Some(cp) = card_path_opt {
                    fb_cmd.env("CONTEXTRA_CARD", cp.to_string_lossy().to_string());
                }
                match fb_cmd.output() {
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

fn compute_required_risk_gates(
    root: &Path,
    risk_cfg: Option<&JulesFacadeRiskGatesConfig>,
) -> Vec<String> {
    let changed =
        blast_radius::blast_radius_get_changed_files_worktree(root, "origin/main", "HEAD")
            .unwrap_or_default();
    let mut gates = Vec::new();

    let mut max_tier = "none";
    let mut touched_unsafe = false;
    let mut touched_cargo = false;
    let mut touched_protected = false;

    for file in &changed {
        let f_lower = file.to_lowercase();
        if f_lower.contains("crypto") || f_lower.contains("privacy") {
            max_tier = "sec";
        } else if f_lower.contains("wal")
            || f_lower.contains("store")
            || f_lower.contains("checkpoint")
        {
            if max_tier != "sec" {
                max_tier = "crash";
            }
        } else if (f_lower.contains("simd") || f_lower.contains("vector"))
            && max_tier != "sec"
            && max_tier != "crash"
        {
            max_tier = "simd";
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

    let default_sec = vec!["mutants-diff".to_string(), "fuzz-smoke".to_string()];
    let default_crash = vec!["wal-replay-verify".to_string()];
    let default_simd = vec!["unsafe-audit".to_string()];
    let default_cargo = vec!["dependency-gate".to_string()];
    let default_protected = vec!["protected-paths".to_string()];
    let default_diff = vec!["diff-budget".to_string()];

    match max_tier {
        "sec" => {
            let list = risk_cfg
                .map(|rc| &rc.sec)
                .filter(|v| !v.is_empty())
                .unwrap_or(&default_sec);
            gates.extend(list.clone());
        }
        "crash" => {
            let list = risk_cfg
                .map(|rc| &rc.crash)
                .filter(|v| !v.is_empty())
                .unwrap_or(&default_crash);
            gates.extend(list.clone());
        }
        "simd" => {
            let list = risk_cfg
                .map(|rc| &rc.simd)
                .filter(|v| !v.is_empty())
                .unwrap_or(&default_simd);
            gates.extend(list.clone());
        }
        _ => {}
    }

    if touched_unsafe {
        let list = risk_cfg
            .map(|rc| &rc.simd)
            .filter(|v| !v.is_empty())
            .unwrap_or(&default_simd);
        gates.extend(list.clone());
    }
    if touched_cargo {
        let list = risk_cfg
            .map(|rc| &rc.cargo)
            .filter(|v| !v.is_empty())
            .unwrap_or(&default_cargo);
        gates.extend(list.clone());
    }
    if touched_protected {
        let list = risk_cfg
            .map(|rc| &rc.protected)
            .filter(|v| !v.is_empty())
            .unwrap_or(&default_protected);
        gates.extend(list.clone());
    }
    if changed.len() > 10 {
        let list = risk_cfg
            .map(|rc| &rc.diff_budget)
            .filter(|v| !v.is_empty())
            .unwrap_or(&default_diff);
        gates.extend(list.clone());
    }

    gates.sort();
    gates.dedup();
    gates
}

pub fn jules_facade_execute_phase(
    root: &Path,
    phase_name: &str,
    extra_args: &[String],
    config_override: Option<&JulesFacadeConfig>,
) -> (i32, serde_json::Value) {
    let default_config_path = root.join(".jules/harness-phases.toml");
    let (loaded_config, parsed_risk_gates) =
        if config_override.is_none() && default_config_path.exists() {
            if let Ok(content) = fs::read_to_string(&default_config_path) {
                let cfg: Option<JulesFacadeConfig> = toml::from_str(&content).ok();
                let full_cfg: Option<JulesFacadeFullConfig> = toml::from_str(&content).ok();
                let r_gates = full_cfg.and_then(|fc| fc.risk_gates);
                (cfg, r_gates)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

    let config = config_override.or(loaded_config.as_ref());

    // Task 5 & 8: Parse --card parameter or load active card from .jules/context/active_card
    let mut card_path_buf: Option<PathBuf> = None;
    let mut allow_overlap = false;

    let mut idx = 0;
    while idx < extra_args.len() {
        if extra_args[idx] == "--card" && idx + 1 < extra_args.len() {
            card_path_buf = Some(PathBuf::from(&extra_args[idx + 1]));
            idx += 2;
        } else if extra_args[idx] == "--allow-overlap" {
            allow_overlap = true;
            idx += 1;
        } else {
            idx += 1;
        }
    }

    let active_card_path = root.join(".jules/context/active_card");
    if card_path_buf.is_none() && active_card_path.exists() {
        if let Ok(p_str) = fs::read_to_string(&active_card_path) {
            let trimmed = p_str.trim();
            if !trimmed.is_empty() {
                card_path_buf = Some(PathBuf::from(trimmed));
            }
        }
    }

    if phase_name == "start" && card_path_buf.is_none() && config_override.is_none() {
        let msg = "Fehler: Parameter --card <PFAD> fehlt. Beispiel: cargo xtask jules start --card .jules/tasks/W1-01.toml".to_string();
        eprintln!("{}", msg);
        let json = serde_json::json!({
            "gate": "jules-facade",
            "status": "error",
            "summary": msg,
            "findings": []
        });
        return (2, json);
    }

    if phase_name == "start" {
        if let Some(ref cp) = card_path_buf {
            let ctx_dir = root.join(".jules/context");
            let _ = fs::create_dir_all(&ctx_dir);
            let _ = fs::write(active_card_path, cp.to_string_lossy().to_string());
        }
    }

    // Task 7: Parallel-Schutz via check-branch-overlap bei jules start
    if phase_name == "start" {
        if let Some(ref cp) = card_path_buf {
            let actual_cp = if cp.is_relative() {
                root.join(cp)
            } else {
                cp.clone()
            };
            if actual_cp.exists() {
                let overlap_step = JulesFacadeStep {
                    cmd: vec![
                        "cargo".to_string(),
                        "xtask".to_string(),
                        "check-branch-overlap".to_string(),
                        "--card".to_string(),
                        cp.to_string_lossy().to_string(),
                    ],
                    required: false,
                    parallel_group: None,
                    when_changed: None,
                    serial: false,
                };
                let (code, stdout, stderr) = jules_facade_run_step(
                    &overlap_step,
                    extra_args,
                    root,
                    card_path_buf.as_deref(),
                );
                let combined = format!("{}\n{}", stdout, stderr);
                let is_unknown = combined.contains("Unknown xtask command")
                    || combined.contains("Unbekannter Unterbefehl")
                    || combined.contains("unrecognized subcommand");
                if code != 0 && !is_unknown && !allow_overlap {
                    let msg = "Konflikt, Karte überspringen".to_string();
                    eprintln!("{}", msg);
                    let json = serde_json::json!({
                        "gate": "jules-facade",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    });
                    return (1, json);
                }
            }
        }
    }

    // Task 5: jules start ruft session-init und context-pack als interne Teilschritte auf
    if phase_name == "start" {
        if let Some(ref cp) = card_path_buf {
            let actual_card_path = if cp.is_relative() {
                root.join(cp)
            } else {
                cp.clone()
            };
            if let Ok(card_str) = fs::read_to_string(&actual_card_path) {
                if let Ok(parsed_card) = toml::from_str::<TaskCardMinimal>(&card_str) {
                    let krate = parsed_card.get_crate();
                    let title = parsed_card.title.unwrap_or_default();

                    let init_step = JulesFacadeStep {
                        cmd: vec![
                            "cargo".to_string(),
                            "xtask".to_string(),
                            "session-init".to_string(),
                            "--crate".to_string(),
                            krate.clone(),
                            "--task".to_string(),
                            title,
                        ],
                        required: false,
                        parallel_group: None,
                        when_changed: None,
                        serial: false,
                    };
                    let _ = jules_facade_run_step(
                        &init_step,
                        extra_args,
                        root,
                        card_path_buf.as_deref(),
                    );

                    let pack_step = JulesFacadeStep {
                        cmd: vec![
                            "cargo".to_string(),
                            "xtask".to_string(),
                            "context-pack".to_string(),
                            "--crate".to_string(),
                            krate,
                        ],
                        required: false,
                        parallel_group: None,
                        when_changed: None,
                        serial: false,
                    };
                    let _ = jules_facade_run_step(
                        &pack_step,
                        extra_args,
                        root,
                        card_path_buf.as_deref(),
                    );
                }
            }
        }
    }

    let mut steps = match config.and_then(|c| match phase_name {
        "start" => c.phase.start.clone(),
        "check" => c.phase.check.clone(),
        "verify" => c.phase.verify.clone(),
        "submit" => c.phase.submit.clone(),
        "stop" => c.phase.stop.clone(),
        _ => None,
    }) {
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

    // Task 1: Risiko-Gates belegen (in verify und submit)
    if phase_name == "verify" || phase_name == "submit" {
        let req_risk_gates = compute_required_risk_gates(root, parsed_risk_gates.as_ref());
        for gate_cmd in req_risk_gates {
            let step_cmd = vec!["cargo".to_string(), "xtask".to_string(), gate_cmd];
            if !steps.iter().any(|s| s.cmd == step_cmd) {
                steps.push(JulesFacadeStep {
                    cmd: step_cmd,
                    required: true,
                    parallel_group: None,
                    when_changed: None,
                    serial: false,
                });
            }
        }
    }

    let show_timings = extra_args.iter().any(|a| a == "--timings");
    let phase_start = std::time::Instant::now();

    let changed_files_opt =
        blast_radius::blast_radius_get_changed_files_worktree(root, "origin/main", "HEAD");

    let mut overall_status = 0;
    let mut step_results = Vec::new();
    let mut reasons = Vec::new();

    // Task 6: Persistent last error hash file
    let last_hash_path = root.join(".jules/context/last_error_hash");
    let mut last_error_hash: Option<String> = if last_hash_path.exists() {
        fs::read_to_string(&last_hash_path)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    // Partition steps into sequential items and parallel groups
    let mut step_batches: Vec<Vec<&JulesFacadeStep>> = Vec::new();
    let mut current_batch: Vec<&JulesFacadeStep> = Vec::new();
    let mut current_group: Option<String> = None;

    for step in &steps {
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
                    let matched = files.iter().any(|f| {
                        globs
                            .iter()
                            .any(|g| scope_guard::scope_guard_matches_glob(f, g))
                    });
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
                let res = jules_facade_run_step(step, extra_args, root, card_path_buf.as_deref());
                let dur = start_inst.elapsed();
                vec![(step, Some((res, dur)))]
            }
        } else {
            std::thread::scope(|s| {
                let mut handles = Vec::new();
                for &step in &batch {
                    let should_skip = if let Some(ref globs) = step.when_changed {
                        if let Some(ref files) = changed_files_opt {
                            let matched = files.iter().any(|f| {
                                globs
                                    .iter()
                                    .any(|g| scope_guard::scope_guard_matches_glob(f, g))
                            });
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
                        let cp_ref = card_path_buf.as_deref();
                        let handle = s.spawn(move || {
                            let start_inst = std::time::Instant::now();
                            let res = jules_facade_run_step(step, extra_args, root, cp_ref);
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

            // Task 8: context-card --check Exit 3 -> Hinweis "jules start erneut ausführen"
            if sub_cmd == "context-card" && code == 3 {
                let hint_msg = "context-card veraltet: jules start erneut ausführen".to_string();
                reasons.push(hint_msg.clone());
                step_results.push(serde_json::json!({
                    "command": cmd_str,
                    "required": step.required,
                    "status": "fail",
                    "message": hint_msg
                }));
                if step.required && overall_status == 0 {
                    overall_status = 1;
                }
                continue;
            }

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

                let (hash, count, escalated, lg_err) = match loop_guard::loop_guard_record_text(
                    root,
                    Some(sub_cmd),
                    &combined,
                    None,
                ) {
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

                // Task 6: Thrash-Erkennung — gleicher Fehlerhash zweimal in Folge -> stop --reason thrash vorschlagen, Exit 3
                if let Some(ref prev_hash) = last_error_hash {
                    if prev_hash == &hash && !hash.is_empty() {
                        let thrash_msg = format!("Thrash-Erkennung: Gleicher Fehlerhash {} zweimal in Folge! Empfehlung: cargo xtask jules stop --reason thrash", hash);
                        eprintln!("{}", thrash_msg);
                        reasons.push(thrash_msg);
                        overall_status = 3;
                        batch_stop = true;
                    }
                }
                if !hash.is_empty() {
                    let ctx_dir = root.join(".jules/context");
                    let _ = fs::create_dir_all(&ctx_dir);
                    let _ = fs::write(&last_hash_path, &hash);
                    last_error_hash = Some(hash.clone());
                }

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

    // Task 4: stop -> loop-guard stop ausführen, Grund in .jules/context/ / .jules/local/ protokollieren, verified = true setzen
    if phase_name == "stop" {
        let stop_reason = extra_args
            .iter()
            .position(|a| a == "--reason")
            .and_then(|idx| extra_args.get(idx + 1))
            .cloned()
            .unwrap_or_else(|| "Manuelle Unterbrechung".to_string());

        let lg_res = loop_guard::loop_guard_stop(root, &stop_reason);

        let ctx_dir = root.join(".jules/context");
        let _ = fs::create_dir_all(&ctx_dir);
        let _ = fs::write(ctx_dir.join("LAST_STOP.reason"), &stop_reason);

        let stop_status_code = if lg_res.is_ok() { 0 } else { 1 };

        let stop_json = serde_json::json!({
            "gate": "jules-facade",
            "phase": "stop",
            "status": if stop_status_code == 0 { "pass" } else { "fail" },
            "verified": true,
            "summary": format!("Session gestoppt: {}", stop_reason),
            "findings": []
        });
        return (stop_status_code, stop_json);
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

    if phase_name == "start" && overall_status == 0 {
        println!("Kontextpaket erzeugt unter .jules/context/CONTEXT_PACK.md");
    }

    (overall_status, json)
}

// Task 9: Claims-Invariante
// Das Task-Claim-System (claim) führt Buch über .jules/claims.json.
// Hinweishinweis: .jules/claims.json liegt lokal pro VM und koordiniert keine Parallelsitzungen
// über verschiedene physische Knoten hinweg; die Kollisionsprüfung (Aufgabe 7) stützt sich
// auf Remote-Branches (check-branch-overlap).

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_setup_repo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();
        fs::create_dir_all(root.join(".jules/tasks")).unwrap();
        fs::create_dir_all(root.join(".cargo")).unwrap();
        fs::create_dir_all(root.join("xtask/src")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"xtask\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("capabilities.toml"),
            "[crates.contextra-core]\nstatus = \"active\"\n",
        )
        .unwrap();

        fs::write(
            root.join(".jules/harness-phases.toml"),
            r#"[[phase.start]]
cmd = ["cargo", "--version"]
required = true

[[phase.stop]]
cmd = ["cargo", "xtask", "loop-guard", "stop"]
required = false
"#,
        )
        .unwrap();

        fs::write(
            root.join(".jules/tasks/W1-01.toml"),
            r#"id = "T-2026-9101"
title = "Test Task"
crate = "contextra-core"
tier = 1
risk = "none"
goal = "Test"
non_goals = []
scope = ["crates/contextra-core/src/**"]
forbidden = []
invariants = ["Zero-Panic"]
acceptance = ["cargo test"]
evidence_required = []

[budget]
files = 1
lines = 10
iterations = 1
"#,
        )
        .unwrap();

        (temp_dir, root)
    }

    #[test]
    fn test_missing_card_parameter_returns_error() {
        let (_temp, root) = test_setup_repo();
        let (code, json) = jules_facade_execute_phase(&root, "start", &[], None);
        assert_eq!(code, 2);
        assert_eq!(json["status"], "error");
        assert!(json["summary"]
            .as_str()
            .unwrap()
            .contains("Parameter --card <PFAD> fehlt"));
    }

    #[test]
    fn test_stop_phase_verified_true() {
        let (_temp, root) = test_setup_repo();
        let extra_args = vec!["--reason".to_string(), "unit-test-stop".to_string()];
        let (code, json) = jules_facade_execute_phase(&root, "stop", &extra_args, None);
        assert_eq!(code, 0);
        assert_eq!(json["status"], "pass");
        assert_eq!(json["verified"], true);

        let reason = fs::read_to_string(root.join(".jules/context/LAST_STOP.reason")).unwrap();
        assert_eq!(reason, "unit-test-stop");
    }
}
