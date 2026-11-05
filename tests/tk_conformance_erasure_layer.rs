//! Conformance integration test for ErasureLayer port (SPEC §8.2, §11, PO-05, P1, P12, D-69, HR-10).
//! Tests reference implementation and ensures 100% detection of mutants.
/// T-ID: T-ISO-1
use contextra_testkit::conformance::erasure_layer;

/// T-ID: T-ISO-1
#[tokio::test]
async fn test_erasure_layer_conformance_suite() {
    assert!(
        erasure_layer::run_reference_test().await.is_ok(),
        "ReferenceErasureLayer must pass conformance suite"
    );
    assert!(
        erasure_layer::run_mutant_tests().await.is_ok(),
        "All defective ErasureLayer mutants must be caught"
    );
}
