use crate::core::catalog::{is_phase_due, Catalog};
use crate::core::ctx::Context;
use crate::core::evidence::{save_evidence_with_log, EvidenceRecord};
use crate::core::gate::Outcome;
use crate::core::ratchet::Ratchet;
use crate::core::report::VerificationReport;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

pub fn dispatch(cmd: &str, args: &[String]) -> i32 {
    match cmd {
        "check" => run_check(args),
        "verify" => run_verify(args),
        "gate" => run_gate(args),
        "explain" => run_explain(args),
        _ => {
            eprintln!("Unknown core command: {}", cmd);
            2
        }
    }
}

pub fn load_not_run_baseline(root_dir: &std::path::Path) -> HashSet<String> {
    let ratchet_path = root_dir.join("governance/ratchet.toml");
    let mut baseline = HashSet::new();
    if let Ok(ratchet) = Ratchet::load_from_path(&ratchet_path) {
        if let Some(val) = ratchet.file.entries.get("not_run_baseline") {
            if let Some(array) = val.as_array() {
                for item in array {
                    if let Some(s) = item.as_str() {
                        baseline.insert(s.to_string());
                    }
                }
            } else if let Some(table) = val.as_table() {
                if let Some(gates_val) = table.get("gates") {
                    if let Some(array) = gates_val.as_array() {
                        for item in array {
                            if let Some(s) = item.as_str() {
                                baseline.insert(s.to_string());
                            }
                        }
                    }
                } else {
                    for key in table.keys() {
                        baseline.insert(key.clone());
                    }
                }
            }
        }
    }
    baseline
}

pub fn run_check(args: &[String]) -> i32 {
    let ctx = Context::discover();
    let catalog_path = ctx.root_dir.join("governance/gates.toml");
    let catalog = match Catalog::load_from_path(&catalog_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to load gate catalog: {}", e);
            return 2;
        }
    };

    let mut is_json = false;
    let mut requested_tier: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--json" {
            is_json = true;
        } else if let Some(t) = arg.strip_prefix("--tier=") {
            requested_tier = Some(t.to_string());
        } else if arg == "--tier" && i + 1 < args.len() {
            requested_tier = Some(args[i + 1].clone());
            i += 1;
        }
        i += 1;
    }

    let registered_gates = crate::gates::registered_gates();
    let mut registered_map: HashMap<String, _> = registered_gates
        .into_iter()
        .map(|g| (g.spec().name.clone(), g))
        .collect();

    let mut records = Vec::new();
    let evidence_dir = ctx.root_dir.join("target/evidence");

    for name in &catalog.gate_order {
        let gate_entry = match catalog.gates.get(name) {
            Some(g) => g,
            None => continue,
        };

        // Phase check
        let is_due = match is_phase_due(&gate_entry.from_phase, &catalog.active_phase) {
            Ok(due) => due,
            Err(e) => {
                eprintln!("❌ Error checking phase for gate '{}': {}", name, e);
                return 2;
            }
        };

        let start = Instant::now();
        let mut raw_log = String::new();
        let mut tests_run = 0;
        let mut tests_skipped = 0;

        let outcome = if !is_due {
            Outcome::NotDue {
                reason: format!(
                    "from_phase {} > active_phase {}",
                    gate_entry.from_phase, catalog.active_phase
                ),
            }
        } else if let Some(ref selected_tier) = requested_tier {
            if gate_entry.tier != *selected_tier {
                Outcome::OtherTier {
                    tier: gate_entry.tier.clone(),
                }
            } else {
                run_gate_entry(
                    name,
                    gate_entry,
                    &ctx,
                    &mut registered_map,
                    &mut raw_log,
                    &mut tests_run,
                    &mut tests_skipped,
                )
            }
        } else {
            // Without --tier specified: run all due gates
            run_gate_entry(
                name,
                gate_entry,
                &ctx,
                &mut registered_map,
                &mut raw_log,
                &mut tests_run,
                &mut tests_skipped,
            )
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        let log_sha256 = blake3::hash(raw_log.as_bytes()).to_hex().to_string();

        let record = EvidenceRecord {
            gate: name.clone(),
            cmd: gate_entry.cmd.clone().unwrap_or_default(),
            toolchain: gate_entry
                .toolchain
                .clone()
                .unwrap_or_else(|| "stable".to_string()),
            commit: ctx.commit_hash.clone(),
            exit_code: outcome.exit_code(),
            duration_ms,
            tests_run,
            tests_skipped,
            log_sha256,
            started_utc: chrono::Utc::now().to_rfc3339(),
        };

        if let Err(err_msg) = save_evidence_with_log(&evidence_dir, &record, &raw_log) {
            eprintln!("❌ Failed to save evidence for '{}': {}", name, err_msg);
            return 2;
        }

        records.push((record, outcome));
    }

    let report = VerificationReport::build(&records);

    if is_json {
        if let Ok(j) = report.to_json() {
            println!("{}", j);
        }
    } else {
        println!("{}", report.to_text());
    }

    // Check D-4: compare NotRun gates with governance/ratchet.toml [not_run_baseline]
    let not_run_baseline = load_not_run_baseline(&ctx.root_dir);
    let mut unbaselined_not_run = Vec::new();
    let mut has_fail = false;
    let mut has_error = false;

    for (_rec, outcome) in &records {
        match outcome {
            Outcome::Fail { .. } => has_fail = true,
            Outcome::Error { .. } => has_error = true,
            Outcome::NotRun { .. } => {
                if !not_run_baseline.contains(&_rec.gate) {
                    unbaselined_not_run.push(_rec.gate.clone());
                }
            }
            _ => {}
        }
    }

    if !unbaselined_not_run.is_empty() {
        eprintln!(
            "❌ Gates NOT RUN and not listed in governance/ratchet.toml [not_run_baseline]: {:?}",
            unbaselined_not_run
        );
        return 1;
    }

    if has_fail {
        1
    } else if has_error {
        2
    } else {
        0
    }
}

