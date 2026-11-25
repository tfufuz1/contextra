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

pub fn run_validate_pr_checklist(args: &[String]) -> i32 {
    let success = validate_pr_checklist::run_validate_pr_checklist();
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
