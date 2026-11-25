// Contextra — VETO Review Deadline CI Gate
//
// Parst VETOES.md und überwacht Review-Fristen für bedingt akzeptierte Features ("conditionally_accepted").

use chrono::{DateTime, NaiveDate, Utc};
use std::path::{Path, PathBuf};

pub const VETO_DEADLINE_WARNING_THRESHOLD_DAYS: i64 = 14;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VetoDeadlineEntry {
    pub veto_id: String,
    pub feature_id: String,
    pub status: String,
    pub conditional_review_due: Option<String>,
    pub adr_ref: Option<String>,
    pub affected_paths: Vec<String>,
    pub keywords: Vec<String>,
}

impl VetoDeadlineEntry {
    pub fn is_file_affected(&self, file_path: &str) -> bool {
        let clean_path = file_path.trim_start_matches("./");
        for path in &self.affected_paths {
            let clean_affected = path.trim_start_matches("./").trim_start_matches('/');
            if clean_path.starts_with(clean_affected)
                || clean_path.contains(clean_affected)
                || clean_affected.contains(clean_path)
            {
                return true;
            }
        }
        for kw in &self.keywords {
            let kw_lower = kw.to_lowercase();
            if clean_path.to_lowercase().contains(&kw_lower) {
                return true;
            }
        }
        false
    }
}

