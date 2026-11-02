//! Conformance integration test for LedgerSink port (PO-04, §11.8, F.9).
//! Tests reference implementation and ensures 100% detection of mutants.

#[path = "../crates/contextra-testkit/src/conformance/ledger_sink.rs"]
mod ledger_sink;

#[tokio::test]
async fn test_ledger_sink_conformance_suite() {
    assert!(ledger_sink::run_reference_test().await.is_ok());
    assert!(ledger_sink::run_mutant_tests().await.is_ok());
}
