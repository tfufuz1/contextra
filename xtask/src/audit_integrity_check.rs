use chrono::{DateTime, Utc};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum GapSeverity {
    Clean,
    Suspicious(f64),
    Contradicted(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditFixGap {
    pub audit_commit: String,
    pub audit_date: String,
    pub audit_file: String,
    pub verdict: String,
    pub affected_crate: String,
    pub fix_commit: Option<String>,
    pub fix_date: Option<String>,
    pub gap_hours: Option<f64>,
    pub severity: GapSeverity,
}

fn parse_iso_datetime(ts_str: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(ts_str)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            let date_only = if ts_str.len() >= 10 {
                &ts_str[..10]
            } else {
                ts_str
            };
            format!("{}T00:00:00Z", date_only)
                .parse::<DateTime<Utc>>()
                .ok()
        })
}

fn extract_verdict(msg: &str) -> String {
    if let Some(idx) = msg.find("VERDICT:") {
        let after = &msg[idx + "VERDICT:".len()..];
        after.lines().next().unwrap_or("").trim().to_string()
    } else {
        String::new()
    }
}

fn derive_crate_name(file_path: &str) -> String {
    let normalized = file_path.replace('\\', "/");
    if normalized.starts_with("crates/") {
        let parts: Vec<&str> = normalized.split('/').collect();
        if parts.len() > 1 {
            return parts[1].to_string();
        }
    }
    let parts: Vec<&str> = normalized.split('/').collect();
    if !parts.is_empty() && !parts[0].is_empty() {
        parts[0].to_string()
    } else {
        "unknown".to_string()
    }
}

pub fn run_audit_integrity_check_impl(
    root: &Path,
    since_days: Option<u32>,
    fail_on_contradicted: bool,
) -> Result<Vec<AuditFixGap>, String> {
    let since_arg = since_days.map(|d| format!("--since={} days ago", d));

    // Get all commit hashes with dates and full commit messages
    let mut log_args = vec!["log", "--format=%H|%cI|%B%x00"];
    if let Some(ref sa) = since_arg {
        log_args.push(sa.as_str());
    }

    let output = Command::new("git")
        .current_dir(root)
        .args(&log_args)
        .output()
        .map_err(|e| format!("Failed to execute git log: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut audit_commits = Vec::new();

    for block in stdout.split('\0') {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.splitn(3, '|').collect();
        if parts.len() < 3 {
            continue;
        }

        let hash = parts[0].trim();
        let date_str = parts[1].trim();
        let body = parts[2].trim();

        let lower_body = body.to_lowercase();
        if lower_body.contains("audit") && body.contains("VERDICT") {
            let verdict = extract_verdict(body);
            audit_commits.push((hash.to_string(), date_str.to_string(), verdict));
        }
    }

    let mut results = Vec::new();

    for (audit_hash, audit_date, verdict) in audit_commits {
        let audit_dt = match parse_iso_datetime(&audit_date) {
            Some(dt) => dt,
            None => continue,
        };

        // Get modified files for audit commit
        let files_output = Command::new("git")
            .current_dir(root)
            .args([
                "show",
                "--name-only",
                "--format=",
                "--no-renames",
                &audit_hash,
            ])
            .output()
            .map_err(|e| format!("Failed to execute git show --name-only: {}", e))?;

        if !files_output.status.success() {
            continue;
        }

        let files_stdout = String::from_utf8_lossy(&files_output.stdout);
        let audit_files: Vec<String> = files_stdout
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        for file_path in audit_files {
            let affected_crate = derive_crate_name(&file_path);

            // Find follow-up commits for this file after audit_hash date
            let log_file_output = Command::new("git")
                .current_dir(root)
                .args(["log", "--format=%H|%cI", "--follow", "--", &file_path])
                .output();

            let mut earliest_fix: Option<(String, String, f64)> = None;

            if let Ok(lfo) = log_file_output {
                if lfo.status.success() {
                    let file_log_stdout = String::from_utf8_lossy(&lfo.stdout);
                    for line in file_log_stdout.lines() {
                        let parts: Vec<&str> = line.split('|').collect();
                        if parts.len() < 2 {
                            continue;
                        }
                        let commit_hash = parts[0].trim();
                        let commit_date = parts[1].trim();

                        if commit_hash == audit_hash {
                            continue;
                        }

                        if let Some(commit_dt) = parse_iso_datetime(commit_date) {
                            if commit_dt > audit_dt {
                                let gap_seconds = (commit_dt - audit_dt).num_seconds();
                                let gap_hours = gap_seconds as f64 / 3600.0;

                                if gap_hours <= 48.0 {
                                    match earliest_fix {
                                        None => {
                                            earliest_fix = Some((
                                                commit_hash.to_string(),
                                                commit_date.to_string(),
                                                gap_hours,
                                            ));
                                        }
                                        Some((_, _, existing_gap)) => {
                                            if gap_hours < existing_gap {
                                                earliest_fix = Some((
                                                    commit_hash.to_string(),
                                                    commit_date.to_string(),
                                                    gap_hours,
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let (fix_commit, fix_date, gap_hours, severity) = match earliest_fix {
                Some((c, d, g)) => {
                    let sev = if g <= 2.0 {
                        GapSeverity::Contradicted(g)
                    } else if g <= 6.0 {
                        GapSeverity::Suspicious(g)
                    } else {
                        GapSeverity::Clean
                    };
                    (Some(c), Some(d), Some(g), sev)
                }
                None => (None, None, None, GapSeverity::Clean),
            };

            results.push(AuditFixGap {
                audit_commit: audit_hash.clone(),
                audit_date: audit_date.clone(),
                audit_file: file_path,
                verdict: verdict.clone(),
                affected_crate,
                fix_commit,
                fix_date,
                gap_hours,
                severity,
            });
        }
    }

    if fail_on_contradicted
        && results
            .iter()
            .any(|r| matches!(r.severity, GapSeverity::Contradicted(_)))
    {
        return Err(
            "Audit integrity check failed: detected contradicted audit-to-fix gap (<=2h)"
                .to_string(),
        );
    }

    Ok(results)
}

pub fn run_audit_integrity_check(
    since_days: Option<u32>,
    fail_on_contradicted: bool,
) -> Result<Vec<AuditFixGap>, String> {
    let root = xtask::find_root_dir();
    run_audit_integrity_check_impl(&root, since_days, fail_on_contradicted)
}
