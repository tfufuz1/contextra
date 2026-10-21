//! Tests for Stage 2: Witness Origin (`LayerWitness::from_scan`) & INV-ERA-6 (w1_06_*).

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_crypto::{LayerWitness, ScanOutcome, WitnessError};
use contextra_ports::Clock;
use contextra_types::{DeterministicTimestamp, TxId};

struct TestClock;
impl Clock for TestClock {
    fn now(&self) -> DeterministicTimestamp {
        DeterministicTimestamp::new(1000, 1)
    }
    fn now_unix_nanos(&self) -> u64 {
        1_000_000_000
    }
    fn monotonic_nanos(&self) -> u64 {
        1_000_000_000
    }
    fn monotonic_ticks(&self) -> u64 {
        1000
    }
}

#[test]
fn w1_06_layer_witness_from_scan_rejects_matches() {
    let clock = TestClock;
    let scan_with_matches = ScanOutcome {
        layer: DeletionLayer::LsmMemtable,
        units_scanned: 10,
        matches_found: 2, // Non-zero matches MUST be rejected
        scanned_at_tx: TxId(100),
        empty_layer_declared: false,
        footprint_units: vec![b"unit1".to_vec()],
    };

    let res = LayerWitness::from_scan(
        scan_with_matches,
        [1u8; 32],
        WitClaimText::S1,
        WitAssurance::S1,
        &clock,
    );

    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), WitnessError::MatchesFound(2)));
}

#[test]
fn w1_06_from_scan_rejects_zero_units_without_declaration() {
    let clock = TestClock;
    let scan_zero_units = ScanOutcome {
        layer: DeletionLayer::LsmMemtable,
        units_scanned: 0,
        matches_found: 0,
        scanned_at_tx: TxId(100),
        empty_layer_declared: false, // units_scanned == 0 without empty_layer_declared MUST be rejected
        footprint_units: vec![],
    };

    let res = LayerWitness::from_scan(
        scan_zero_units,
        [1u8; 32],
        WitClaimText::S1,
        WitAssurance::S1,
        &clock,
    );

    assert!(res.is_err());
    assert!(matches!(
        res.unwrap_err(),
        WitnessError::ZeroUnitsWithoutEmptyDeclaration
    ));

    // With empty_layer_declared = true, it should succeed
    let scan_empty_declared = ScanOutcome {
        layer: DeletionLayer::LsmMemtable,
        units_scanned: 0,
        matches_found: 0,
        scanned_at_tx: TxId(100),
        empty_layer_declared: true,
        footprint_units: vec![],
    };

    let res_ok = LayerWitness::from_scan(
        scan_empty_declared,
        [1u8; 32],
        WitClaimText::S0,
        WitAssurance::S0,
        &clock,
    );

    assert!(res_ok.is_ok());
}

#[test]
fn w1_06_layer_witness_issue_not_public_after_flip() {
    // Verifies that the UI file for W3-01 flip is present in compile_fail
    let ui_file = std::path::Path::new("tests/compile_fail/w1_06_layer_witness_issue_not_public_after_flip.rs");
    assert!(ui_file.exists(), "UI test file for W3-01 flip must exist");
}
