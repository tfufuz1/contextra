//! Agent Lifecycle Submodule - Prune Remote Branches
//!
//! Subkommando `cargo xtask prune-branches`
//! Bereinigt alte/gemergte Remote-Branches in Git.

use std::process::Command;

pub fn run_prune_branches() -> bool {
    println!("=== Running xtask prune-branches ===");
    let status = Command::new("git")
        .args(["remote", "prune", "origin"])
        .status();

    match status {
        Ok(st) if st.success() => {
            println!("✅ Remote origin erfolgreich gepruned.");
            true
        }
        _ => {
            eprintln!("❌ Remote prune origin fehlgeschlagen.");
            false
        }
    }
}