pub fn run_gate_entry(
    name: &str,
    gate_entry: &crate::core::catalog::GateEntry,
    ctx: &Context,
    registered_map: &mut HashMap<String, Box<dyn crate::core::gate::Gate>>,
    raw_log: &mut String,
    tests_run: &mut u64,
    tests_skipped: &mut u64,
) -> Outcome {
    if gate_entry.kind == "builtin" {
        if let Some(impl_gate) = registered_map.remove(name) {
            let res = impl_gate.run(ctx);
            *raw_log = format!("Builtin gate output:\n{:?}", res);
            res
        } else {
            *raw_log = format!("Builtin gate '{}' not registered", name);
            Outcome::NotRun {
                reason: format!(
                    "Builtin gate '{}' not implemented by owner {}",
                    name, gate_entry.owner
                ),
            }
        }
    } else if gate_entry.kind == "cmd" {
        if let Some(ref cmd_args) = gate_entry.cmd {
            if cmd_args.is_empty() {
                *raw_log = format!("Command gate '{}' missing cmd definition", name);
                Outcome::NotRun {
                    reason: format!("Command gate '{}' missing cmd definition", name),
                }
            } else {
                let exec_res = crate::core::exec::run_cmd(
                    &cmd_args[0],
                    &cmd_args[1..].iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    &ctx.root_dir,
                );
                *raw_log = exec_res.output;
                *tests_run = exec_res.tests_run;
                *tests_skipped = exec_res.tests_skipped;

                if exec_res.test_parse_error {
                    Outcome::Error {
                        message: format!(
                            "Failed to parse test output counters for gate '{}'",
                            name
                        ),
                    }
                } else if exec_res.exit_code == 0 {
                    Outcome::Pass
                } else if exec_res.exit_code == 1 {
                    Outcome::Fail {
                        reason: format!("Command failed with exit code 1"),
                    }
                } else {
                    Outcome::Error {
                        message: format!("Command error with exit code {}", exec_res.exit_code),
                    }
                }
            }
        } else {
            *raw_log = format!("Command gate '{}' missing cmd field", name);
            Outcome::NotRun {
                reason: format!("Command gate '{}' missing cmd field", name),
            }
        }
    } else if gate_entry.kind == "xtask-heavy" || gate_entry.kind == "xtask" {
        *raw_log = format!("Gate '{}' is kind {}", name, gate_entry.kind);
        Outcome::NotRun {
            reason: format!("Gate '{}' is owned by xtask/xtask-heavy", name),
        }
    } else {
        *raw_log = format!("Unknown gate kind '{}' for {}", gate_entry.kind, name);
        Outcome::NotRun {
            reason: format!("Unknown gate kind '{}' for {}", gate_entry.kind, name),
        }
    }
}

