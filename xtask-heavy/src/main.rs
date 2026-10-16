// TODO: entfernen - Ausnahmen für Dateien außerhalb des Task-Scopes in xtask-heavy
#![allow(
    clippy::unwrap_used,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

mod bench_gate;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: xtask-heavy <subcommand> [args...]");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "bench-gate" => {
            let extra_args = &args[2..];
            let success = bench_gate::run_bench_gate(extra_args);
            if success {
                std::process::exit(0);
            } else {
                std::process::exit(1);
            }
        }
        subcommand => {
            eprintln!("Unknown subcommand for xtask-heavy: {}", subcommand);
            std::process::exit(1);
        }
    }
}
