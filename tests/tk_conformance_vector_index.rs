//! Conformance integration test for VectorIndex port (PO-02).
//! Tests reference implementation and ensures 100% detection of mutants.

#[path = "../crates/contextra-testkit/src/conformance/vector_index.rs"]
mod vector_index;

#[tokio::test]
async fn test_vector_index_conformance_suite() {
    assert!(vector_index::run_reference_test().await.is_ok());
    assert!(vector_index::run_mutant_tests().await.is_ok());
}
