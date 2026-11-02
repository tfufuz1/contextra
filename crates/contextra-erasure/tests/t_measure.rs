//! Tests für Messung und Typestate Scanner (T-RAW-1..6).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::measure::{EvidenceKind, MeasurementCoordinator, UnarmedScanner};

#[test]
fn t_raw_2_positive_control_and_witness_generation() {
    let _coordinator = MeasurementCoordinator::new();
    let unarmed = UnarmedScanner::new();

    let evidence = unarmed
        .run_positive_control(b"known-positive-control-pattern")
        .expect("positive control must be observed");
    assert_eq!(evidence.kind(), EvidenceKind::Observed);

    let armed = unarmed.arm(evidence);

    let witness = armed
        .finish(0)
        .expect("finish with 0 matches produces witness");
    assert_eq!(witness.witness_hash().len(), 32);

    let armed_fail = UnarmedScanner::new()
        .run_positive_control(b"pattern")
        .map(|ev| UnarmedScanner::new().arm(ev))
        .expect("arm failed");
    let residue_err = armed_fail.finish(3);
    assert!(
        residue_err.is_err(),
        "Residues must never produce a witness"
    );
}
