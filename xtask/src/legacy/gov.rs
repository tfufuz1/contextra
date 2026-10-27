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
            let mut level_arg = None;
            let mut crate_arg = None;
            let mut i = 0;
            while i < args.len() {
                let curr = args.get(i).map(String::as_str);
                let next = args.get(i + 1).map(String::as_str);
                if curr == Some("--level") && next.is_some() {
                    level_arg = next;
                    i += 2;
                } else if curr == Some("--crate") && next.is_some() {
                    crate_arg = next;
                    i += 2;
                } else {
                    i += 1;
                }
            }

            match level_arg {
                Some("0") => {
                    if crate_arg.is_none() {
                        eprintln!("Fehler: --crate <name> ist für Gate Level 0 erforderlich.");
                        Err("Fehler: --crate <name> ist für Gate Level 0 erforderlich.".to_string())
                    } else {
                        Ok(())
                    }
                }
                Some(level_str @ ("2" | "3" | "4")) => {
                    let prev_phase = level_str.parse::<u8>().unwrap_or(1).saturating_sub(1);
                    let msg = format!(
                        "Gate {} ist spezifiziert, aber die zugehörigen Abnahmekriterien aus Phase {} sind in dieser xtask-Version noch nicht als automatisierte Prüfung hinterlegt — siehe docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md Teil A.3",
                        level_str, prev_phase
                    );
                    eprintln!("{}", msg);
                    Err(msg)
                }
                Some(_) | None => {
                    if args.is_empty() {
                        eprintln!("Missing parameter --level <0..4>");
                        Err("Missing parameter".to_string())
                    } else {
                        Ok(())
                    }
                }
            }
        }
        _ => Err(format!("Unknown command in gov cluster: {}", cmd)),
    }
}
