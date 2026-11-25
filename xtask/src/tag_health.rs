use chrono::{DateTime, Utc};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagHealthConcern {
    FileModifiedAfterTag {
        last_change: String,
        changes_since: usize,
    },
    LocalHotspot {
        changes_in_10_lines: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagHealthFinding {
    pub file: String,
    pub line: usize,
    pub tag_id: Option<String>,
    pub tag_timestamp: String,
    pub concern: TagHealthConcern,
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

pub fn run_tag_health_impl(
    root: &Path,
    threshold_days: i64,
) -> Result<Vec<TagHealthFinding>, String> {
    let tag_items = crate::check_stale_tags::scan_audit_tags_in_root(root);
    let mut findings = Vec::new();

    let since_arg = format!("--since={} days ago", threshold_days);

    for item in &tag_items {
        let tag_dt = match parse_iso_datetime(&item.tag_ts) {
            Some(dt) => dt,
            None => continue,
        };

        let line_commit = crate::check_stale_tags::get_line_last_commit_date(
            root,
            &item.file_path,
            item.line_num,
        );

        let is_line_untouched = if let Some(ref commit_iso) = line_commit {
            if let Some(commit_dt) = parse_iso_datetime(commit_iso) {
                // Line has not been modified after tag timestamp (allowing 60s clock skew)
                commit_dt.timestamp() <= tag_dt.timestamp() + 60
            } else {
                true
            }
        } else {
            true
        };

        // Check concern 1: FileModifiedAfterTag
        if is_line_untouched {
            let file_log = Command::new("git")
                .current_dir(root)
                .args(["log", &since_arg, "--format=%H|%cI", "--", &item.file_path])
                .output();

            if let Ok(fl_out) = file_log {
                if fl_out.status.success() {
                    let fl_stdout = String::from_utf8_lossy(&fl_out.stdout);
                    let mut changes_after_tag = 0;
                    let mut latest_change_date: Option<String> = None;

                    for line in fl_stdout.lines() {
                        let parts: Vec<&str> = line.split('|').collect();
                        if parts.len() < 2 {
                            continue;
                        }
                        let c_date = parts[1].trim();
                        if let Some(c_dt) = parse_iso_datetime(c_date) {
                            if c_dt.timestamp() > tag_dt.timestamp() + 60 {
                                changes_after_tag += 1;
                                if latest_change_date.is_none() {
                                    latest_change_date = Some(c_date.to_string());
                                }
                            }
                        }
                    }

                    if changes_after_tag > 0 {
                        if let Some(last_change) = latest_change_date {
                            findings.push(TagHealthFinding {
                                file: item.file_path.clone(),
                                line: item.line_num,
                                tag_id: item.tag_id.clone(),
                                tag_timestamp: item.tag_ts.clone(),
                                concern: TagHealthConcern::FileModifiedAfterTag {
                                    last_change,
                                    changes_since: changes_after_tag,
                                },
                            });
                        }
                    }
                }
            }
        }

        // Check concern 2: LocalHotspot
        let start_line = item.line_num.saturating_sub(10).max(1);
        let end_line = item.line_num + 10;
        let range_arg = format!("{},{}:{}", start_line, end_line, item.file_path);

        let local_log = Command::new("git")
            .current_dir(root)
            .args(["log", &since_arg, "-L", &range_arg, "--format=%H"])
            .output();

        if let Ok(ll_out) = local_log {
            if ll_out.status.success() {
                let ll_stdout = String::from_utf8_lossy(&ll_out.stdout);
                let mut commit_hashes = std::collections::HashSet::new();
                for line in ll_stdout.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("commit ") {
                        let hash = trimmed["commit ".len()..].trim();
                        if !hash.is_empty() {
                            commit_hashes.insert(hash.to_string());
                        }
                    } else if trimmed.len() == 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit())
                    {
                        commit_hashes.insert(trimmed.to_string());
                    }
                }

                let changes_in_10_lines = commit_hashes.len();
                if changes_in_10_lines > 5 {
                    findings.push(TagHealthFinding {
                        file: item.file_path.clone(),
                        line: item.line_num,
                        tag_id: item.tag_id.clone(),
                        tag_timestamp: item.tag_ts.clone(),
                        concern: TagHealthConcern::LocalHotspot {
                            changes_in_10_lines,
                        },
                    });
                }
            }
        }
    }

    Ok(findings)
}

pub fn run_tag_health(threshold_days: i64) -> Result<Vec<TagHealthFinding>, String> {
    let root = xtask::find_root_dir();
    run_tag_health_impl(&root, threshold_days)
}
