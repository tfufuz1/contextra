//! Conformance integration test for FileKeyring port (PO-06, D-85).
//! Tests reference implementation and ensures 100% detection of mutants.

#[path = "../crates/contextra-testkit/src/conformance/file_keyring.rs"]
mod file_keyring;

#[tokio::test]
async fn test_file_keyring_conformance_suite() {
    assert!(file_keyring::run_reference_test().await.is_ok());
    assert!(file_keyring::run_mutant_tests().await.is_ok());
}
