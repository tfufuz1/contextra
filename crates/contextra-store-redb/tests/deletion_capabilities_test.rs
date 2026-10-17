//! Deletion capability descriptor tests for `RedbStorageEngine`.

use tempfile::tempdir;

use contextra_store_redb::{DeletionStage, RedbStorageEngine};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_redb_deletion_capabilities_descriptor() -> TestResult {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.redb");

    let store = RedbStorageEngine::open(&db_path)?;
    let caps = store.deletion_capabilities();

    // Verify honest deletion semantics for redb backend
    assert!(
        !caps.physical_purge_supported,
        "Redb must report physical_purge_supported = false"
    );
    assert_eq!(
        caps.max_deletion_stage,
        DeletionStage::S1,
        "Redb max deletion stage must be S1 (Logical free-listing)"
    );
    assert!(
        caps.explanation.contains("free-list"),
        "Capabilities explanation must describe free-list page retention"
    );

    Ok(())
}