pub fn run_verify(args: &[String]) -> i32 {
    let ctx = Context::discover();
    let catalog_path = ctx.root_dir.join("governance/gates.toml");
    let catalog = match Catalog::load_from_path(&catalog_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to load gate catalog: {}", e);
            return 2;
        }
    };

    let check_code = run_check(args);

    println!("\n=== VERIFY: EVIDENCE COMPLETENESS & MAPPINGS ===");
    let evidence_dir = ctx.root_dir.join("target/evidence");

    let mut missing_evidence = Vec::new();
    let mut invalid_commit = Vec::new();
    let mut has_not_run = false;

    for name in &catalog.gate_order {
        let gate_entry = match catalog.gates.get(name) {
            Some(g) => g,
            None => continue,
        };

        let is_due = is_phase_due(&gate_entry.from_phase, &catalog.active_phase).unwrap_or(false);
        if !is_due {
            continue;
        }

        let ev_file = evidence_dir.join(format!("{}.json", name));
        let log_file = evidence_dir.join(format!("{}.log", name));

        if !ev_file.exists() || !log_file.exists() {
            missing_evidence.push(name.clone());
            continue;
        }

        if let Ok(content) = std::fs::read(&ev_file) {
            if let Ok(rec) = serde_json::from_slice::<EvidenceRecord>(&content) {
                if rec.commit != ctx.commit_hash {
                    invalid_commit.push(name.clone());
                }
                if rec.exit_code == 2 {
                    has_not_run = true;
                }
            } else {
                missing_evidence.push(name.clone());
            }
        } else {
            missing_evidence.push(name.clone());
        }
    }

    if !missing_evidence.is_empty() {
        eprintln!(
            "❌ Missing evidence files for due gates: {:?}",
            missing_evidence
        );
    }
    if !invalid_commit.is_empty() {
        eprintln!("❌ Evidence commit != HEAD for gates: {:?}", invalid_commit);
    }

    println!("\n=== GUARANTEE -> TEST -> GATE MAPPING ===");
    for name in &catalog.gate_order {
        if let Some(gate_entry) = catalog.gates.get(name) {
            if !gate_entry.enforces.is_empty() {
                println!("  Gate {:<25} enforces: {:?}", name, gate_entry.enforces);
            }
        }
    }

    if check_code != 0 || !missing_evidence.is_empty() || !invalid_commit.is_empty() || has_not_run
    {
        2
    } else {
        0
    }
}

pub fn run_gate(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Verwendung: cargo xtask gate <gate-name> [-- args]");
        return 2;
    }

    let gate_name = &args[0];
    let ctx = Context::discover();

    let registered_gates = crate::gates::registered_gates();
    for gate in registered_gates {
        if gate.spec().name == *gate_name {
            let outcome = gate.run(&ctx);
            println!("Gate '{}' outcome: {:?}", gate_name, outcome);
            return outcome.exit_code();
        }
    }

    eprintln!(
        "❌ Gate '{}' not found in registered builtin gates",
        gate_name
    );
    2
}

pub fn run_explain(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Verwendung: cargo xtask explain <gate|INV-*|REQ-*>");
        return 2;
    }

    let topic = &args[0];

    let catalog_path = Context::discover().root_dir.join("governance/gates.toml");
    if let Ok(catalog) = Catalog::load_from_path(&catalog_path) {
        if let Some(g) = catalog.gates.get(topic) {
            println!("=== GATE EXPLANATION: {} ===", g.name);
            println!("Tier: {}", g.tier);
            println!("Phase: {}", g.from_phase);
            println!("Owner: {}", g.owner);
            println!("Enforces: {:?}", g.enforces);
            println!("Why: {}", g.why);
            return 0;
        }
    }

    eprintln!("Topic/Gate '{}' explanation not found.", topic);
    2
}
