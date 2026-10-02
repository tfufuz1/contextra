// Integration tests for check_vetoes gate with scoped blocking and base branch extension verification.

use std::fs;
use tempfile::tempdir;

#[test]
fn test_vetoes_missing_file_fail_closed() {
    let temp = tempdir().unwrap();
    // VETOES.md missing
    let result = xtask::check_vetoes::check_vetoes_with_root_and_opts(
        temp.path(),
        "2026-10-01",
        Some(&[]),
        None,
        None,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("FAIL-CLOSED"));
    assert!(err.contains("VETOES.md"));
}

#[test]
fn test_expired_veto_feature_touched_red() {
    let temp = tempdir().unwrap();
    let vetoes_file = temp.path().join("VETOES.md");
    fs::write(
        &vetoes_file,
        r#"
## VETO-F02

feature_id: F-02
status: conditionally_accepted
conditional_review_due: 2026-09-01
adr_ref: docs/decisions/ADR-077.md
affected_paths: ["crates/contextra-vector/"]
keywords: ["partial hnsw rebuild"]
reason: >
  Test expired VETO
"#,
    )
    .unwrap();

    let changed_files = vec!["crates/contextra-vector/src/hnsw.rs".to_string()];
    let result = xtask::check_vetoes::check_vetoes_with_root_and_opts(
        temp.path(),
        "2026-10-01",
        Some(&changed_files),
        None,
        None,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("VETO-FRIST ÜBERSCHRITTEN"));
    assert!(err.contains("F-02"));
}

#[test]
fn test_expired_veto_feature_not_touched_green_with_warning() {
    let temp = tempdir().unwrap();
    let vetoes_file = temp.path().join("VETOES.md");
    fs::write(
        &vetoes_file,
        r#"
## VETO-F02

feature_id: F-02
status: conditionally_accepted
conditional_review_due: 2026-09-01
adr_ref: docs/decisions/ADR-077.md
affected_paths: ["crates/contextra-vector/"]
keywords: ["partial hnsw rebuild"]
reason: >
  Test expired VETO
"#,
    )
    .unwrap();

    let changed_files = vec!["crates/contextra-store/src/lib.rs".to_string()];
    let result = xtask::check_vetoes::check_vetoes_with_root_and_opts(
        temp.path(),
        "2026-10-01",
        Some(&changed_files),
        None,
        None,
    );
    // PR should NOT be blocked if the expired feature is untouched
    assert!(result.is_ok());
}

#[test]
fn test_veto_date_extension_in_same_pr_red() {
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
  Extended VETO date in PR
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
  Old VETO date
"#;

    // ADR-099-new.md was added in the SAME diff, so check_adr_exists_on_base returns false
    let check_adr_on_base = |_adr: &str| -> bool { false };

    let result = xtask::check_vetoes::check_vetoes_with_root_and_opts(
        temp.path(),
        "2026-10-01",
        Some(&[]),
        Some(base_vetoes),
        Some(&check_adr_on_base),
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("UNZULÄSSIGE VETO-VERLÄNGERUNG"));
    assert!(err.contains("ADR 'docs/decisions/ADR-099-new.md' existiert NICHT auf dem Basis-Branch"));
}
