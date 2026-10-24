use chrono::{TimeZone, Utc};
use std::path::PathBuf;

#[allow(dead_code)]
pub fn find_root_dir() -> PathBuf {
    PathBuf::from(".")
}

#[path = "../src/veto_deadline_gate.rs"]
mod veto_deadline_gate;

use veto_deadline_gate::{
    check_veto_deadlines_from_content, parse_veto_deadlines, VetoDeadlineEntry,
};

const TEST_VETOES_FIXTURE: &str = r#"# Contextra — Feature-Veto-Register (VETOES.md)

## VETO-F02

feature_id: F-02
status: conditionally_accepted
conditional_review_due: 2026-10-07
adr_ref: DECISIONS.md#adr-077
keywords: ["partial hnsw rebuild", "nucleation", "rebuild_region", "F-02"]
reason: >
  Kein partielles HNSW-Rewiring oder Teilgraph-Rebuilding (F-02) durchführen wegen Recall-Kollaps und RwLock-Contention; zulässig ist ausschließlich reines Tombstone-Pruning.

## VETO-F10

feature_id: F-10
status: permanent_rejected
keywords: ["cross-tenant", "osmotic knowledge exchange", "tenant knowledge sharing", "F-10"]
reason: >
  Keine mandantenübergreifenden Datenflüsse, Knowledge-Sharing oder Cross-Tenant-Aggregationen (F-10) herstellen; Mandantenisolation (TenantId) ist absolut zur Wahrung von DSGVO-Löschgarantien und KV-Cache-Sicherheit.

## VETO-OP03

feature_id: OP-03
status: conditionally_accepted
conditional_review_due: 2026-10-07
adr_ref: DECISIONS.md#adr-077
keywords: ["voice assistant", "speech-to-text", "realtime-audio", "jarvis", "OP-03"]
reason: >
  Keine Realtime-Audio-, Speech-to-Text-, Voice- oder Jarvis-Assistenten-Funktionen (OP-03) integrieren, da Audio-Streaming nicht zum bi-temporalen Speichersubstrat gehört.
"#;

#[test]
fn test_parse_veto_deadlines() {
    let entries = parse_veto_deadlines(TEST_VETOES_FIXTURE).unwrap();
    assert_eq!(entries.len(), 3);

    let f02 = entries.iter().find(|e| e.veto_id == "VETO-F02").unwrap();
    assert_eq!(f02.feature_id, "F-02");
    assert_eq!(f02.status, "conditionally_accepted");
    assert_eq!(f02.conditional_review_due.as_deref(), Some("2026-10-07"));
    assert_eq!(f02.adr_ref.as_deref(), Some("DECISIONS.md#adr-077"));

    let f10 = entries.iter().find(|e| e.veto_id == "VETO-F10").unwrap();
    assert_eq!(f10.feature_id, "F-10");
    assert_eq!(f10.status, "permanent_rejected");
    assert_eq!(f10.conditional_review_due, None);
    assert_eq!(f10.adr_ref, None);

    let op03 = entries.iter().find(|e| e.veto_id == "VETO-OP03").unwrap();
    assert_eq!(op03.feature_id, "OP-03");
    assert_eq!(op03.status, "conditionally_accepted");
    assert_eq!(op03.conditional_review_due.as_deref(), Some("2026-10-07"));
    assert_eq!(op03.adr_ref.as_deref(), Some("DECISIONS.md#adr-077"));
}

#[test]
fn test_check_veto_deadlines_expired_returns_err() {
    // Current date: 2026-10-08 (deadline 2026-10-07 was yesterday -> expired)
    let now = Utc.with_ymd_and_hms(2026, 10, 8, 12, 0, 0).unwrap();
    let res = check_veto_deadlines_from_content(TEST_VETOES_FIXTURE, now);

    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.contains("VETO-REVIEW-FRIST ÜBERSCHRITTEN"));
    assert!(err.contains("VETO-F02"));
    assert!(err.contains("VETO-OP03"));
}

#[test]
fn test_check_veto_deadlines_warning_window() {
    // Current date: 2026-09-27 (deadline 2026-10-07 is 10 days away -> within 14 days warning window)
    let now = Utc.with_ymd_and_hms(2026, 9, 27, 12, 0, 0).unwrap();
    let res = check_veto_deadlines_from_content(TEST_VETOES_FIXTURE, now);

    assert!(res.is_ok());
    let warnings = res.unwrap();
    assert_eq!(warnings.len(), 2);
    let ids: Vec<String> = warnings.into_iter().map(|w| w.veto_id).collect();
    assert!(ids.contains(&"VETO-F02".to_string()));
    assert!(ids.contains(&"VETO-OP03".to_string()));
}

#[test]
fn test_check_veto_deadlines_far_future_returns_empty_ok() {
    // Current date: 2026-08-01 (deadline 2026-10-07 is 67 days away -> > 14 days)
    let now = Utc.with_ymd_and_hms(2026, 8, 1, 12, 0, 0).unwrap();
    let res = check_veto_deadlines_from_content(TEST_VETOES_FIXTURE, now);

    assert!(res.is_ok());
    let warnings = res.unwrap();
    assert!(warnings.is_empty());
}
