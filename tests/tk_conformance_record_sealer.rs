//! Conformance integration test for RecordSealer port (D-53, D-79, I-09).
//! Tests reference implementation and ensures 100% detection of mutants.

#[path = "../crates/contextra-testkit/src/conformance/record_sealer.rs"]
mod record_sealer;

#[tokio::test]
async fn test_record_sealer_conformance_suite() {
    assert!(record_sealer::run_reference_test().await.is_ok());
    assert!(record_sealer::run_mutant_tests().await.is_ok());
}
