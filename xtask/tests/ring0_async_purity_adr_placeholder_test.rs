use std::fs;
use tempfile::NamedTempFile;
use xtask::check_ring0_async_purity::load_allowlist;

#[test]
fn test_load_allowlist_rejects_adr_todo_placeholder() {
    let file = NamedTempFile::new().unwrap();
    let content = r#"
[[exceptions]]
crate = "contextra-graph"
dependency = "tokio"
reason = "Async compaction"
adr_reference = "ADR-TODO"
"#;
    fs::write(file.path(), content).unwrap();

    let result = load_allowlist(file.path());
    assert!(result.is_err(), "Expected Err for ADR-TODO placeholder");
    let err = result.unwrap_err();
    assert!(
        err.contains("contextra-graph"),
        "Error message should mention crate name: {}",
        err
    );
    assert!(
        err.contains("tokio"),
        "Error message should mention dependency: {}",
        err
    );
    assert!(
        err.contains("ADR-TODO"),
        "Error message should mention invalid adr_reference: {}",
        err
    );
}

#[test]
fn test_load_allowlist_accepts_valid_adr_reference() {
    let file = NamedTempFile::new().unwrap();
    let content = r#"
[[exceptions]]
crate = "contextra-graph"
dependency = "tokio"
reason = "Async compaction"
adr_reference = "ADR-0042"
"#;
    fs::write(file.path(), content).unwrap();

    let result = load_allowlist(file.path());
    assert!(
        result.is_ok(),
        "Expected Ok for valid ADR reference, got: {:?}",
        result
    );
    let allowlist = result.unwrap();
    assert_eq!(allowlist.len(), 1);
    assert_eq!(allowlist[0].crate_name, "contextra-graph");
    assert_eq!(allowlist[0].dependency, "tokio");
    assert_eq!(allowlist[0].adr_reference, "ADR-0042");
}