pub fn parse_veto_deadlines(content: &str) -> Result<Vec<VetoDeadlineEntry>, String> {
    let mut entries = Vec::new();
    let blocks = content.split("\n## ");

    for block in blocks {
        let trimmed_block = block.trim();
        if !trimmed_block.starts_with("VETO-") && !trimmed_block.starts_with("## VETO-") {
            continue;
        }

        let mut veto_id = String::new();
        let mut feature_id = String::new();
        let mut status = String::new();
        let mut conditional_review_due = None;
        let mut adr_ref = None;
        let mut affected_paths = Vec::new();
        let mut keywords = Vec::new();

        for line in trimmed_block.lines() {
            let line_trimmed = line.trim();
            if line_trimmed.starts_with("## VETO-") {
                if let Some(id) = line_trimmed
                    .trim_start_matches("## ")
                    .split_whitespace()
                    .next()
                {
                    veto_id = id.to_string();
                }
            } else if line_trimmed.starts_with("VETO-") && veto_id.is_empty() {
                if let Some(id) = line_trimmed.split_whitespace().next() {
                    veto_id = id.to_string();
                }
            } else if line_trimmed.starts_with("affected_paths:") {
                let ap_str = line_trimmed.trim_start_matches("affected_paths:").trim();
                if ap_str.starts_with('[') && ap_str.ends_with(']') {
                    let inner = &ap_str[1..ap_str.len() - 1];
                    for item in inner.split(',') {
                        let cleaned = item.trim().trim_matches('"').trim_matches('\'').to_string();
                        if !cleaned.is_empty() {
                            affected_paths.push(cleaned);
                        }
                    }
                }
            } else if line_trimmed.starts_with("keywords:") {
                let kw_str = line_trimmed.trim_start_matches("keywords:").trim();
                if kw_str.starts_with('[') && kw_str.ends_with(']') {
                    let inner = &kw_str[1..kw_str.len() - 1];
                    for item in inner.split(',') {
                        let cleaned = item.trim().trim_matches('"').trim_matches('\'').to_string();
                        if !cleaned.is_empty() {
                            keywords.push(cleaned);
                        }
                    }
                }
            } else if let Some((key, val)) = line_trimmed.split_once(':') {
                let key = key.trim();
                let val = val.trim();
                match key {
                    "feature_id" => feature_id = val.to_string(),
                    "status" => status = val.to_string(),
                    "conditional_review_due" => {
                        if !val.is_empty() {
                            conditional_review_due = Some(val.to_string());
                        }
                    }
                    "adr_ref" => {
                        if !val.is_empty() {
                            adr_ref = Some(val.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        if !veto_id.is_empty() {
            entries.push(VetoDeadlineEntry {
                veto_id,
                feature_id,
                status,
                conditional_review_due,
                adr_ref,
                affected_paths,
                keywords,
            });
        }
    }

    Ok(entries)
}

pub fn check_veto_deadlines_from_content_scoped(
    content: &str,
    now: DateTime<Utc>,
    changed_files: Option<&[String]>,
) -> Result<Vec<VetoDeadlineEntry>, String> {
    let entries = parse_veto_deadlines(content)?;
    let today = now.date_naive();
    let is_json = std::env::args().any(|a| a == "--json");

    let mut overdue_errors = Vec::new();
    let mut warning_entries = Vec::new();

    for entry in entries {
        if !entry.status.starts_with("conditionally_accepted") {
            continue;
        }

        if let Some(due_str) = &entry.conditional_review_due {
            let due_date = NaiveDate::parse_from_str(due_str, "%Y-%m-%d").map_err(|e| {
                format!(
                    "Ungültiges Datumsformat in VETO-Eintrag {}: '{}' ({e})",
                    entry.veto_id, due_str
                )
            })?;

            let days_remaining = (due_date - today).num_days();

            if days_remaining < 0 {
                let has_assignment = !entry.affected_paths.is_empty() || !entry.keywords.is_empty();

                let is_affected = match changed_files {
                    Some(files) => {
                        if !has_assignment {
                            true
                        } else {
                            files.iter().any(|file| entry.is_file_affected(file))
                        }
                    }
                    None => true,
                };

                if is_affected {
                    if !has_assignment {
                        overdue_errors.push(format!(
                            "{} (Zuordnung fehlt: affected_paths/keywords fehlen in VETOES.md)",
                            entry.veto_id
                        ));
                    } else {
                        overdue_errors.push(entry.veto_id.clone());
                    }
                } else {
                    let msg = format!(
                        "⚠️ WARNUNG: VETO {} Review-Frist {} ist abgelaufen (adr_ref: {:?}), aber der PR berührt dieses Feature nicht. Hinweis: Bitte Review-Ticket anlegen.",
                        entry.veto_id, due_str, entry.adr_ref
                    );
                    eprintln!("{msg}");
                    warning_entries.push(entry);
                }
            } else if days_remaining <= VETO_DEADLINE_WARNING_THRESHOLD_DAYS {
                let msg = format!(
                    "⚠️ WARNUNG: VETO {} Review-Frist {} laeuft in {} Tag(en) ab (adr_ref: {:?}). Hinweis: Bitte Review-Ticket anlegen.",
                    entry.veto_id, due_str, days_remaining, entry.adr_ref
                );
                eprintln!("{msg}");
                warning_entries.push(entry);
            }
        }
    }

    if is_json {
        let json_out = serde_json::json!({
            "gate": "check-veto-deadlines",
            "overdue_errors": overdue_errors,
            "warnings": warning_entries.iter().map(|w| serde_json::json!({
                "veto_id": w.veto_id,
                "feature_id": w.feature_id,
                "due": w.conditional_review_due,
                "adr_ref": w.adr_ref
            })).collect::<Vec<_>>()
        });
        eprintln!("{}", json_out);
    }

    if !overdue_errors.is_empty() {
        return Err(format!(
            "❌ VETO-REVIEW-FRIST ÜBERSCHRITTEN: Folgende VETO-Einträge sind überfällig: {}. Bitte Frist per ADR verlängern oder Status in VETOES.md anpassen.",
            overdue_errors.join(", ")
        ));
    }

    Ok(warning_entries)
}

pub fn check_veto_deadlines_from_content(
    content: &str,
    now: DateTime<Utc>,
) -> Result<Vec<VetoDeadlineEntry>, String> {
    check_veto_deadlines_from_content_scoped(content, now, None)
}

fn get_git_changed_files(root: &Path) -> Vec<String> {
    let mut base = String::new();
    if let Ok(out) = std::process::Command::new("git")
        .current_dir(root)
        .args(["merge-base", "HEAD", "origin/main"])
        .output()
    {
        if out.status.success() {
            base = String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    if base.is_empty() {
        base = "HEAD~1".to_string();
    }

    if let Ok(out) = std::process::Command::new("git")
        .current_dir(root)
        .args(["diff", "--name-only", &format!("{base}...HEAD")])
        .output()
    {
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

pub fn run_check_veto_deadlines(now: DateTime<Utc>) -> Result<Vec<VetoDeadlineEntry>, String> {
    let root: PathBuf = crate::find_root_dir();
    let vetoes_path = root.join("VETOES.md");
    let content = std::fs::read_to_string(&vetoes_path).map_err(|e| {
        format!(
            "Kann VETOES.md unter {} nicht lesen: {e}",
            vetoes_path.display()
        )
    })?;

    let git_changed_files = get_git_changed_files(&root);
    check_veto_deadlines_from_content_scoped(&content, now, Some(&git_changed_files))
}
