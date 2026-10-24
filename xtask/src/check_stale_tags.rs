use chrono::{DateTime, NaiveDate, Utc};
use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

/// Stichtag (Cutoff Date), ab dem Tags OHNE `until=YYYY-MM-DD` als Fehler (rot) behandelt werden.
/// Bis zu diesem Stichtag werden sie als Warnung mit Zaehler gemeldet.
pub const TAG_UNTIL_MANDATORY_CUTOFF_DATE: &str = "2026-12-31";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagIssueType {
    Expired { until_date: String },
    InvalidUntilFormat { raw_until: String },
    MissingUntilAfterCutoff,
    StaleUntouched { threshold_days: i64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaleTagCandidate {
    pub file_path: String,
    pub line_num: usize,
    pub tag_id: Option<String>,
    pub tag_ts: String,
    pub line_commit_ts: Option<String>,
    pub age_days: i64,
    pub raw: String,
    pub until: Option<String>,
    pub issue_type: TagIssueType,
}

#[derive(Debug, Clone)]
pub struct TagScanItem {
    pub file_path: String,
    pub line_num: usize,
    pub tag_id: Option<String>,
    pub tag_ts: String,
    pub until_raw: Option<String>,
    pub raw: String,
}

pub fn scan_audit_tags_in_root(root: &Path) -> Vec<TagScanItem> {
    let crates_dir = root.join("crates");
    if !crates_dir.exists() {
        return Vec::new();
    }

    let mut items = Vec::new();
    let re_audit_tag = match Regex::new(r"AI-TAG\[.*?\]\s*TODO\(audit-[^)]+\)") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let re_ts = match Regex::new(r"\(TS:\s*([0-9T:-]+Z)\)") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let re_id = match Regex::new(r"\(ID:\s*([^\s\)]+)\)") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let re_until = match Regex::new(r"until=([^\s\)\],]+)") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    for entry in WalkDir::new(&crates_dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            if let Ok(content) = fs::read_to_string(path) {
                for (idx, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip resolved / completed tags
                    if trimmed.contains("RESOLVED")
                        || trimmed.contains("STATUS:DONE")
                        || trimmed.contains("STATUS:RESOLVED")
                    {
                        continue;
                    }

                    if re_audit_tag.is_match(trimmed) {
                        let tag_ts = if let Some(caps) = re_ts.captures(trimmed) {
                            caps[1].to_string()
                        } else {
                            continue;
                        };

                        let tag_id = re_id.captures(trimmed).map(|caps| caps[1].to_string());
                        let until_raw = re_until.captures(trimmed).map(|caps| caps[1].to_string());

                        items.push(TagScanItem {
                            file_path: rel_path.clone(),
                            line_num: idx + 1,
                            tag_id,
                            tag_ts,
                            until_raw,
                            raw: trimmed.to_string(),
                        });
                    }
                }
            }
        }
    }

    items
}

pub fn get_line_last_commit_date(root: &Path, file_path: &str, line_num: usize) -> Option<String> {
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "log",
            "-1",
            "--format=%aI",
            "-L",
            &format!("{},{}:{}", line_num, line_num, file_path),
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next()?.trim();
    if first_line.is_empty() {
        None
    } else {
        Some(first_line.to_string())
    }
}

pub fn parse_iso_datetime(ts_str: &str) -> Option<DateTime<Utc>> {
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

pub fn parse_until_date(until_str: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(until_str.trim(), "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format '{}': {}", until_str, e))
}

pub fn check_stale_tags_impl(
    root: &Path,
    threshold_days: i64,
    strict: bool,
    now: DateTime<Utc>,
) -> Result<Vec<StaleTagCandidate>, String> {
    println!(
        "=== Gate: check-stale-tags (threshold = {} days, strict = {}) ===",
        threshold_days, strict
    );

    let tag_items = scan_audit_tags_in_root(root);
    if tag_items.is_empty() {
        println!("✅ check-stale-tags: No open AI-TAG audit comments found.");
        return Ok(Vec::new());
    }

    let today_date = now.date_naive();
    let cutoff_date = NaiveDate::parse_from_str(TAG_UNTIL_MANDATORY_CUTOFF_DATE, "%Y-%m-%d")
        .ok()
        .or_else(|| NaiveDate::from_ymd_opt(2026, 12, 31))
        .unwrap_or(NaiveDate::MIN);

    let mut hard_errors = Vec::new();
    let mut warnings = Vec::new();
    let mut missing_until_count = 0usize;

    for item in &tag_items {
        let mut has_until_issue = false;

        if let Some(ref until_str) = item.until_raw {
            match parse_until_date(until_str) {
                Ok(until_date) => {
                    if today_date > until_date {
                        has_until_issue = true;
                        hard_errors.push(StaleTagCandidate {
                            file_path: item.file_path.clone(),
                            line_num: item.line_num,
                            tag_id: item.tag_id.clone(),
                            tag_ts: item.tag_ts.clone(),
                            line_commit_ts: None,
                            age_days: (today_date - until_date).num_days(),
                            raw: item.raw.clone(),
                            until: Some(until_str.clone()),
                            issue_type: TagIssueType::Expired {
                                until_date: until_date.to_string(),
                            },
                        });
                    }
                }
                Err(_) => {
                    has_until_issue = true;
                    hard_errors.push(StaleTagCandidate {
                        file_path: item.file_path.clone(),
                        line_num: item.line_num,
                        tag_id: item.tag_id.clone(),
                        tag_ts: item.tag_ts.clone(),
                        line_commit_ts: None,
                        age_days: 0,
                        raw: item.raw.clone(),
                        until: Some(until_str.clone()),
                        issue_type: TagIssueType::InvalidUntilFormat {
                            raw_until: until_str.clone(),
                        },
                    });
                }
            }
        } else {
            missing_until_count += 1;
            if today_date >= cutoff_date {
                has_until_issue = true;
                hard_errors.push(StaleTagCandidate {
                    file_path: item.file_path.clone(),
                    line_num: item.line_num,
                    tag_id: item.tag_id.clone(),
                    tag_ts: item.tag_ts.clone(),
                    line_commit_ts: None,
                    age_days: 0,
                    raw: item.raw.clone(),
                    until: None,
                    issue_type: TagIssueType::MissingUntilAfterCutoff,
                });
            }
        }

        if !has_until_issue {
            if let Some(tag_dt) = parse_iso_datetime(&item.tag_ts) {
                let age_days = (now - tag_dt).num_days();
                if age_days >= threshold_days {
                    let line_commit =
                        get_line_last_commit_date(root, &item.file_path, item.line_num);

                    let is_untouched = if let Some(ref commit_iso) = line_commit {
                        if let Some(commit_dt) = parse_iso_datetime(commit_iso) {
                            commit_dt.timestamp() <= tag_dt.timestamp() + 60
                        } else {
                            true
                        }
                    } else {
                        true
                    };

                    if is_untouched {
                        if item.until_raw.is_none() {
                            let cand = StaleTagCandidate {
                                file_path: item.file_path.clone(),
                                line_num: item.line_num,
                                tag_id: item.tag_id.clone(),
                                tag_ts: item.tag_ts.clone(),
                                line_commit_ts: line_commit,
                                age_days,
                                raw: item.raw.clone(),
                                until: None,
                                issue_type: TagIssueType::StaleUntouched { threshold_days },
                            };
                            if strict {
                                hard_errors.push(cand);
                            } else {
                                warnings.push(cand);
                            }
                        }
                    }
                }
            }
        }
    }

    if missing_until_count > 0 && today_date < cutoff_date {
        println!(
            "⚠️ Warning: Found {} tag(s) without `until=YYYY-MM-DD` expiration date (mandatory starting {}).",
            missing_until_count, TAG_UNTIL_MANDATORY_CUTOFF_DATE
        );
    }

    if !warnings.is_empty() {
        println!(
            "⚠️ Warning: Found {} stale audit tag candidate(s) older than {} days without line modification!",
            warnings.len(),
            threshold_days
        );
        for c in &warnings {
            println!(
                "  - File: {}:{}\n    Tag ID: {}\n    Tag TS: {}\n    Line Last Commit: {}\n    Tag Age: {} days\n    Raw: {}\n",
                c.file_path,
                c.line_num,
                c.tag_id.as_deref().unwrap_or("N/A"),
                c.tag_ts,
                c.line_commit_ts.as_deref().unwrap_or("unknown"),
                c.age_days,
                c.raw
            );
        }
    }

    if !hard_errors.is_empty() {
        println!(
            "❌ Error: Detected {} expired/invalid/stale audit tag candidate(s):",
            hard_errors.len()
        );
        for c in &hard_errors {
            let issue_desc = match &c.issue_type {
                TagIssueType::Expired { until_date } => format!("Expired on {}", until_date),
                TagIssueType::InvalidUntilFormat { raw_until } => {
                    format!("Invalid until date format '{}'", raw_until)
                }
                TagIssueType::MissingUntilAfterCutoff => format!(
                    "Missing until date (mandatory as of {})",
                    TAG_UNTIL_MANDATORY_CUTOFF_DATE
                ),
                TagIssueType::StaleUntouched { threshold_days } => {
                    format!("Stale (>{} days untouched)", threshold_days)
                }
            };
            println!(
                "  - File: {}:{}\n    Tag ID: {}\n    Tag TS: {}\n    Until: {}\n    Issue: {}\n    Raw: {}\n",
                c.file_path,
                c.line_num,
                c.tag_id.as_deref().unwrap_or("N/A"),
                c.tag_ts,
                c.until.as_deref().unwrap_or("N/A"),
                issue_desc,
                c.raw
            );
        }
        Err(format!(
            "{} stale/expired audit tag candidate(s) detected.",
            hard_errors.len()
        ))
    } else {
        println!(
            "✅ check-stale-tags: All scanned tags ({}) passed expiration/format checks.",
            tag_items.len()
        );
        Ok(warnings)
    }
}

pub fn run_check_stale_tags(threshold_days: i64, strict: bool) -> Result<(), String> {
    let root = xtask::find_root_dir();
    let now = Utc::now();
    check_stale_tags_impl(&root, threshold_days, strict, now).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_audit_tags_regex() {
        let sample = r#"
        // AI-TAG[SMELL][MINOR] TODO(audit-C-1): Fix startup flush. (ID: AGT-STORE-bae66245) (TS: 2026-09-10T19:14:58Z) (SESSION: 21a8d3e8)
        // AI-TAG[SMELL][MINOR] TODO(audit-C-2): Already fixed. (ID: AGT-STORE-12345678) (TS: 2026-09-10T19:14:58Z) STATUS:RESOLVED
        // AI-TAG[PERF][MINOR] Non-audit tag (TS: 2026-09-10T19:14:58Z)
        "#;

        let re_audit = Regex::new(r"AI-TAG\[.*?\]\s*TODO\(audit-[^)]+\)").unwrap();
        assert_eq!(re_audit.find_iter(sample).count(), 2);
    }

    #[test]
    fn test_stale_tags_fixture() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();

        let run_git = |args: &[&str]| {
            let status = Command::new("git")
                .current_dir(root)
                .args(args)
                .status()
                .expect("Failed to execute git");
            assert!(status.success());
        };

        run_git(&["init"]);
        run_git(&["config", "user.name", "Test User"]);
        run_git(&["config", "user.email", "test@example.com"]);

        let crate_dir = root.join("crates/contextra-test/src");
        fs::create_dir_all(&crate_dir).unwrap();

        let file_path = crate_dir.join("lib.rs");
        let code_content = r#"
// Line 1
// AI-TAG[SMELL][MINOR] TODO(audit-1.1): Old stale audit tag (ID: AGT-TEST-00000001) (TS: 2026-01-01T00:00:00Z) (SESSION: 00000001)
// AI-TAG[SMELL][MINOR] TODO(audit-1.2): Recent audit tag (ID: AGT-TEST-00000002) (TS: 2026-09-10T00:00:00Z) (SESSION: 00000002)
"#;
        fs::write(&file_path, code_content).unwrap();

        run_git(&["add", "."]);
        let mut cmd = Command::new("git");
        cmd.current_dir(root)
            .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
            .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
            .args(["commit", "-m", "initial commit"]);
        assert!(cmd.status().unwrap().success());

        let now = DateTime::parse_from_rfc3339("2026-09-11T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let candidates = check_stale_tags_impl(root, 60, false, now).expect("Execution failed");

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].tag_id.as_deref(), Some("AGT-TEST-00000001"));
        assert_eq!(candidates[0].line_num, 3);
    }
}
