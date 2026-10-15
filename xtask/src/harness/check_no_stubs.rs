//! Harness module for `cargo xtask check-no-stubs` (G-30 / REG-18) in warn mode.

use super::stub_impls;

pub fn run_check_no_stubs(args: &[String]) -> i32 {
    let json_output = args.iter().any(|a| a == "--json");

    println!("### Running No-Stubs Check (G-30 / REG-18, Warn-Modus)...");

    let mut modified_args = vec!["--min-severity".to_string(), "niedrig".to_string()];
    if json_output {
        modified_args.push("--json".to_string());
    }

    // Run stub-impls scanner
    let _exit_code = stub_impls::run_stub_impls(&modified_args);

    println!("\n> [G-30 Warn-Modus] Stub-Scanner hat Befunde gemeldet. Im Warnmodus wird nicht blockiert.");

    // Warn mode always returns 0 to avoid blocking CI / gates in M1.
    0
}
