mod bench_gate;
mod check_bandit_latency_budget;

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
        "check-bandit-latency-budget" => {
            match check_bandit_latency_budget::check_bandit_latency_budget() {
                Ok(_) => std::process::exit(0),
                Err(e) => {
                    eprintln!("❌ check-bandit-latency-budget failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        subcommand => {
            eprintln!("Unknown subcommand for xtask-heavy: {}", subcommand);
            std::process::exit(1);
        }
    }
}
