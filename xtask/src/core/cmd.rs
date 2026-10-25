use crate::core::catalog::Catalog;
use crate::core::ctx::Context;
use crate::core::evidence::{save_evidence, EvidenceRecord};
use crate::core::gate::Outcome;
use crate::core::report::VerificationReport;
use std::collections::HashMap;
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
    let mut _requested_phase = catalog.active_phase.clone();

    for arg in args {
        if arg == "--json" {
            is_json = true;
        } else if let Some(p) = arg.strip_prefix("--phase=") {
            _requested_phase = p.to_string();
        }
    }

    let registered_gates = crate::gates::registered_gates();
    let mut registered_map: HashMap<String, _> = registered_gates
        .into_iter()
        .map(|g| (g.spec().name.clone(), g))
        .collect();

    let mut records = Vec::new();

    for (name, gate_entry) in &catalog.gates {
        if gate_entry.tier != "pr" {
            continue;
        }

        let _spec = crate::core::gate::GateSpec::from(gate_entry);
        let start = Instant::now();

        let outcome = if gate_entry.kind == "builtin" {
            if let Some(impl_gate) = registered_map.remove(name) {
                impl_gate.run(&ctx)
            } else {
                Outcome::Error {
                    message: format!("Builtin gate '{}' not implemented by owner {}", name, gate_entry.owner),
                }
            }
        } else if gate_entry.kind == "cmd" {
            if let Some(ref cmd_args) = gate_entry.cmd {
                if cmd_args.is_empty() {
                    Outcome::Error {
                        message: format!("Command gate '{}' missing cmd definition", name),
                    }
                } else {
                    let exec_res = crate::core::exec::run_cmd(
                        &cmd_args[0],
                        &cmd_args[1..].iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                        &ctx.root_dir,
                    );
                    if exec_res.exit_code == 0 {
                        Outcome::Pass
                    } else if exec_res.exit_code == 1 {
                        Outcome::Fail {
                            reason: exec_res.output,
                        }
                    } else {
                        Outcome::Error {
                            message: exec_res.output,
                        }
                    }
                }
            } else {
                Outcome::Error {
                    message: format!("Command gate '{}' missing cmd field", name),
                }
            }
        } else if gate_entry.kind == "xtask-heavy" {
            Outcome::Error {
                message: format!("Gate '{}' is owned by xtask-heavy", name),
            }
        } else {
            Outcome::Error {
                message: format!("Unknown gate kind '{}' for {}", gate_entry.kind, name),
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        let record = EvidenceRecord {
            gate: name.clone(),
            cmd: gate_entry.cmd.clone().unwrap_or_default(),
            toolchain: gate_entry.toolchain.clone().unwrap_or_else(|| "stable".to_string()),
            commit: ctx.commit_hash.clone(),
            exit_code: outcome.exit_code(),
            duration_ms,
            tests_run: 0,
            tests_skipped: 0,
            log_sha256: blake3::hash(format!("{:?}", outcome).as_bytes()).to_hex().to_string(),
            started_utc: chrono::Utc::now().to_rfc3339(),
        };

        let _ = save_evidence(&ctx.root_dir.join("target/evidence"), &record);
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

    if report.overall_passed {
        0
    } else if report.failed_gates > 0 {
        1
    } else {
        2
    }
}

pub fn run_verify(args: &[String]) -> i32 {
    let ctx = Context::discover();
    let catalog_path = ctx.root_dir.join("governance/gates.toml");
    let _catalog = match Catalog::load_from_path(&catalog_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to load gate catalog: {}", e);
            return 2;
        }
    };

    let mut _is_json = false;
    let mut _evidence_dir = ctx.root_dir.join("target/evidence");

    for arg in args {
        if arg == "--json" {
            _is_json = true;
        } else if let Some(d) = arg.strip_prefix("--evidence-dir=") {
            _evidence_dir = ctx.root_dir.join(d);
        }
    }

    run_check(args)
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

    eprintln!("❌ Gate '{}' not found in registered builtin gates", gate_name);
    2
}

pub fn run_explain(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Verwendung: cargo xtask explain <gate|INV-*|REQ-*>");
        return 2;
    }

    let topic = &args[0];
    let ctx = Context::discover();

    let catalog_path = ctx.root_dir.join("governance/gates.toml");
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
