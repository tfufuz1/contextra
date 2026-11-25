//! Preflight Claim- und Branch-Overlap-Guard (claim-guard).

use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run_claim_guard(args: &[String]) -> i32 {
    let mut root_dir = find_root_dir();
    let mut base_rev = "origin/main".to_string();
    let mut head_rev = "HEAD".to_string();

    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                if idx + 1 < args.len() {
                    root_dir = PathBuf::from(&args[idx + 1]);
                    idx += 1;
                }
            }
            "--base" => {
                if idx + 1 < args.len() {
                    base_rev = args[idx + 1].clone();
                    idx += 1;
                }
            }
            "--head" => {
                if idx + 1 < args.len() {
                    head_rev = args[idx + 1].clone();
                    idx += 1;
                }
            }
            _ => {}
        }
        idx += 1;
    }

    let changed_files = get_changed_files(&root_dir, &base_rev, &head_rev);
    let mut touched_crates = std::collections::HashSet::new();

    for file in changed_files {
        if file.starts_with("crates/") {
            let parts: Vec<&str> = file.split('/').collect();
            if parts.len() >= 2 {
                touched_crates.insert(parts[1].to_string());
            }
        }
    }

    if touched_crates.is_empty() {
        println!("✅ [CLAIM-GUARD]: Keine Crates berührt. Überspringe Claim-Prüfung.");
        return 0;
    }

    println!("ℹ️ [CLAIM-GUARD]: Berührte Crates: {:?}", touched_crates);

    let mut collision_found = false;

    // 1. Aktive Claims prüfen
    for krate in &touched_crates {
        let claim_args = vec!["--crate".to_string(), krate.clone(), "--dry-run".to_string()];
        let ok = crate::claim::run_claim(&claim_args);
        if !ok {
            eprintln!("❌ Claim-Kollision für Crate '{}'", krate);
            collision_found = true;
        }
    }

    // 2. Remote-Branch-Überschneidung prüfen
    // (Prüft den globalen Branch-Overlap-Status)
    let overlap_ok = crate::agent_lifecycle::branch_overlap::run_check_branch_overlap_with_args(&[]);
    if !overlap_ok {
        eprintln!("❌ Branch-Überschneidung gefunden.");
        collision_found = true;
    }

    if collision_found {
        2
    } else {
        0
    }
}

fn find_root_dir() -> PathBuf {
    if let Ok(out) = Command::new("git").args(["rev-parse", "--show-toplevel"]).output() {
        if out.status.success() {
            return PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string());
        }
    }
    PathBuf::from(".")
}

fn get_changed_files(root: &Path, base: &str, head: &str) -> Vec<String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["diff", "--name-only", base, head])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    Vec::new()
}
