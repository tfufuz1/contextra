//! Conformance integration test for TextIndex port (PO-02).
//! Tests reference implementation and ensures 100% detection of mutants.

#[path = "../crates/contextra-testkit/src/conformance/text_index.rs"]
mod text_index;

#[tokio::test]
async fn test_text_index_conformance_suite() {
    assert!(text_index::run_reference_test().await.is_ok());
    assert!(text_index::run_mutant_tests().await.is_ok());
}
