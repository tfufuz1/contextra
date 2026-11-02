//! Conformance integration test for ReadGate port (PO-07, PO-04, D-12, I-18, D-57).
//! Tests reference implementation and ensures 100% detection of mutants.

#[path = "../crates/contextra-testkit/src/conformance/read_gate.rs"]
mod read_gate;

#[test]
fn test_read_gate_conformance_suite() {
    assert!(read_gate::run_reference_test().is_ok());
    assert!(read_gate::run_mutant_tests().is_ok());
}
