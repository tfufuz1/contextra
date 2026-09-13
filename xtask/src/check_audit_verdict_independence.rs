use chrono::{DateTime, Utc};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const PENDING_GRACE_PERIOD_HOURS: i64 = 72;

#[derive(Debug, PartialEq, Clone)]
pub enum VerdictStatus {
    AltFormatWarning(String),
    PendingPass(String),
    PendingFail(String),
    SelfVerificationFail {
        author_session: String,
        verifier_session: String,
    },
    VerifiedPass {
        author_session: Option<String>,
        verifier_session: String,
    },
}

impl VerdictStatus {
    pub fn is_fail(&self) -> bool {
        matches!(
            self,
            VerdictStatus::PendingFail(_) | VerdictStatus::SelfVerificationFail { .. }
        )
    }
}

#[derive(Debug, Clone)]
pub struct VerdictIssue {
    pub file_path: String,
    pub line_number: usize,
    pub verdict_text: String,
    pub status: VerdictStatus,
}

pub fn find_all_audit_md_files(root: &Path) -> Vec<PathBuf> {
    let audits_dir = root.join("docs/audits");
    if !audits_dir.exists() {
        return Vec::new();
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(&audits_dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.ends_with(".md") {
                    files.push(path.to_path_buf());
                }
            }
        }
    }
    files
}

pub fn parse_timestamp(ts_str: &str) -> Option<DateTime<Utc>> {
    let ts_str = ts_str.trim();
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts_str) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = DateTime::parse_from_str(ts_str, "%Y-%m-%dT%H:%M:%SZ") {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = DateTime::parse_from_str(ts_str, "%Y-%m-%dT%H:%M:%S") {
        return Some(dt.with_timezone(&Utc));
    }
    None
}

pub fn check_file_verdicts(content: &str, file_path: &str) -> Vec<VerdictIssue> {
    check_file_verdicts_at_time(content, file_path, Utc::now())
}

pub fn check_file_verdicts_at_time(
    content: &str,
    file_path: &str,
    now: DateTime<Utc>,
) -> Vec<VerdictIssue> {
    let lines: Vec<&str> = content.lines().collect();
    let verdict_regex = Regex::new(r"\*\*VERDICT:\s*(GO\s*/\s*APPROVED|CONDITIONAL)").unwrap();
    let verified_by_regex =
        Regex::new(r"<!--\s*VERIFIED-BY-SESSION:\s*([A-Za-z0-9_-]+)(?:\s+\(TS:\s*([^)]+)\))?\s*-->")
            .unwrap();
    let session_regex = Regex::new(r"(?:SESSION|Session):\s*`?([0-9a-fA-F]{6,10})`?").unwrap();
    let ts_regex =
        Regex::new(r"(?:TS|ts):\s*`?([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z?)`?")
            .unwrap();

    let mut issues = Vec::new();

    for i in 0..lines.len() {
        let line = lines[i];
        if !verdict_regex.is_match(line) {
            continue;
        }

        let line_number = i + 1;
        let verdict_text = line.trim().to_string();

        // Check directly following non-empty line
        let mut following_line_idx = None;
        for j in (i + 1)..lines.len() {
            if !lines[j].trim().is_empty() {
                following_line_idx = Some(j);
                break;
            }
        }

        let verified_match = following_line_idx
            .map(|idx| lines[idx])
            .and_then(|fl| verified_by_regex.captures(fl));

        // Upward search within section for author session and timestamp
        let mut author_session: Option<String> = None;
        let mut section_ts: Option<String> = None;

        for k in (0..i).rev() {
            let uline = lines[k];
            if uline.trim().starts_with("## ") {
                // Section boundary header reached
                // Still check the header line itself for session / ts before stopping
                if author_session.is_none() {
                    if let Some(caps) = session_regex.captures(uline) {
                        author_session = Some(caps[1].to_string());
                    }
                }
                if section_ts.is_none() {
                    if let Some(caps) = ts_regex.captures(uline) {
                        section_ts = Some(caps[1].to_string());
                    }
                }
                break;
            }
            if author_session.is_none() {
                if let Some(caps) = session_regex.captures(uline) {
                    author_session = Some(caps[1].to_string());
                }
            }
            if section_ts.is_none() {
                if let Some(caps) = ts_regex.captures(uline) {
                    section_ts = Some(caps[1].to_string());
                }
            }
        }

        let status = match verified_match {
            None => VerdictStatus::AltFormatWarning(
                "Legacy format verdict without VERIFIED-BY-SESSION HTML comment tag".to_string(),
            ),
            Some(caps) => {
                let verifier_session = caps[1].to_string();
                if verifier_session.eq_ignore_ascii_case("PENDING") {
                    if let Some(ts_str) = &section_ts {
                        if let Some(parsed_ts) = parse_timestamp(ts_str) {
                            let duration = now.signed_duration_since(parsed_ts);
                            let hours = duration.num_hours();
                            if hours >= 0 && hours <= PENDING_GRACE_PERIOD_HOURS {
                                VerdictStatus::PendingPass(format!(
                                    "PENDING verdict within {}h grace period (age: {}h)",
                                    PENDING_GRACE_PERIOD_HOURS, hours
                                ))
                            } else {
                                VerdictStatus::PendingFail(format!(
                                    "PENDING verdict expired grace period of {}h (age: {}h)",
                                    PENDING_GRACE_PERIOD_HOURS, hours
                                ))
                            }
                        } else {
                            VerdictStatus::PendingFail(format!(
                                "PENDING verdict section has unparseable TS timestamp: '{}'",
                                ts_str
                            ))
                        }
                    } else {
                        VerdictStatus::PendingFail(
                            "PENDING verdict section missing TS timestamp".to_string(),
                        )
                    }
                } else if let Some(author) = &author_session {
                    if author.eq_ignore_ascii_case(&verifier_session) {
                        VerdictStatus::SelfVerificationFail {
                            author_session: author.clone(),
                            verifier_session,
                        }
                    } else {
                        VerdictStatus::VerifiedPass {
                            author_session: Some(author.clone()),
                            verifier_session,
                        }
                    }
                } else {
                    VerdictStatus::VerifiedPass {
                        author_session: None,
                        verifier_session,
                    }
                }
            }
        };

        issues.push(VerdictIssue {
            file_path: file_path.to_string(),
            line_number,
            verdict_text,
            status,
        });
    }

    issues
}

