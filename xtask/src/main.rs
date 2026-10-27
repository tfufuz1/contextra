// xtask entry-point — Contextra quality-gate runner | WP-X.2 2026-10-26
//
// Primary commands (also callable from `just`):
//   check   — run all registered gates for the current phase
//   gate    — interact with a single gate (check / explain / list)
//   verify  — full workspace verification (pre-merge)
//
// Additional commands are dispatched via the legacy chain below.
//
// Flags:
//   --help         Print this help text and exit
//   --list-gates   List all gates from governance/gates.toml and exit

use std::env;

fn print_help() {
    println!("cargo xtask <command> [args...]");
    println!();
    println!("Primary commands:");
    println!("  check              Run all registered gates for the current phase");
    println!("  verify             Full workspace verification (pre-merge)");
    println!("  gate check         Check a specific gate (--gate <name>)");
    println!("  gate explain <n>   Explain a gate (alias: gate-explain)");
    println!("  gate-explain <n>   Show gate purpose, SPEC refs and ratchet baseline");
    println!("  gate-list [--json] List all gates from governance/gates.toml");
    println!();
    println!("Use --list-gates for a quick overview of all configured gates.");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Top-level flags that do not require a subcommand
    if args.len() >= 2 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--list-gates" => {
                let ctx = xtask::core::ctx::Context::discover();
                let catalog_path = ctx.root_dir.join("governance/gates.toml");
                if let Ok(catalog) = xtask::core::catalog::Catalog::load_from_path(&catalog_path) {
                    println!("=== Configured Gates (Active Phase: {}) ===", catalog.active_phase);
                    for (name, gate) in catalog.gates {
                        println!("  {:<25} [{}] {}", name, gate.tier, gate.why);
                    }
                }
                std::process::exit(0);
            }
            _ => {}
        }
    }

    if args.len() < 2 {
        eprintln!("Usage: cargo xtask <command> [args...]");
        eprintln!("       cargo xtask --help");
        std::process::exit(1);
    }

    let cmd = &args[1];
    let cmd_args = &args[2..];

    // ── Core dispatcher (check | gate | verify | explain) ────────────────────
    if cmd == "check" || cmd == "verify" || cmd == "gate" || cmd == "explain" {
        let code = xtask::core::cmd::dispatch(cmd, cmd_args);
        std::process::exit(code);
    }

    // ── New gate utilities ────────────────────────────────────────────────────
    if cmd == "gate-explain" {
        let code = xtask::core::cmd::run_explain(cmd_args);
        std::process::exit(code);
    }

    if cmd == "gate-check" {
        let res = xtask::legacy::gov::run_gov_command(cmd, cmd_args);
        match res {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    if cmd == "gate-list" {
        let ctx = xtask::core::ctx::Context::discover();
        let catalog_path = ctx.root_dir.join("governance/gates.toml");
        if let Ok(catalog) = xtask::core::catalog::Catalog::load_from_path(&catalog_path) {
            println!("=== Configured Gates (Active Phase: {}) ===", catalog.active_phase);
            for (name, gate) in catalog.gates {
                println!("  {:<25} [{}] {}", name, gate.tier, gate.why);
            }
            std::process::exit(0);
        } else {
            eprintln!("❌ Failed to load gate catalog");
            std::process::exit(2);
        }
    }

    // ── Session / docs commands ───────────────────────────────────────────────
    if cmd == "session" || cmd == "task" || cmd == "status" || cmd == "docs" || cmd == "ratchet" {
        let res = match cmd.as_str() {
            "session" => xtask::legacy::session::run_session_command("session", cmd_args),
            "docs" => xtask::legacy::docs::run_docs_command("docs", cmd_args),
            _ => {
                eprintln!("❌ Subcommand '{}' is owned by another MP prompt.", cmd);
                std::process::exit(2);
            }
        };
        match res {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("❌ Error executing {}: {}", cmd, e);
                std::process::exit(2);
            }
        }
    }

    // ── Legacy chain ─────────────────────────────────────────────────────────
    let res = xtask::legacy::arch::run_arch_command(cmd, cmd_args)
        .or_else(|_| xtask::legacy::code::run_code_command(cmd, cmd_args))
        .or_else(|_| xtask::legacy::gov::run_gov_command(cmd, cmd_args))
        .or_else(|_| xtask::legacy::ratchet_supply::run_ratchet_supply_command(cmd, cmd_args))
        .or_else(|_| xtask::legacy::session::run_session_command(cmd, cmd_args))
        .or_else(|_| xtask::legacy::docs::run_docs_command(cmd, cmd_args))
        .or_else(|_| xtask::legacy::tags::run_tags_command(cmd, cmd_args))
        .or_else(|_| xtask::legacy::heavy::run_heavy_command(cmd, cmd_args));

    match res {
        Ok(()) => std::process::exit(0),
        Err(_) => {
            eprintln!("Unknown xtask command: {}", cmd);
            eprintln!("Run `cargo xtask --help` for a list of available commands.");
            eprintln!("Run `cargo xtask --list-gates` for all configured gates.");
            std::process::exit(1);
        }
    }
}
