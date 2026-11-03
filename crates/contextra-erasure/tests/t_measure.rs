//! Tests für Messung und Typestate Scanner (T-RAW-1..6, ER-25, D-72).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::measure::positive_control::{run_positive_control, verify_scanner_integrity};
use contextra_erasure::measure::{
    arm_scanner, finish_scan, run_scanner_flow, EvidenceKind, MeasurementCoordinator,
    UnarmedScanner,
};
use contextra_types::erasure::{LayerId, MeasureClass, Scanner, Unarmed};
use std::time::Duration;

#[test]
fn t_raw_2_positive_control_and_witness_generation() {
    let _coordinator = MeasurementCoordinator::new();
    let unarmed = UnarmedScanner::new();

    // 1. Echte Positivkontrolle muss für valides Muster gelingen
    let evidence = unarmed
        .run_positive_control(b"known-positive-control-pattern")
        .expect("positive control must be observed");
    assert_eq!(evidence.kind(), EvidenceKind::Observed);

    // 2. Scanner armen
    let armed = unarmed.arm(evidence);

    // 3. Scan abschließen mit 0 Treffern -> Zeuge erzeugt
    let witness = armed
        .finish(0)
        .expect("finish with 0 matches produces witness");
    assert_eq!(witness.witness_hash().len(), 32);

    // 4. Scan mit Residuen (> 0 Treffer) darf NIE einen Zeugen erzeugen
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

#[test]
fn t_raw_2_positive_control_genuine_scan_and_mutants() {
    // Leeres Muster muss scheitern
    assert!(run_positive_control(b"").is_none());

    // Echtes Muster muss gelingen
    let pattern = b"contextra-canary-sequence-42";
    let ev = run_positive_control(pattern);
    assert!(ev.is_some(), "Echtes Muster muss gefunden werden");

    // Mutant: Verändertes Muster darf im Scratch-Puffer nicht matchen
    let mutated_detected = verify_scanner_integrity(pattern, |p| {
        let mut m = p.to_vec();
        if let Some(first) = m.first_mut() {
            *first ^= 0xff;
        }
        m
    });
    assert!(mutated_detected, "Mutiertes Muster muss abgewiesen werden");
}

#[test]
fn t_raw_2_typed_scanner_flow() {
    let layer = LayerId::new([0x01; 16]);
    let pattern = b"flow-pattern-test";

    let witness = run_scanner_flow(layer, MeasureClass::Mk1, 1, pattern)
        .expect("flow should produce witness with 0 matches");
    assert_eq!(witness.layer(), layer);
    assert_eq!(witness.class(), MeasureClass::Mk1);

    // Unbewaffneter Scanner kann nicht finishen ohne arm
    let unarmed = Scanner::<Unarmed>::new(layer, MeasureClass::Mk1, 1);
    let ev = run_positive_control(pattern).expect("positive control ok");
    let mut armed = arm_scanner(unarmed, ev);
    armed.record(2, 1024);

    let res = finish_scan(armed, 0, Duration::from_millis(5), [0xaa; 32]);
    assert!(res.is_ok());
}
