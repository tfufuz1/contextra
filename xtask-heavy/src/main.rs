use std::env;
use std::process;

mod legacy;

fn main() {
    let args: Vec<String> = env::args().collect();
    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    let sub_args = if args.len() >= 2 { &args[2..] } else { &[] };

    match subcommand {
        "fuzz" => {
            eprintln!("❌ heavy subcommand 'fuzz' is not implemented in MP-02; owned by MP-13");
            process::exit(2);
        }
        "miri" => {
            eprintln!("❌ heavy subcommand 'miri' is not implemented in MP-02; owned by MP-13");
            process::exit(2);
        }
        "mutants" => {
            eprintln!("❌ heavy subcommand 'mutants' is not implemented in MP-02; owned by MP-13");
            process::exit(2);
        }
        "chaos" => {
            eprintln!("❌ heavy subcommand 'chaos' is not implemented in MP-02; owned by MP-13");
            process::exit(2);
        }
        "loom" => {
            eprintln!("❌ heavy subcommand 'loom' is not implemented in MP-02; owned by MP-13");
            process::exit(2);
        }
        "bench" => {
            eprintln!("❌ heavy subcommand 'bench' is not implemented in MP-02; owned by MP-13");
            process::exit(2);
        }
        "bench-gate" => {
            let success = legacy::bench_gate::run_bench_gate(sub_args);
            if !success {
                process::exit(1);
            }
        }
        "check-bandit-latency-budget" => {
            if let Err(e) = legacy::check_bandit_latency_budget::check_bandit_latency_budget() {
                eprintln!("❌ check-bandit-latency-budget failed: {}", e);
                process::exit(1);
            }
        }
        "jules-dispatch" => {
            eprintln!("❌ heavy subcommand 'jules-dispatch' is not implemented in MP-02; owned by MP-13");
            process::exit(2);
        }
        _ => {
            eprintln!("xtask-heavy: standalone heavy runner for fuzz, miri, mutants, chaos, loom, bench, bench-gate, jules-dispatch");
            process::exit(2);
        }
    }
}