pub fn run_check_audit_verdict_independence() -> Result<(), String> {
    println!("=== Gate: check-audit-verdict-independence ===");

    let root = crate::find_root_dir();
    let audit_files = find_all_audit_md_files(&root);

    if audit_files.is_empty() {
        println!("No .md files found in docs/audits/.");
        return Ok(());
    }

    let mut total_issues = Vec::new();
    let mut fail_count = 0;
    let mut warning_count = 0;
    let mut pass_count = 0;

    for file_path in &audit_files {
        let rel_path = file_path
            .strip_prefix(&root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        if let Ok(content) = fs::read_to_string(file_path) {
            let issues = check_file_verdicts(&content, &rel_path);
            for issue in issues {
                if issue.status.is_fail() {
                    fail_count += 1;
                } else if matches!(
                    issue.status,
                    VerdictStatus::AltFormatWarning(_) | VerdictStatus::PendingPass(_)
                ) {
                    warning_count += 1;
                } else {
                    pass_count += 1;
                }
                total_issues.push(issue);
            }
        }
    }

    println!(
        "Scanned {} files under docs/audits/. Total verdicts: {} (Passed: {}, Warnings: {}, Fails: {})",
        audit_files.len(),
        total_issues.len(),
        pass_count,
        warning_count,
        fail_count
    );

    for issue in &total_issues {
        match &issue.status {
            VerdictStatus::AltFormatWarning(msg) => {
                println!(
                    "⚠️  ALT-FORMAT WARNING [{}:{}] {}\n    Detail: {}",
                    issue.file_path, issue.line_number, issue.verdict_text, msg
                );
            }
            VerdictStatus::PendingPass(msg) => {
                println!(
                    "ℹ️  PENDING NOTICE [{}:{}] {}\n    Detail: {}",
                    issue.file_path, issue.line_number, issue.verdict_text, msg
                );
            }
            VerdictStatus::PendingFail(msg) => {
                eprintln!(
                    "❌ PENDING FAIL [{}:{}] {}\n    Detail: {}\n    Action Required: Second session must verify and update VERIFIED-BY-SESSION tag within 72h.",
                    issue.file_path, issue.line_number, issue.verdict_text, msg
                );
            }
            VerdictStatus::SelfVerificationFail {
                author_session,
                verifier_session,
            } => {
                eprintln!(
                    "❌ SELF-VERIFICATION FAIL [{}:{}] {}\n    Author Session: {}, Verifier Session: {}\n    Action Required: Author session cannot self-verify. A distinct second session must perform audit verification.",
                    issue.file_path, issue.line_number, issue.verdict_text, author_session, verifier_session
                );
            }
            VerdictStatus::VerifiedPass {
                author_session,
                verifier_session,
            } => {
                println!(
                    "✅ VERIFIED [{}:{}] {}\n    Author: {}, Verifier: {}",
                    issue.file_path,
                    issue.line_number,
                    issue.verdict_text,
                    author_session.as_deref().unwrap_or("unknown"),
                    verifier_session
                );
            }
        }
    }

    if fail_count > 0 {
        Err(format!(
            "check-audit-verdict-independence failed with {} verdict failure(s).",
            fail_count
        ))
    } else {
        println!("✅ check-audit-verdict-independence passed cleanly with 0 failures.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_valid_independent_verified_verdict_pass() {
        let markdown = r#"
## 15. Audit Verification (TS: 2026-09-09T12:00:00Z) (SESSION: 11b56d50)

### 15.4 Verdict
**VERDICT: GO / APPROVED**
<!-- VERIFIED-BY-SESSION: 92d7bb7d (TS: 2026-09-09T14:50:00Z) -->
"#;
        let now = Utc.with_ymd_and_hms(2026, 9, 10, 12, 0, 0).unwrap();
        let issues = check_file_verdicts_at_time(markdown, "docs/audits/test.md", now);
        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].status,
            VerdictStatus::VerifiedPass {
                author_session: Some("11b56d50".to_string()),
                verifier_session: "92d7bb7d".to_string(),
            }
        );
        assert!(!issues[0].status.is_fail());
    }

    #[test]
    fn test_pending_within_grace_period_pass() {
        let markdown = r#"
## 16. Audit Section (TS: 2026-09-09T12:00:00Z) (SESSION: 11b56d50)

### 16.4 Verdict
**VERDICT: GO / APPROVED**
<!-- VERIFIED-BY-SESSION: PENDING -->
"#;
        // 24 hours later -> within 72h
        let now = Utc.with_ymd_and_hms(2026, 9, 10, 12, 0, 0).unwrap();
        let issues = check_file_verdicts_at_time(markdown, "docs/audits/test.md", now);
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0].status, VerdictStatus::PendingPass(_)));
        assert!(!issues[0].status.is_fail());
    }

    #[test]
    fn test_pending_outside_grace_period_fail() {
        let markdown = r#"
## 16. Audit Section (TS: 2026-09-09T12:00:00Z) (SESSION: 11b56d50)

### 16.4 Verdict
**VERDICT: GO / APPROVED**
<!-- VERIFIED-BY-SESSION: PENDING -->
"#;
        // 96 hours later -> exceeds 72h
        let now = Utc.with_ymd_and_hms(2026, 9, 13, 13, 0, 0).unwrap();
        let issues = check_file_verdicts_at_time(markdown, "docs/audits/test.md", now);
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0].status, VerdictStatus::PendingFail(_)));
        assert!(issues[0].status.is_fail());
    }

    #[test]
    fn test_identical_session_self_verification_fail() {
        let markdown = r#"
## 16. Audit Section (TS: 2026-09-09T12:00:00Z) (SESSION: 11b56d50)

### 16.4 Verdict
**VERDICT: GO / APPROVED**
<!-- VERIFIED-BY-SESSION: 11b56d50 -->
"#;
        let now = Utc.with_ymd_and_hms(2026, 9, 10, 12, 0, 0).unwrap();
        let issues = check_file_verdicts_at_time(markdown, "docs/audits/test.md", now);
        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].status,
            VerdictStatus::SelfVerificationFail {
                author_session: "11b56d50".to_string(),
                verifier_session: "11b56d50".to_string(),
            }
        );
        assert!(issues[0].status.is_fail());
    }

    #[test]
    fn test_alt_format_without_verified_by_session_pass_warning() {
        let markdown = r#"
## 15. Audit Section (TS: 2026-09-09T12:00:00Z) (SESSION: 11b56d50)

### 15.4 Verdict
**VERDICT: GO / APPROVED**. All invariants satisfied.

## Next Section
"#;
        let now = Utc.with_ymd_and_hms(2026, 9, 10, 12, 0, 0).unwrap();
        let issues = check_file_verdicts_at_time(markdown, "docs/audits/test.md", now);
        assert_eq!(issues.len(), 1);
        assert!(matches!(
            issues[0].status,
            VerdictStatus::AltFormatWarning(_)
        ));
        assert!(!issues[0].status.is_fail());
    }
}
