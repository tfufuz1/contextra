use crate::core::catalog::Catalog;
use crate::core::evidence::{save_evidence, EvidenceRecord};
use crate::core::gate::{Gate, GateContext, GateSpec, Outcome};
use crate::core::report::VerificationReport;

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

fn run_check(args: &[String]) -> i32 {
    let phase = parse_arg_val(args, "--phase").unwrap_or_else(|| "PH0".to_string());
    let json_output = args.iter().any(|a| a == "--json");

    let root = crate::find_root_dir();
    let gates_toml_path = root.join("governance/gates.toml");

    let catalog = match Catalog::load_from_path(&gates_toml_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load gate catalog: {}", e);
            return 2;
        }
    };

    let mut records = Vec::new();
    let mut total_failures = 0;

    for (_name, entry) in &catalog.gates {
        if entry.from_phase == phase {
            let spec: GateSpec = entry.into();
            let (rec, outcome) = execute_gate(&spec);
            if matches!(outcome, Outcome::Fail { .. }) {
                total_failures += 1;
            }
            records.push((rec, outcome));
        }
    }

    let report = VerificationReport::build(&records);
    if json_output {
        if let Ok(json) = report.to_json() {
            println!("{}", json);
        }
    } else {
        println!("{}", report.to_text());
    }

    if total_failures > 0 {
        1
    } else {
        0
    }
}

fn run_verify(args: &[String]) -> i32 {
    let phase = parse_arg_val(args, "--phase").unwrap_or_else(|| "PH0".to_string());
    let json_output = args.iter().any(|a| a == "--json");

    let root = crate::find_root_dir();
    let gates_toml_path = root.join("governance/gates.toml");

    let catalog = match Catalog::load_from_path(&gates_toml_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load gate catalog: {}", e);
            return 2;
        }
    };

    let evidence_dir = root.join("target/evidence");
    let mut records = Vec::new();
    let mut total_errors = 0;

    for (_name, entry) in &catalog.gates {
        if entry.from_phase == phase {
            let spec: GateSpec = entry.into();
            let (rec, outcome) = execute_gate(&spec);
            let _ = save_evidence(&evidence_dir, &rec);
            if !outcome.is_pass_or_na() {
                total_errors += 1;
            }
            records.push((rec, outcome));
        }
    }

    let report = VerificationReport::build(&records);
    if json_output {
        if let Ok(json) = report.to_json() {
            println!("{}", json);
        }
    } else {
        println!("{}", report.to_text());
    }

    if total_errors > 0 {
        2
    } else {
        0
    }
}

fn run_gate(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: cargo xtask gate <name> [-- args]");
        return 2;
    }

    let gate_name = &args[0];
    let root = crate::find_root_dir();
    let gates_toml_path = root.join("governance/gates.toml");

    let catalog = match Catalog::load_from_path(&gates_toml_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load gate catalog: {}", e);
            return 2;
        }
    };

    let entry = match catalog.gates.get(gate_name) {
        Some(g) => g,
        None => {
            eprintln!("Gate '{}' not found in catalog", gate_name);
            return 2;
        }
    };

    let spec: GateSpec = entry.into();
    let (_rec, outcome) = execute_gate(&spec);
    println!("Gate '{}' outcome: {:?}", gate_name, outcome);

    outcome.exit_code()
}

fn run_explain(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: cargo xtask explain <gate|INV-*|REQ-*>");
        return 2;
    }

    let topic = &args[0];
    let root = crate::find_root_dir();
    let gates_toml_path = root.join("governance/gates.toml");

    let catalog = match Catalog::load_from_path(&gates_toml_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load gate catalog: {}", e);
            return 2;
        }
    };

    if let Some(entry) = catalog.gates.get(topic) {
        println!("Gate: {}", entry.name);
        println!("Tier: {}", entry.tier);
        println!("From Phase: {}", entry.from_phase);
        println!("Kind: {}", entry.kind);
        println!("Enforces: {:?}", entry.enforces);
        println!("Why: {}", entry.why);
        return 0;
    }

    let matching_gates: Vec<_> = catalog
        .gates
        .values()
        .filter(|g| g.enforces.contains(&topic.to_string()))
        .collect();

    if !matching_gates.is_empty() {
        println!("Enforcement topic '{}' is guarded by:", topic);
        for g in matching_gates {
            println!("  - Gate: {} ({})", g.name, g.why);
        }
        0
    } else {
        println!("No gate found explaining topic or gate name '{}'", topic);
        2
    }
}

fn execute_gate(gate: &GateSpec) -> (EvidenceRecord, Outcome) {
    let start = std::time::Instant::now();
    let outcome = if gate.kind == "builtin" {
        if gate.name == "gate-selftests" {
            crate::gates::gov::gate_selftests::GateSelftests.run(&GateContext::default())
        } else if gate.name == "catalog-sync" {
            crate::gates::gov::catalog_sync::CatalogSync.run(&GateContext::default())
        } else {
            Outcome::Error {
                message: format!("Builtin gate '{}' not implemented yet", gate.name),
            }
        }
    } else if gate.kind == "cmd" {
        if let Some(cmd_args) = &gate.cmd {
            if cmd_args.is_empty() {
                Outcome::Error {
                    message: "Empty cmd args".to_string(),
                }
            } else {
                let mut command = std::process::Command::new(&cmd_args[0]);
                if cmd_args.len() > 1 {
                    command.args(&cmd_args[1..]);
                }
                match command.status() {
                    Ok(status) if status.success() => Outcome::Pass,
                    Ok(status) => Outcome::Fail {
                        reason: format!("Process exited with status {}", status),
                    },
                    Err(e) => Outcome::Error {
                        message: format!("Command execution failed: {}", e),
                    },
                }
            }
        } else {
            Outcome::Error {
                message: "Missing cmd array".to_string(),
            }
        }
    } else {
        Outcome::Error {
            message: format!("Unknown gate kind '{}'", gate.kind),
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    let record = EvidenceRecord {
        gate: gate.name.clone(),
        cmd: gate.cmd.clone().unwrap_or_default(),
        toolchain: gate.toolchain.clone().unwrap_or_else(|| "stable".to_string()),
        commit: "HEAD".to_string(),
        exit_code: outcome.exit_code(),
        duration_ms,
        tests_run: 0,
        tests_skipped: 0,
        log_sha256: "".to_string(),
        started_utc: "".to_string(),
    };

    (record, outcome)
}

fn parse_arg_val(args: &[String], key: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == key && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}
