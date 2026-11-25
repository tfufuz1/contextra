#![allow(unused_imports, dead_code, unused_variables)]
use crate::*;
use std::path::{Path, PathBuf};
use std::process;

pub fn run_generate_diagnostics(args: &[String]) -> i32 {
    if let Err(e) = xtask::generate_diagnostics::run_generate_diagnostics() {
        eprintln!("❌ generate-diagnostics failed: {}", e);
        return 1;
    }
    0
}

pub fn run_jules_preflight(args: &[String]) -> i32 {
    let fast_only = args.iter().any(|arg| arg == "--fast");
    let success = jules_preflight::run_jules_preflight(fast_only);
    if !success {
        return 1;
    }
    0
}

pub fn run_validate_pr_checklist(args: &[String]) -> i32 {
    let success = validate_pr_checklist::run_validate_pr_checklist();
    if !success {
        return 1;
    }
    0
}

pub fn run_claim(args: &[String]) -> i32 {
    let success = claim::run_claim(&args[2..]);
    if !success {
        return 1;
    }
    0
}

pub fn run_pre_push(args: &[String]) -> i32 {
    let success = jules_submit_gate::run_pre_push();
    if !success {
        return 1;
    }
    0
}

pub fn run_jules_submit_gate(args: &[String]) -> i32 {
    let mut crate_name = None;
    let mut i = 2;
    while i < args.len() {
        if let Some(val) = args[i].strip_prefix("--crate=") {
            crate_name = Some(val);
        } else if args[i] == "--crate" && i + 1 < args.len() {
            crate_name = Some(args[i + 1].as_str());
            i += 1;
        }
        i += 1;
    }
    let success = jules_submit_gate::run_jules_submit_gate(crate_name);
    if !success {
        return 1;
    }
    0
}

pub fn run_gate_check(args: &[String]) -> i32 {
    let mut level = None;
    let mut crate_name = None;
    let mut i = 2;
    while i < args.len() {
        if let Some(val) = args[i].strip_prefix("--level=") {
            level = Some(val.to_string());
        } else if args[i] == "--level" && i + 1 < args.len() {
            level = Some(args[i + 1].clone());
            i += 1;
        } else if let Some(val) = args[i].strip_prefix("--crate=") {
            crate_name = Some(val.to_string());
        } else if args[i] == "--crate" && i + 1 < args.len() {
            crate_name = Some(args[i + 1].clone());
            i += 1;
        }
        i += 1;
    }

    let parsed_level = match level.as_deref() {
        Some(lvl_str) => match gate_check::GateLevel::from_str(lvl_str) {
            Ok(lvl) => lvl,
            Err(e) => {
                eprintln!("❌ {}", e);
                return 1;
            }
        },
        None => {
            eprintln!("❌ Parameter --level <0..4> ist erforderlich.");
            return 1;
        }
    };

    let opts = gate_check::GateCheckOptions {
        level: parsed_level,
        crate_name,
    };

    match gate_check::run(opts) {
        Ok(report) => {
            println!(
                "✅ Gate Check Level {:?} ERFOLGREICH BESTANDEN",
                report.level
            );
            if let Some(ref c) = report.crate_name {
                println!("   Betroffenes Crate: {}", c);
            }
            println!("   Ausgeführte Schritte ({}):", report.passed_steps.len());
            for step in &report.passed_steps {
                println!("     - {}", step);
            }
            if !report.todo_notes.is_empty() {
                println!("   Hinweise / TODOs:");
                for note in &report.todo_notes {
                    println!("     - {}", note);
                }
            }
        }
        Err(err) => {
            eprintln!("❌ Gate Check FEHLGESCHLAGEN: {}", err);
            return 1;
        }
    }
    0
}

pub fn run_generate_markers(args: &[String]) -> i32 {
    let target_path = args.get(2).map(PathBuf::from);
    if let Err(e) = generate_markers::run_generate_markers(target_path.as_deref()) {
        eprintln!("❌ generate-markers failed: {}", e);
        return 1;
    }
    println!("✅ generate-markers complete");
    0
}

pub fn run_context_pack(args: &[String]) -> i32 {
    let crate_filter = args.iter().find_map(|arg| arg.strip_prefix("--crate="));
    let fast = args.iter().any(|arg| arg == "--fast");
    let output_str = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--output="))
        .unwrap_or(".jules/context/CONTEXT_PACK.md");
    let output_path = PathBuf::from(output_str);
    if let Err(e) = context_pack::run_context_pack(crate_filter, fast, &output_path) {
        eprintln!("❌ context-pack failed: {}", e);
        return 1;
    }
    println!("✅ context-pack generated at {}", output_path.display());
    0
}

pub fn run_session_init(args: &[String]) -> i32 {
    let crate_name = args.iter().find_map(|arg| arg.strip_prefix("--crate="));
    let task_description = args.iter().find_map(|arg| arg.strip_prefix("--task="));
    let output_env = args.iter().any(|arg| arg == "--output-env");
    match session_init::run_session_init(crate_name, task_description, output_env) {
        Ok(res) => {
            println!("SESSION_HASH: {}", res.session_hash);
            println!("Offene BLOCKER: {}", res.open_blockers);
        }
        Err(e) => {
            eprintln!("❌ session-init failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_crate_context(args: &[String]) -> i32 {
    let crate_name = match args.get(2) {
        Some(arg) if !arg.starts_with("--") => arg.as_str(),
        _ => {
            eprintln!("Usage: cargo xtask crate-context <CRATE> [--format=json] [--output=PFAD]");
            return 1;
        }
    };
    let format_str = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--format="))
        .unwrap_or("markdown");
    let format = match format_str {
        "json" => crate_context::OutputFormat::Json,
        _ => crate_context::OutputFormat::Markdown,
    };
    let output_path = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--output="))
        .map(PathBuf::from);
    match crate_context::run_crate_context(crate_name, format, output_path.as_deref()) {
        Ok(output) => {
            if output_path.is_none() {
                println!("{}", output);
            } else {
                println!(
                    "✅ crate-context written to {}",
                    output_path.unwrap().display()
                );
            }
        }
        Err(e) => {
            eprintln!("❌ crate-context failed: {}", e);
            return 1;
        }
    }
    0
}
