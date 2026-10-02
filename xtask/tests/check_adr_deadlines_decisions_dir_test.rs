// Integration tests for check_adr_deadlines gate with docs/decisions/ directory.

use std::fs;
use tempfile::tempdir;

#[test]
fn test_adr_deadlines_missing_dir_fail_closed() {
    let temp = tempdir().unwrap();
    // Do not create docs/decisions
    let result = xtask::check_adr_deadlines::check_adr_deadlines_with_root_and_date(
        temp.path(),
        "2026-10-01",
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("FAIL-CLOSED"));
    assert!(err.contains("existiert nicht"));
}

#[test]
fn test_adr_deadlines_empty_dir_fail_closed() {
    let temp = tempdir().unwrap();
    let decisions_dir = temp.path().join("docs").join("decisions");
    fs::create_dir_all(&decisions_dir).unwrap();

    let result = xtask::check_adr_deadlines::check_adr_deadlines_with_root_and_date(
        temp.path(),
        "2026-10-01",
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("FAIL-CLOSED"));
    assert!(err.contains("Keine Markdown-Dateien"));
}

#[test]
fn test_adr_without_deadline_passes() {
    let temp = tempdir().unwrap();
    let decisions_dir = temp.path().join("docs").join("decisions");
    fs::create_dir_all(&decisions_dir).unwrap();

    let adr_file = decisions_dir.join("ADR-001-test.md");
    fs::write(
        &adr_file,
        "# ADR-001: Standard Architecture Decision\n\n* **Status:** Akzeptiert\n",
    )
    .unwrap();

    let result = xtask::check_adr_deadlines::check_adr_deadlines_with_root_and_date(
        temp.path(),
        "2026-10-01",
    );
    assert!(result.is_ok());
}

#[test]
fn test_adr_deadline_in_10_days_issues_warning() {
    let temp = tempdir().unwrap();
    let decisions_dir = temp.path().join("docs").join("decisions");
    fs::create_dir_all(&decisions_dir).unwrap();

    let target_dir = temp.path().join("crates/contextra-legacy");
    fs::create_dir_all(&target_dir).unwrap();

    let adr_file = decisions_dir.join("ADR-088-legacy-crate.md");
    fs::write(
        &adr_file,
        "# ADR-088: Legacy Crate Removal\n\n* **Status:** Deprecated\n* **Removal Deadline:** 2026-10-15\n* **Target Path:** crates/contextra-legacy\n",
    )
    .unwrap();

    let entries =
        xtask::check_adr_deadlines::parse_adr_deadlines(&fs::read_to_string(&adr_file).unwrap())
            .unwrap();
    let check_res =
        xtask::check_adr_deadlines::check_adr_deadlines_at(&entries, "2026-10-05", temp.path());

    assert_eq!(check_res.errors.len(), 0);
    assert_eq!(check_res.warnings.len(), 1);
    assert!(check_res.warnings[0].contains("10 Tag(en)"));
}

#[test]
fn test_adr_deadline_expired_target_exists_fails() {
    let temp = tempdir().unwrap();
    let decisions_dir = temp.path().join("docs").join("decisions");
    fs::create_dir_all(&decisions_dir).unwrap();

    let target_dir = temp.path().join("crates/contextra-legacy");
    fs::create_dir_all(&target_dir).unwrap();

    let adr_file = decisions_dir.join("ADR-088-legacy-crate.md");
    fs::write(
        &adr_file,
        "# ADR-088: Legacy Crate Removal\n\n* **Status:** Deprecated\n* **Removal Deadline:** 2026-10-01\n* **Target Path:** crates/contextra-legacy\n",
    )
    .unwrap();

    let entries =
        xtask::check_adr_deadlines::parse_adr_deadlines(&fs::read_to_string(&adr_file).unwrap())
            .unwrap();
    let check_res =
        xtask::check_adr_deadlines::check_adr_deadlines_at(&entries, "2026-10-05", temp.path());

    assert_eq!(check_res.errors.len(), 1);
    assert!(check_res.errors[0].contains("ADR-FRIST ÜBERSCHRITTEN"));
}
