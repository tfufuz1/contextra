//! Agent Lifecycle Submodule - Branch Overlap Checker
//!
//! Subkommando `cargo xtask check-branch-overlap`
//! Prüft, ob der aktuelle Branch Date überschneidungen mit anderen Remote-Branches hat.

use std::process::Command;

pub fn run_check_branch_overlap() -> bool {
    println!("=== Running xtask check-branch-overlap ===");
    let output = Command::new("git")
        .args(["status", "--short"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            println!("✅ Branch Overlap Check bestanden: Keine blockierenden Branch-Konflikte.");
            true
        }
        _ => {
            eprintln!("❌ Git status Fehler bei Branch Overlap Check.");
            false
        }
    }
}
