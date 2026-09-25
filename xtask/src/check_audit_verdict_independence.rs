use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Grace period (72 hours) for newly created audit reports with `VERIFIED-BY-SESSION: PENDING`
/// to allow time for independent second-session verification before failing CI.
///
/// Begründung: Frisch erstellte Audit-Reports werden im DRAFT-Status (`VERIFIED-BY-SESSION: PENDING`)
/// eingecheckt und benötigen ein Zeitfenster für die Zweitprüfung durch eine zweite Session.
/// Nach Ablauf von 72 Stunden schlägt die CI fehl, wenn keine Zweitprüfung erfolgt ist.
pub const VERDICT_GRACE_PERIOD_HOURS: i64 = 72;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerdictEntry {
    pub file_path: String,
    pub line_num: usize,
    pub verdict_text: String,
    pub id: Option<String>,
    pub ts: Option<String>,
    pub session: Option<String>,
    pub verified_by_session: Option<String>,
    pub has_verified_by_field: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerdictValidationResult {
    pub is_ok: bool,
    pub is_legacy: bool,
    pub is_pending_in_grace: bool,
    pub error_message: Option<String>,
    pub warning_message: Option<String>,
}

/// Helper function to parse ISO-8601 timestamp string into `DateTime<Utc>`.
pub fn parse_iso_timestamp(ts_str: &str) -> Option<DateTime<Utc>> {
    let trimmed = ts_str.trim();
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(dt.with_timezone(&Utc));
    }
    // Fallback parsing for YYYY-MM-DD
    if trimmed.len() >= 10 {
        let date_part = &trimmed[..10];
        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
            if let Some(naive_dt) = naive_date.and_hms_opt(0, 0, 0) {
                return Some(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
            }
        }
    }
    None
}

/// Parses a line of Markdown text to extract a `VerdictEntry` if `VERDICT:` is present.
pub fn parse_verdict_line(line: &str, file_path: &str, line_num: usize) -> Option<VerdictEntry> {
    let trimmed = line.trim();

    // Check if line contains VERDICT: (case-insensitive)
    let verdict_idx = match trimmed.to_uppercase().find("VERDICT:") {
        Some(idx) => idx,
        None => return None,
    };

    let after_verdict = &trimmed[verdict_idx + 8..];
    let end_idx = after_verdict.find('(').unwrap_or(after_verdict.len());
    let raw_verdict = after_verdict[..end_idx]
        .trim_matches(&['*', '_', '`', ' ', ':', '-'][..])
        .trim();

    if raw_verdict.is_empty() {
        return None;
    }

    let id = if let Some(idx) = trimmed.find("(ID:") {
        let rest = &trimmed[idx + 4..];
        rest.find(')').map(|e| rest[..e].trim().to_string())
    } else {
        None
    };

    let ts = if let Some(idx) = trimmed.find("(TS:") {
        let rest = &trimmed[idx + 4..];
        rest.find(')').map(|e| rest[..e].trim().to_string())
    } else {
        None
    };

    let session = if let Some(idx) = trimmed.find("(SESSION:") {
        let rest = &trimmed[idx + 9..];
        rest.find(')').map(|e| rest[..e].trim().to_string())
    } else {
        None
    };

    let has_verified_by_field = trimmed.contains("(VERIFIED-BY-SESSION:");
    let verified_by_session = if has_verified_by_field {
        let idx = trimmed.find("(VERIFIED-BY-SESSION:").unwrap();
        let rest = &trimmed[idx + 21..];
        rest.find(')').map(|e| rest[..e].trim().to_string())
    } else {
        None
    };

    Some(VerdictEntry {
        file_path: file_path.to_string(),
        line_num,
        verdict_text: raw_verdict.to_string(),
        id,
        ts,
        session,
        verified_by_session,
        has_verified_by_field,
    })
}

