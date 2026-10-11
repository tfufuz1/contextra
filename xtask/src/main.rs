#![allow(unused_imports, dead_code)]
use std::env;
use std::process;
use std::time::Instant;
use xtask::cli::COMMAND_DISPATCH_TABLE;

fn main() {
    let raw_args: Vec<String> = env::args().collect();
    let timing_requested = env::var("XTASK_TIMINGS").map(|v| v == "1").unwrap_or(false)
        || raw_args.iter().any(|a| a == "--timings");
    let args: Vec<String> = raw_args.into_iter().filter(|a| a != "--timings").collect();
    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("sync-docs");
    let extra_args = if args.len() >= 2 { &args[2..] } else { &[] };
    let start_time = Instant::now();

    if let Some(code) = xtask::harness::dispatch_with_builtin(subcommand, extra_args) {
        if timing_requested {
            eprintln!(
                "[timings] Command '{}' took {:?}",
                subcommand,
                start_time.elapsed()
            );
        }
        process::exit(code);
    }

    for (cmd_name, handler) in COMMAND_DISPATCH_TABLE {
        if *cmd_name == subcommand {
            let code = handler(&args);
            if timing_requested {
                eprintln!(
                    "[timings] Command '{}' took {:?}",
                    subcommand,
                    start_time.elapsed()
                );
            }
            process::exit(code);
        }
    }

    eprintln!("Unknown xtask command: {}", subcommand);
    let mut available_cmds: Vec<&str> = COMMAND_DISPATCH_TABLE
        .iter()
        .map(|(name, _)| *name)
        .collect();
    available_cmds.sort_unstable();
    available_cmds.dedup();
    eprintln!("Available commands: {}", available_cmds.join(", "));
    process::exit(1);
}
