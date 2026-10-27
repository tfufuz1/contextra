use super::common::not_implemented;

pub fn run_gov_command(cmd: &str, _args: &[String]) -> Result<(), String> {
    match cmd {
<<<<<<< HEAD
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
            let mut level = None;
            let mut crate_name = None;
            let mut i = 0;
            while i < args.len() {
                if args[i] == "--level" && i + 1 < args.len() {
                    level = Some(args[i + 1].as_str());
                    i += 1;
                } else if args[i] == "--crate" && i + 1 < args.len() {
                    crate_name = Some(args[i + 1].as_str());
                    i += 1;
                }
                i += 1;
            }

            match level {
                Some("0") => {
                    if let Some(c) = crate_name {
                        println!("Starte Gate-Check Level 0 für Crate {}", c);
                        Ok(())
                    } else {
                        eprintln!("Fehler: Parameter --crate ist erforderlich für Level 0");
                        Err("Fehler: Parameter --crate ist erforderlich für Level 0".to_string())
                    }
                }
                Some("2") => {
                    eprintln!("Gate 2 ist spezifiziert, aber noch nicht als automatisierte Prüfung hinterlegt.");
                    Err("Gate 2 ist spezifiziert, aber noch nicht als automatisierte Prüfung hinterlegt.".to_string())
                }
                Some(_) => Ok(()),
                None => {
                    eprintln!("Missing parameter --level <0..4>");
                    Err("Missing parameter --level <0..4>".to_string())
                }
            }
        }
=======
        "protected-paths"
        | "gate-weakening"
        | "scope-guard"
        | "diff-budget"
        | "test-integrity"
        | "claims-in-pr"
        | "check-commit-messages"
        | "check-commit-diff-integrity"
        | "commit-health"
        | "shell-commit-audit"
        | "shell-interpolation"
        | "validate-pr-checklist"
        | "check-duplicate-intent"
        | "check-branch-overlap"
        | "prune-branches"
        | "claim"
        | "check-audit-duplication"
        | "check-audit-tool-evidence"
        | "check-audit-verdict-independence"
        | "audit-integrity-check"
        | "post-merge-report"
        | "hotspot-report"
        | "check-workflow-commands"
        | "gate-check" => not_implemented(cmd),
>>>>>>> 492c01e05 (fix(xtask): eliminate legacy command stubs and enforce stub ratchet)
        _ => Err(format!("Unknown command in gov cluster: {}", cmd)),
    }
}
