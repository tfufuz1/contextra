// Contextra — VETO Review Deadline CI Gate
//
// Parst VETOES.md und überwacht Review-Fristen für bedingt akzeptierte Features ("conditionally_accepted").

use chrono::{DateTime, NaiveDate, Utc};
use std::path::PathBuf;

pub const VETO_DEADLINE_WARNING_THRESHOLD_DAYS: i64 = 14;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VetoDeadlineEntry {
    pub veto_id: String,
    pub feature_id: String,
    pub status: String,
    pub conditional_review_due: Option<String>,
    pub adr_ref: Option<String>,
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
            });
        }
    }

    Ok(entries)
}

pub fn check_veto_deadlines_from_content(
    content: &str,
    now: DateTime<Utc>,
) -> Result<Vec<VetoDeadlineEntry>, String> {
    let entries = parse_veto_deadlines(content)?;
    let today = now.date_naive();

    let mut overdue_ids = Vec::new();
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
                overdue_ids.push(entry.veto_id.clone());
            } else if days_remaining <= VETO_DEADLINE_WARNING_THRESHOLD_DAYS {
                warning_entries.push(entry);
            }
        }
    }

    if !overdue_ids.is_empty() {
        return Err(format!(
            "❌ VETO-REVIEW-FRIST ÜBERSCHRITTEN: Folgende VETO-Einträge sind überfällig: {}. Bitte Frist per ADR verlängern oder Status in VETOES.md anpassen.",
            overdue_ids.join(", ")
        ));
    }

    Ok(warning_entries)
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
    check_veto_deadlines_from_content(&content, now)
}
