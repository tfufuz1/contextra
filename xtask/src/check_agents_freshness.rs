//! Crate AGENTS.md Freshness Gate (`check_agents_freshness.rs`)
//!
//! Subkommando `cargo xtask check-agents-freshness`
//! Prüft, ob ein PR Source-Code innerhalb eines Crates (`crates/<name>/src/**`) ändert,
//! ohne dass die zugehörige `crates/<name>/AGENTS.md` aktualisiert wurde oder ein
//! expliziter Waiver-Marker im Commit/Diff enthalten ist.

use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FreshnessViolation {
    pub crate_name: String,
    pub modified_file: String,
}

pub fn get_changed_files_from_git() -> Vec<String> {
    // Check working tree status first, fallback to merge-base / HEAD~1
    let mut files = Vec::new();
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output();

    if let Ok(out) = status_output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.len() > 3 {
                    let path_part = line[3..].trim();
                    files.push(path_part.to_string());
                }
            }
        }
    }

    if files.is_empty() {
        let diff_output = Command::new("git")
            .args(["diff", "--name-only", "HEAD~1"])
            .output();

        if let Ok(out) = diff_output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                files.extend(stdout.lines().map(|s| s.trim().to_string()));
            }
        }
    }

    files.sort();
    files.dedup();
    files
}

pub fn check_agents_freshness_for_files(changed_files: &[String], root: &Path) -> Vec<FreshnessViolation> {
    let mut violations = Vec::new();

    // Check for global waiver marker in latest commit message
    let commit_msg_output = Command::new("git")
        .args(["log", "-1", "--pretty=%B"])
        .output();

    if let Ok(out) = commit_msg_output {
        if out.status.success() {
            let msg = String::from_utf8_lossy(&out.stdout);
            if msg.contains("<!-- agents-md-waiver:") || msg.contains("agents-md-waiver") {
                println!("ℹ️ Waiver-Marker für AGENTS.md Freshness erkannt — Gate übersprungen.");
                return Vec::new();
            }
        }
    }

    let mut modified_crates = std::collections::HashSet::new();
    let mut updated_agents = std::collections::HashSet::new();

    for file in changed_files {
        if file.starts_with("crates/") {
            let parts: Vec<&str> = file.split('/').collect();
            if parts.len() >= 2 {
                let crate_dir_name = parts[1];
                if file.ends_with("AGENTS.md") {
                    updated_agents.insert(crate_dir_name.to_string());
                } else if file.contains("/src/") {
                    modified_crates.insert((crate_dir_name.to_string(), file.clone()));
                }
            }
        }
    }

    for (crate_name, mod_file) in modified_crates {
        if !updated_agents.contains(&crate_name) {
            let agents_path = root.join("crates").join(&crate_name).join("AGENTS.md");
            if agents_path.exists() {
                violations.push(FreshnessViolation {
                    crate_name,
                    modified_file: mod_file,
                });
            }
        }
    }

    violations
}

pub fn run_check_agents_freshness() -> Result<(), String> {
    println!("=== Running xtask check-agents-freshness ===");
    let root = crate::find_root_dir();
    let changed_files = get_changed_files_from_git();

    let violations = check_agents_freshness_for_files(&changed_files, &root);

    if !violations.is_empty() {
        for v in &violations {
            eprintln!(
                "❌ [check-agents-freshness]: Crate '{}' hat Source-Änderungen in '{}', aber 'crates/{}/AGENTS.md' wurde nicht aktualisiert.",
                v.crate_name, v.modified_file, v.crate_name
            );
            eprintln!("   💡 Aktualisiere 'crates/{}/AGENTS.md' oder füge '<!-- agents-md-waiver: <Grund> -->' zur Commit-Message hinzu.", v.crate_name);
        }
        return Err(format!(
            "check-agents-freshness failed with {} stale crate AGENTS.md file(s)",
            violations.len()
        ));
    }

    println!("✅ Alle geänderten Crates besitzen ein aktualisiertes AGENTS.md oder Waiver.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agents_freshness_clean() {
        let changed = vec![
            "crates/contextra-core/src/lib.rs".to_string(),
            "crates/contextra-core/AGENTS.md".to_string(),
        ];
        let root = crate::find_root_dir();
        let violations = check_agents_freshness_for_files(&changed, &root);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_agents_freshness_stale() {
        let changed = vec![
            "crates/contextra-core/src/lib.rs".to_string(),
        ];
        let root = crate::find_root_dir();
        let violations = check_agents_freshness_for_files(&changed, &root);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].crate_name, "contextra-core");
    }
}