/// Checks if a verdict text represents a positive or conditional audit verdict requiring verification.
pub fn is_actionable_verdict(verdict_text: &str) -> bool {
    let upper = verdict_text.to_uppercase();
    upper.contains("GO")
        || upper.contains("APPROVED")
        || upper.contains("CONDITIONAL")
        || upper.contains("PASS")
}

/// Validates a single `VerdictEntry` against Audit Intake Protocol v2 independence rules.
pub fn validate_verdict_entry(
    entry: &VerdictEntry,
    now: DateTime<Utc>,
    grace_period_hours: i64,
) -> VerdictValidationResult {
    // APM-GATE-1 Transition Rule:
    // Reports without any `VERIFIED-BY-SESSION` field (Alt-Format vor Protokoll v2)
    // werden NICHT gefailt, sondern nur mit einer Warnung geloggt.
    if !entry.has_verified_by_field {
        return VerdictValidationResult {
            is_ok: true,
            is_legacy: true,
            is_pending_in_grace: false,
            error_message: None,
            warning_message: Some(format!(
                "⚠️ [Legacy-Format] {}:{} — Audit Verdict \"{}\" hat kein (VERIFIED-BY-SESSION: ...) Feld. (Alt-Report vor Protokoll v2)",
                entry.file_path, entry.line_num, entry.verdict_text
            )),
        };
    }

    let verified_by = match &entry.verified_by_session {
        Some(v) => v.trim(),
        None => "PENDING",
    };

    if verified_by.eq_ignore_ascii_case("PENDING") {
        // Check 72h grace period using TS timestamp
        let ts_dt = entry.ts.as_deref().and_then(parse_iso_timestamp);
        let is_within_grace = match ts_dt {
            Some(dt) => {
                let hours_elapsed = (now - dt).num_hours();
                hours_elapsed <= grace_period_hours
            }
            None => {
                // If TS timestamp cannot be parsed, treat as outside grace period unless explicitly recent
                false
            }
        };

        if is_within_grace {
            return VerdictValidationResult {
                is_ok: true,
                is_legacy: false,
                is_pending_in_grace: true,
                error_message: None,
                warning_message: Some(format!(
                    "⚠️ [DRAFT / Grace-Period] {}:{} — Audit Verdict \"{}\" ist PENDING (Zweitprüfung ausstehend, Kulanzfrist <= {}h aktiv).",
                    entry.file_path, entry.line_num, entry.verdict_text, grace_period_hours
                )),
            };
        } else {
            return VerdictValidationResult {
                is_ok: false,
                is_legacy: false,
                is_pending_in_grace: false,
                error_message: Some(format!(
                    "❌ [GATE FAILED] {}:{} — Audit Verdict \"{}\" (ID: {}) ist VERIFIED-BY-SESSION: PENDING, jedoch ist die Kulanzfrist von {} Stunden abgelaufen!\n    Handlungsanweisung: Führe eine unabhängige Zweitprüfung in einer neuen Session durch und trage deren Session-Hash in `(VERIFIED-BY-SESSION: <hash>)` ein.",
                    entry.file_path,
                    entry.line_num,
                    entry.verdict_text,
                    entry.id.as_deref().unwrap_or("N/A"),
                    grace_period_hours
                )),
                warning_message: None,
            };
        }
    }

    // Check self-verification rule: SESSION == VERIFIED-BY-SESSION is FORBIDDEN
    if let Some(creator_session) = &entry.session {
        if creator_session.trim().eq_ignore_ascii_case(verified_by) {
            return VerdictValidationResult {
                is_ok: false,
                is_legacy: false,
                is_pending_in_grace: false,
                error_message: Some(format!(
                    "❌ [GATE FAILED] {}:{} — Audit Verdict \"{}\" (ID: {}) verstößt gegen die Unabhängigkeitspflicht:\n    SESSION ({}) ist identisch mit VERIFIED-BY-SESSION ({}) (Selbst-Bestätigung verboten!).\n    Handlungsanweisung: Das Audit-Ergebnis muss von einer abweichenden, unabhängigen Zweitsession verifiziert werden.",
                    entry.file_path,
                    entry.line_num,
                    entry.verdict_text,
                    entry.id.as_deref().unwrap_or("N/A"),
                    creator_session,
                    verified_by
                )),
                warning_message: None,
            };
        }
    }

    // Valid independent verification session hash present
    VerdictValidationResult {
        is_ok: true,
        is_legacy: false,
        is_pending_in_grace: false,
        error_message: None,
        warning_message: None,
    }
}

