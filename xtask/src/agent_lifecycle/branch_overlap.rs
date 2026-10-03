//! Agent Lifecycle Submodule - Branch Overlap Checker
//!
//! Subkommando `cargo xtask check-branch-overlap`
//! Prüft, ob offene Remote-Branches unter `claim/*` Dateiüberschneidungen mit dem
//! Scope der aktiven Task-Karte haben.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "../harness/scope_guard.rs"]
mod scope_guard;

use scope_guard::{
    scope_guard_find_card_id, scope_guard_matches_glob, scope_guard_resolve_card_path,
    ScopeGuardCard,
};

/// Haupt-Einstiegspunkt für `cargo xtask check-branch-overlap`.
/// Greift Argumente aus der Umgebung ab (`std::env::args`).
pub fn run_check_branch_overlap() -> bool {
    let args: Vec<String> = std::env::args().collect();
    run_check_branch_overlap_with_args(&args)
}

/// Parametrisierbare Implementierung von `check-branch-overlap`.
pub fn run_check_branch_overlap_with_args(args: &[String]) -> bool {
    println!("=== Running xtask check-branch-overlap ===");

    let mut root_arg: Option<String> = None;
    let mut card_arg: Option<String> = None;
    let mut remote_name = String::from("origin");

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                if i + 1 < args.len() {
                    root_arg = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--card" => {
                if i + 1 < args.len() {
                    card_arg = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--remote" => {
                if i + 1 < args.len() {
                    remote_name = args[i + 1].clone();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let root = match root_arg {
        Some(ref p) => PathBuf::from(p),
        None => find_root_dir_local(),
    };

    // 1. Offene Remote-Branches unter claim/* per `git ls-remote` abfragen (Fail-closed bei Git/Netzwerkfehler)
    let ref_spec = "refs/heads/claim/*";
    let output = Command::new("git")
        .current_dir(&root)
        .args(["ls-remote", "--heads", &remote_name, ref_spec])
        .output();

    let ls_out = match output {
        Ok(out) if out.status.success() => out,
        _ => {
            eprintln!(
                "❌ Branch Overlap Check FEHLGESCHLAGEN: Remote '{}' nicht erreichbar oder 'git ls-remote' fehlgeschlagen.",
                remote_name
            );
            return false;
        }
    };

    let ls_stdout = String::from_utf8_lossy(&ls_out.stdout);
    let remote_branches = parse_ls_remote_branches(&ls_stdout);

    if remote_branches.is_empty() {
        println!(
            "✅ Branch Overlap Check bestanden: Keine offenen Remote-Branches unter 'claim/*' auf '{}' gefunden.",
            remote_name
        );
        return true;
    }

    // 2. Scope der aktiven Task-Karte ermitteln
    let active_card_scope = match determine_active_card_scope(&root, card_arg.as_deref()) {
        Some(scope) => scope,
        None => {
            println!(
                "ℹ️ Branch Overlap Check: Keine aktive Task-Karte in .jules/tasks/ oder Session/Claims gefunden."
            );
            println!("ℹ️ Ohne definierte Task-Karte entfällt die Scope-Überlappungsprüfung. Gate bestanden.");
            return true;
        }
    };

    if active_card_scope.is_empty() {
        println!("ℹ️ Scope der aktiven Task-Karte ist leer. Branch Overlap Check bestanden.");
        return true;
    }

    // Current HEAD SHA and branch name
    let head_sha = git_rev_parse(&root, "HEAD");
    let current_branch = git_rev_parse_branch(&root);

    let mut conflicts: Vec<(String, String)> = Vec::new();

    for (sha, branch_ref) in remote_branches {
        let branch_name = branch_ref
            .strip_prefix("refs/heads/")
            .unwrap_or(&branch_ref)
            .to_string();

        // Skip our own current branch / commit
        if (head_sha.is_some() && head_sha.as_deref() == Some(&sha))
            || (current_branch.is_some() && current_branch.as_deref() == Some(&branch_name))
        {
            continue;
        }

        // Merge-base zwischen HEAD und Remote-Commit ermitteln
        let merge_base = match git_merge_base(&root, &sha) {
            Some(mb) => mb,
            None => {
                eprintln!(
                    "❌ Branch Overlap Check: Kann Merge-Base für Remote-Branch '{}' ({}) nicht ermitteln.",
                    branch_name, sha
                );
                return false;
            }
        };

        // Geänderte Dateien des Remote-Branches gegenüber Merge-Base ermitteln
        let changed_files = match git_diff_name_only(&root, &merge_base, &sha) {
            Some(files) => files,
            None => {
                eprintln!(
                    "❌ Branch Overlap Check: 'git diff' für Remote-Branch '{}' fehlgeschlagen.",
                    branch_name
                );
                return false;
            }
        };

        // Prüfe Überlappung mit dem Scope der aktiven Task-Karte
        for file in changed_files {
            for glob_pattern in &active_card_scope {
                if scope_guard_matches_glob(&file, glob_pattern) {
                    conflicts.push((branch_name.clone(), file.clone()));
                    break;
                }
            }
        }
    }

    if !conflicts.is_empty() {
        eprintln!(
            "❌ Branch Overlap Check FEHLGESCHLAGEN: Blockierende Scope-Überlappung(en) mit offenen Remote-Branches gefunden:"
        );
        for (branch, file) in &conflicts {
            eprintln!(
                "  - Branch '{}' ändert Datei '{}', die im Scope der aktiven Task-Karte liegt.",
                branch, file
            );
        }
        false
    } else {
        println!(
            "✅ Branch Overlap Check bestanden: Keine Scope-Überlappungen mit anderen 'claim/*' Branches."
        );
        true
    }
}

fn find_root_dir_local() -> PathBuf {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return PathBuf::from(path);
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Ermittelt den Scope (Liste von Glob-Mustern) der aktiven Task-Karte.
fn determine_active_card_scope(root: &Path, card_arg: Option<&str>) -> Option<Vec<String>> {
    let card_path = if let Some(c) = card_arg {
        Some(scope_guard_resolve_card_path(root, c))
    } else {
        find_card_id_from_git_log(root)
            .or_else(find_card_id_from_pr_body)
            .or_else(|| find_card_id_from_session_md(root))
            .or_else(|| find_card_id_from_claims_json(root))
            .map(|id| scope_guard_resolve_card_path(root, &id))
    };

    let path = card_path?;
    let content = fs::read_to_string(&path).ok()?;
    let card: ScopeGuardCard = toml::from_str(&content).ok()?;
    Some(card.scope)
}

fn find_card_id_from_git_log(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["log", "-n", "10", "--format=%B"])
        .output()
        .ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        scope_guard_find_card_id(&text)
    } else {
        None
    }
}

fn find_card_id_from_pr_body() -> Option<String> {
    let body = std::env::var("PR_BODY").ok()?;
    scope_guard_find_card_id(&body)
}

fn find_card_id_from_session_md(root: &Path) -> Option<String> {
    let session_path = root.join(".jules/SESSION.md");
    let content = fs::read_to_string(session_path).ok()?;
    for line in content.lines() {
        if line.contains("Issue / Task ID:") || line.contains("Task-Card:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in parts {
                if part.starts_with("T-") {
                    return Some(part.to_string());
                }
            }
        }
    }
    None
}

fn find_card_id_from_claims_json(root: &Path) -> Option<String> {
    let claims_path = root.join(".jules/claims.json");
    let content = fs::read_to_string(claims_path).ok()?;
    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    let claims = val.get("claims")?.as_array()?;
    for claim in claims {
        let active = claim.get("active")?.as_bool().unwrap_or(false);
        if active {
            if let Some(issue) = claim.get("issue").and_then(|i| i.as_str()) {
                if issue.starts_with("T-") {
                    return Some(issue.to_string());
                }
            }
        }
    }
    None
}

fn parse_ls_remote_branches(ls_stdout: &str) -> Vec<(String, String)> {
    let mut branches = Vec::new();
    for line in ls_stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            branches.push((parts[0].to_string(), parts[1].to_string()));
        }
    }
    branches
}

fn git_rev_parse(root: &Path, rev: &str) -> Option<String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["rev-parse", rev])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn git_rev_parse_branch(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch != "HEAD" {
            Some(branch)
        } else {
            None
        }
    } else {
        None
    }
}

fn git_merge_base(root: &Path, commit_b: &str) -> Option<String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["merge-base", "HEAD", commit_b])
        .output()
        .ok()?;
    if output.status.success() {
        let mb = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !mb.is_empty() {
            Some(mb)
        } else {
            None
        }
    } else {
        None
    }
}

fn git_diff_name_only(root: &Path, base: &str, target: &str) -> Option<Vec<String>> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["diff", "--name-only", base, target])
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Some(files)
    } else {
        None
    }
}
