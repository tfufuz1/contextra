use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo xtask <command> [args...]");
        std::process::exit(1);
    }

    let cmd = &args[1];
    let cmd_args = &args[2..];

    if cmd == "check" || cmd == "verify" || cmd == "gate" || cmd == "explain" {
        let code = xtask::core::cmd::dispatch(cmd, cmd_args);
        std::process::exit(code);
    }

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
            eprintln!("Available commands: check, verify, gate, explain");
            std::process::exit(1);
        }
    }
}
