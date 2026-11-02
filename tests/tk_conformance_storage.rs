//! Conformance integration test for StorageEngine port (PO-02).
//! Tests reference implementation and ensures 100% detection of mutants.

#[path = "../crates/contextra-testkit/src/conformance/storage.rs"]
mod storage;

#[tokio::test]
async fn test_storage_conformance_suite() {
    assert!(storage::run_reference_test().await.is_ok());
    assert!(storage::run_mutant_tests().await.is_ok());
}
