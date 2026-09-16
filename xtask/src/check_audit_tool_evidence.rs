use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Audits created after this timestamp must include explicit evidence markers
/// `<!-- EVIDENCE: <log_path> (exit=0) -->`.
pub const EVIDENCE_GATE_CUTOFF_TS: &str = "2026-09-16T12:00:00Z";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvidenceViolation {
    pub file_path: String,
    pub line_num: usize,
    pub verdict_text: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceMarker {
    pub log_path: String,
    pub expected_exit: i32,
}

/// Parses evidence comments in audit Markdown files.
/// Example: `<!-- EVIDENCE: logs/audits/cov.log (exit=0) -->`
pub fn parse_evidence_markers(content: &str) -> Vec<EvidenceMarker> {
    let mut markers = Vec::new();
    for line in content.lines() {
        if let Some(pos) = line.find("EVIDENCE:") {
            let rest = line[pos + "EVIDENCE:".len()..].trim();
            let rest_clean = rest.trim_end_matches("-->").trim();
            let parts: Vec<&str> = rest_clean.split_whitespace().collect();
            if !parts.is_empty() {
                let log_path = parts[0].to_string();
                let mut expected_exit = 0;
                if parts.len() > 1 && parts[1].starts_with("(exit=") {
                    let exit_str = parts[1].trim_start_matches("(exit=").trim_end_matches(')');
                    if let Ok(code) = exit_str.parse::<i32>() {
                        expected_exit = code;
                    }
                }
                markers.push(EvidenceMarker {
                    log_path,
                    expected_exit,
                });
            }
        }
    }
    markers
}

/// Helper function to parse timestamp from file content (e.g. `TS: 2026-09-16T15:00:00Z`).
pub fn extract_audit_timestamp(content: &str) -> Option<DateTime<Utc>> {
    for line in content.lines() {
        if let Some(pos) = line.find("TS:") {
            let rest = line[pos + 3..].trim();
            let ts_part = rest.split_whitespace().next().unwrap_or("").trim_matches(&['(', ')', ',', ';'][..]);
            if let Ok(dt) = DateTime::parse_from_rfc3339(ts_part) {
                return Some(dt.with_timezone(&Utc));
            }
        }
    }
    None
}

/// Validates that audit reports with GO/APPROVED verdicts provide real evidence.
pub fn run_check_audit_tool_evidence(root: &Path) -> Result<Vec<AuditEvidenceViolation>, String> {
    let audits_dir = root.join("docs").join("audits");
    if !audits_dir.exists() {
        return Ok(Vec::new());
    }

    let cutoff_dt = DateTime::parse_from_rfc3339(EVIDENCE_GATE_CUTOFF_TS)
        .unwrap()
        .with_timezone(&Utc);

    let mut violations = Vec::new();

    for entry in WalkDir::new(&audits_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "md"))
    {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => return Err(format!("Fehler beim Lesen von {}: {}", relative_path, e)),
        };

        let evidence_markers = parse_evidence_markers(&content);
        let audit_ts = extract_audit_timestamp(&content);
        let is_post_cutoff = audit_ts.is_some_and(|ts| ts >= cutoff_dt);

        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx + 1;
            let upper = line.to_uppercase();

            let is_approved_verdict = (upper.contains("VERDICT:") && (upper.contains("GO") || upper.contains("APPROVED")))
                || upper.contains("VERDICT: GO")
                || upper.contains("VERDICT: APPROVED")
                || upper.contains("VERDICT: GO/APPROVED");

            if !is_approved_verdict {
                continue;
            }

            let mentions_skipped_tools = upper.contains("ÜBERSPRUNGEN:")
                || upper.contains("SKIPPED:")
                || upper.contains("TOOLING SKIPPED")
                || upper.contains("NOT INSTALLABLE")
                || upper.contains("NICHT INSTALLIERBAR");

            if mentions_skipped_tools {
                violations.push(AuditEvidenceViolation {
                    file_path: relative_path.clone(),
                    line_num,
                    verdict_text: line.trim().to_string(),
                    reason: "Verdict claims GO/APPROVED, but report explicitly notes tools were skipped/not installed".to_string(),
                });
                continue;
            }

            if evidence_markers.is_empty() {
                if is_post_cutoff {
                    violations.push(AuditEvidenceViolation {
                        file_path: relative_path.clone(),
                        line_num,
                        verdict_text: line.trim().to_string(),
                        reason: "Post-cutoff audit verdict claims GO/APPROVED without an <!-- EVIDENCE: <log_path> (exit=0) --> marker".to_string(),
                    });
                }
                continue;
            }

            for marker in &evidence_markers {
                let log_full_path = root.join(&marker.log_path);
                if !log_full_path.exists() {
                    violations.push(AuditEvidenceViolation {
                        file_path: relative_path.clone(),
                        line_num,
                        verdict_text: line.trim().to_string(),
                        reason: format!("Referenced evidence log '{}' does not exist", marker.log_path),
                    });
                } else {
                    let log_content = fs::read_to_string(&log_full_path).unwrap_or_default();
                    let exit_ok = log_content.contains("exit_code: 0")
                        || log_content.contains("exit=0")
                        || log_content.contains("Exit Code: 0")
                        || log_content.contains("SUCCESS")
                        || log_content.contains("test result: ok");

                    if !exit_ok {
                        violations.push(AuditEvidenceViolation {
                            file_path: relative_path.clone(),
                            line_num,
                            verdict_text: line.trim().to_string(),
                            reason: format!(
                                "Referenced evidence log '{}' does not show exit code 0 or successful test result",
                                marker.log_path
                            ),
                        });
                    }
                }
            }
        }
    }

    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_evidence_markers() {
        let text = r#"
# Audit Report
<!-- EVIDENCE: logs/audits/cov.log (exit=0) -->
VERDICT: GO/APPROVED
"#;
        let markers = parse_evidence_markers(text);
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].log_path, "logs/audits/cov.log");
        assert_eq!(markers[0].expected_exit, 0);
    }

    #[test]
    fn test_extract_audit_timestamp() {
        let text = "VERDICT: APPROVED (TS: 2026-09-16T14:30:00Z)";
        let ts = extract_audit_timestamp(text);
        assert!(ts.is_some());
        assert_eq!(ts.unwrap().to_rfc3339(), "2026-09-16T14:30:00+00:00");
    }
}
