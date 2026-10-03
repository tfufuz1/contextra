// Integration tests for R-10: Veto deadline gate scoped checking & fail-closed assignment logic.

use chrono::{TimeZone, Utc};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[allow(dead_code)]
pub fn find_root_dir() -> PathBuf {
    PathBuf::from(".")
}

#[path = "../src/veto_deadline_gate.rs"]
mod veto_deadline_gate;

use veto_deadline_gate::check_veto_deadlines_from_content_scoped;

#[test]
fn test_veto_deadline_in_5_days_warning_exit0() {
    let content = r#"# Contextra Vetoes
## VETO-F02
feature_id: F-02
status: conditionally_accepted
conditional_review_due: 2026-10-08
adr_ref: docs/decisions/ADR-077.md
affected_paths: ["crates/contextra-vector/"]
keywords: ["partial hnsw rebuild"]
reason: >
  Test
"#;
    // Today is 2026-10-03 -> due in 5 days
    let now = Utc.with_ymd_and_hms(2026, 10, 3, 12, 0, 0).unwrap();
    let changed_files = vec!["crates/contextra-vector/src/hnsw.rs".to_string()];

    let res = check_veto_deadlines_from_content_scoped(content, now, Some(&changed_files));
    assert!(res.is_ok(), "Frist in 5 Tagen muss Exit 0 (Ok) liefern");
    let warnings = res.unwrap();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].veto_id, "VETO-F02");
}

#[test]
fn test_expired_veto_feature_distant_pr_warning_exit0() {
    let content = r#"# Contextra Vetoes
## VETO-F02
feature_id: F-02
status: conditionally_accepted
conditional_review_due: 2026-09-01
adr_ref: docs/decisions/ADR-077.md
affected_paths: ["crates/contextra-vector/"]
keywords: ["partial hnsw rebuild"]
reason: >
  Test
"#;
    // Today is 2026-10-03 -> overdue (due 2026-09-01)
    let now = Utc.with_ymd_and_hms(2026, 10, 3, 12, 0, 0).unwrap();
    let changed_files = vec!["crates/contextra-store/src/lib.rs".to_string()];

    let res = check_veto_deadlines_from_content_scoped(content, now, Some(&changed_files));
    assert!(
        res.is_ok(),
        "Überfälliges Veto ohne PR-Bezug muss Exit 0 (Ok) liefern"
    );
    let warnings = res.unwrap();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].veto_id, "VETO-F02");
}

#[test]
fn test_expired_veto_feature_touching_pr_error_exit_nonzero() {
    let content = r#"# Contextra Vetoes
## VETO-F02
feature_id: F-02
status: conditionally_accepted
conditional_review_due: 2026-09-01
adr_ref: docs/decisions/ADR-077.md
affected_paths: ["crates/contextra-vector/"]
keywords: ["partial hnsw rebuild"]
reason: >
  Test
"#;
    // Today is 2026-10-03 -> overdue
    let now = Utc.with_ymd_and_hms(2026, 10, 3, 12, 0, 0).unwrap();
    let changed_files = vec!["crates/contextra-vector/src/hnsw.rs".to_string()];

    let res = check_veto_deadlines_from_content_scoped(content, now, Some(&changed_files));
    assert!(
        res.is_err(),
        "Überfälliges Veto mit PR-Bezug muss fehlschlagen (Err)"
    );
    let err = res.unwrap_err();
    assert!(
        err.contains("VETO-REVIEW-FRIST ÜBERSCHRITTEN") || err.contains("VETO-FRIST ÜBERSCHRITTEN")
    );
    assert!(err.contains("VETO-F02") || err.contains("F-02"));
}

#[test]
fn test_expired_veto_missing_assignment_error_exit_nonzero() {
    let content = r#"# Contextra Vetoes
## VETO-F02
feature_id: F-02
status: conditionally_accepted
conditional_review_due: 2026-09-01
adr_ref: docs/decisions/ADR-077.md
reason: >
  Test missing assignment
"#;
    // Today is 2026-10-03 -> overdue, but neither affected_paths nor keywords exist
    let now = Utc.with_ymd_and_hms(2026, 10, 3, 12, 0, 0).unwrap();
    let changed_files = vec!["crates/contextra-store/src/lib.rs".to_string()];

    let res = check_veto_deadlines_from_content_scoped(content, now, Some(&changed_files));
    assert!(
        res.is_err(),
        "Überfälliges Veto mit fehlender Zuordnung muss fail-closed fehlschlagen (Err)"
    );
    let err = res.unwrap_err();
    assert!(
        err.contains("Zuordnung fehlt")
            || err.contains("FAIL-CLOSED")
            || err.contains("VETO-FRIST ÜBERSCHRITTEN")
    );
}

#[test]
fn test_veto_date_extension_without_base_adr_error_exit_nonzero() {
    let temp = tempdir().unwrap();
    let vetoes_file = temp.path().join("VETOES.md");
    fs::write(
        &vetoes_file,
        r#"
## VETO-F02
feature_id: F-02
status: conditionally_accepted
conditional_review_due: 2026-12-01
adr_ref: docs/decisions/ADR-099-new.md
affected_paths: ["crates/contextra-vector/"]
reason: >
  Extension
"#,
    )
    .unwrap();

    let base_vetoes = r#"
## VETO-F02
feature_id: F-02
status: conditionally_accepted
conditional_review_due: 2026-09-01
adr_ref: docs/decisions/ADR-077.md
affected_paths: ["crates/contextra-vector/"]
reason: >
  Original
"#;

    // ADR-099-new.md does NOT exist on base branch
    let check_adr_on_base = |_adr: &str| -> bool { false };

    let result = xtask::check_vetoes::check_vetoes_with_root_and_opts(
        temp.path(),
        "2026-10-03",
        Some(&[]),
        Some(base_vetoes),
        Some(&check_adr_on_base),
    );

    assert!(result.is_err());
}