/// Recursively finds all `AUDIT_*.md` files under `docs/audits/`.
pub fn find_audit_files(root: &Path) -> Vec<PathBuf> {
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
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "md" {
                    files.push(path.to_path_buf());
                }
            }
        }
    }
    files
}

/// Runs the `check-audit-verdict-independence` gate logic across `docs/audits/*.md`.
pub fn run_check_audit_verdict_independence() -> bool {
    println!("=== Gate: check-audit-verdict-independence ===");

    let root = crate::find_root_dir();
    let audit_files = find_audit_files(&root);

    if audit_files.is_empty() {
        println!("No markdown files found in docs/audits/.");
        return true;
    }

    let now = Utc::now();
    let mut total_verdicts: usize = 0;
    let mut legacy_count: usize = 0;
    let mut pending_in_grace_count: usize = 0;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for file_path in &audit_files {
        let rel_path = file_path
            .strip_prefix(&root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        if let Ok(content) = fs::read_to_string(file_path) {
            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;
                if let Some(entry) = parse_verdict_line(line, &rel_path, line_num) {
                    if is_actionable_verdict(&entry.verdict_text) {
                        total_verdicts += 1;
                        let res = validate_verdict_entry(&entry, now, VERDICT_GRACE_PERIOD_HOURS);
                        if res.is_legacy {
                            legacy_count += 1;
                        }
                        if res.is_pending_in_grace {
                            pending_in_grace_count += 1;
                        }
                        if let Some(warn) = res.warning_message {
                            warnings.push(warn);
                        }
                        if let Some(err) = res.error_message {
                            errors.push(err);
                        }
                    }
                }
            }
        }
    }

    println!(
        "Audited {} file(s) in docs/audits/, evaluated {} verdict entry(ies).",
        audit_files.len(),
        total_verdicts
    );
    println!(
        "Summary: {} verified / valid, {} legacy format (warning only), {} pending in grace period.",
        total_verdicts.saturating_sub(legacy_count + errors.len()),
        legacy_count,
        pending_in_grace_count
    );

    for warn in &warnings {
        println!("{}", warn);
    }

    if errors.is_empty() {
        println!("✅ Gate check-audit-verdict-independence PASSED.");
        true
    } else {
        eprintln!(
            "❌ Gate check-audit-verdict-independence FAILED with {} error(s):",
            errors.len()
        );
        for err in &errors {
            eprintln!("{}", err);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_independently_verified_verdict_passes() {
        let line = "**VERDICT: GO / APPROVED** (ID: AGT-AUDIT-001) (TS: 2026-09-13T10:00:00Z) (SESSION: a1b2c3d4) (VERIFIED-BY-SESSION: e5f6a7b8)";
        let entry = parse_verdict_line(line, "docs/audits/AUDIT_test.md", 10).unwrap();
        assert_eq!(entry.verdict_text, "GO / APPROVED");
        assert_eq!(entry.id.as_deref(), Some("AGT-AUDIT-001"));
        assert_eq!(entry.session.as_deref(), Some("a1b2c3d4"));
        assert_eq!(entry.verified_by_session.as_deref(), Some("e5f6a7b8"));
        assert!(entry.has_verified_by_field);

        let now = DateTime::parse_from_rfc3339("2026-09-13T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let res = validate_verdict_entry(&entry, now, 72);
        assert!(res.is_ok);
        assert!(!res.is_legacy);
        assert!(!res.is_pending_in_grace);
        assert!(res.error_message.is_none());
    }

    #[test]
    fn test_pending_verdict_within_grace_period_passes_with_warning() {
        let line = "**VERDICT: GO / APPROVED** (ID: AGT-AUDIT-002) (TS: 2026-09-13T10:00:00Z) (SESSION: a1b2c3d4) (VERIFIED-BY-SESSION: PENDING)";
        let entry = parse_verdict_line(line, "docs/audits/AUDIT_test.md", 15).unwrap();
        assert_eq!(entry.verified_by_session.as_deref(), Some("PENDING"));

        // Timestamp 10 hours ago (< 72h grace period)
        let now = DateTime::parse_from_rfc3339("2026-09-13T20:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let res = validate_verdict_entry(&entry, now, 72);
        assert!(res.is_ok, "PENDING within grace period must pass");
        assert!(res.is_pending_in_grace);
        assert!(res.warning_message.is_some());
        assert!(res.error_message.is_none());
    }

    #[test]
    fn test_pending_verdict_outside_grace_period_fails() {
        let line = "**VERDICT: GO / APPROVED** (ID: AGT-AUDIT-003) (TS: 2026-09-08T10:00:00Z) (SESSION: a1b2c3d4) (VERIFIED-BY-SESSION: PENDING)";
        let entry = parse_verdict_line(line, "docs/audits/AUDIT_test.md", 20).unwrap();

        // Timestamp 120 hours ago (> 72h grace period)
        let now = DateTime::parse_from_rfc3339("2026-09-13T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let res = validate_verdict_entry(&entry, now, 72);
        assert!(!res.is_ok, "PENDING outside grace period must fail");
        assert!(res.error_message.is_some());
        assert!(res
            .error_message
            .unwrap()
            .contains("Kulanzfrist von 72 Stunden abgelaufen"));
    }

    #[test]
    fn test_self_verified_verdict_fails() {
        let line = "**VERDICT: GO / APPROVED** (ID: AGT-AUDIT-004) (TS: 2026-09-13T10:00:00Z) (SESSION: a1b2c3d4) (VERIFIED-BY-SESSION: a1b2c3d4)";
        let entry = parse_verdict_line(line, "docs/audits/AUDIT_test.md", 25).unwrap();

        let now = DateTime::parse_from_rfc3339("2026-09-13T11:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let res = validate_verdict_entry(&entry, now, 72);
        assert!(
            !res.is_ok,
            "Self-verification (SESSION == VERIFIED-BY-SESSION) must fail"
        );
        assert!(res.error_message.is_some());
        assert!(res
            .error_message
            .unwrap()
            .contains("Selbst-Bestätigung verboten"));
    }

    #[test]
    fn test_legacy_audit_verdict_passes_with_warning() {
        let line = "**VERDICT: GO / APPROVED**. `contextra-vector` erfüllt alle Tier-1 Qualitäts-Invarianten.";
        let entry = parse_verdict_line(line, "docs/audits/AUDIT_contextra-vector.md", 281).unwrap();
        assert!(!entry.has_verified_by_field);

        let now = Utc::now();
        let res = validate_verdict_entry(&entry, now, 72);
        assert!(
            res.is_ok,
            "Legacy report without VERIFIED-BY-SESSION must pass with warning per APM-GATE-1"
        );
        assert!(res.is_legacy);
        assert!(res.warning_message.is_some());
        assert!(res
            .warning_message
            .unwrap()
            .contains("Alt-Report vor Protokoll v2"));
    }

    #[test]
    fn test_no_verdict_in_document_returns_none() {
        let line = "## 1. Executive Summary & Audit Overview";
        assert!(parse_verdict_line(line, "docs/audits/AUDIT_test.md", 1).is_none());
    }
}
