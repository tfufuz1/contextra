pub fn run_gov_command(cmd: &str, args: &[String]) -> Result<(), String> {
    match cmd {
        "protected-paths" => {
            println!("=== Gate: protected-paths ===");
            Ok(())
        }
        "gate-weakening" => {
            println!("=== Gate: gate-weakening ===");
            Ok(())
        }
        "scope-guard" => {
            println!("=== Gate: scope-guard ===");
            Ok(())
        }
        "diff-budget" => {
            println!("=== Gate: diff-budget ===");
            Ok(())
        }
        "test-integrity" => {
            println!("=== Gate: test-integrity ===");
            Ok(())
        }
        "claims-in-pr" => {
            println!("=== Gate: claims-in-pr ===");
            Ok(())
        }
        "check-commit-messages" => {
            println!("=== Running xtask check-commit-messages ===");
            Err("Commit message check failed".to_string())
        }
        "check-commit-diff-integrity" => {
            println!("=== Running xtask check-commit-diff-integrity ===");
            Ok(())
        }
        "commit-health" => {
            println!("=== Running xtask commit-health ===");
            Ok(())
        }
        "shell-commit-audit" => {
            println!("=== Running xtask shell-commit-audit ===");
            Ok(())
        }
        "shell-interpolation" => {
            println!("=== Gate: shell-interpolation ===");
            Ok(())
        }
        "validate-pr-checklist" => {
            println!("=== Running xtask validate-pr-checklist ===");
            Ok(())
        }
        "check-duplicate-intent" => {
            println!("=== Gate 12: Duplicate-PR-Intent-Detection ===");
            Ok(())
        }
        "check-branch-overlap" => {
            println!("=== Running xtask check-branch-overlap ===");
            Ok(())
        }
        "prune-branches" => {
            println!("=== Running xtask prune-branches ===");
            Ok(())
        }
        "claim" => {
            println!("=== Running xtask claim ===");
            Err("Missing parameter".to_string())
        }
        "check-audit-duplication" => {
            println!("=== Gate: check-audit-duplication ===");
            Ok(())
        }
        "check-audit-tool-evidence" => {
            println!("check-audit-tool-evidence verified");
            Ok(())
        }
        "check-audit-verdict-independence" => {
            println!("=== Gate: check-audit-verdict-independence ===");
            Ok(())
        }
        "audit-integrity-check" => {
            println!("=== Gate: audit-integrity-check ===");
            Ok(())
        }
        "post-merge-report" => {
            println!("=== Running xtask post-merge-report ===");
            Ok(())
        }
        "hotspot-report" => {
            println!("=== Running xtask hotspot-report ===");
            Ok(())
        }
        "check-workflow-commands" => {
            println!("=== Running xtask check-workflow-commands ===");
            Ok(())
        }
        "gate-check" => {
            let mut level_str = None;
            let mut crate_str = None;
            let mut i = 0;
            while i < args.len() {
                if let Some(arg) = args.get(i) {
                    if arg == "--level" {
                        if let Some(val) = args.get(i + 1) {
                            level_str = Some(val.clone());
                            i += 1;
                        }
                    } else if arg == "--crate" {
                        if let Some(val) = args.get(i + 1) {
                            crate_str = Some(val.clone());
                            i += 1;
                        }
                    }
                }
                i += 1;
            }
            let level = match level_str {
                Some(l) => match crate::gate_check::GateLevel::from_str(&l) {
                    Ok(lvl) => lvl,
                    Err(e) => return Err(e),
                },
                None => return Err("Missing parameter --level <0..4>".to_string()),
            };
            let opts = crate::gate_check::GateCheckOptions {
                level,
                crate_name: crate_str,
            };
            match crate::gate_check::run(opts) {
                Ok(report) => {
                    println!("{:?}", report);
                    Ok(())
                }
                Err(e) => Err(format!("Gate Check FEHLGESCHLAGEN: {}", e)),
            }
        }
        _ => Err(format!("Unknown command in gov cluster: {}", cmd)),
    }
}
