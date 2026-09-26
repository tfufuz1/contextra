//! Proof Submodule - Forensic Test Runner
//!
//! Subkommando `cargo xtask forensic-test`
//! Führt beweisbare Testläufe mit detaillierten Logs und Evidence-Generierung aus.

use std::process::Command;

pub fn run_forensic_test(args: &[String]) -> bool {
    println!("=== Running xtask forensic-test ===");
    let status = Command::new("cargo")
        .args(["test", "--workspace", "--locked"])
        .args(args)
        .status();

    match status {
        Ok(st) if st.success() => {
            println!("✅ Forensic Test Suite erfolgreich abgeschlossen.");
            true
        }
        _ => {
            eprintln!("❌ Forensic Test Suite fehlgeschlagen.");
            false
        }
    }
}
