//! Tests für Report, Ledger und Restore-Guard (T-REPORT-1, T-PROOF-1, T-RESTORE-1..3).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::ledger::InMemoryLedger;
use contextra_erasure::report::{
    build_report, canonicalize_report, SignedReport,
};
use contextra_erasure::restore::{GuardOutcome, RestoreGuard};
use contextra_types::ids::ErasureId;

#[test]
fn t_report_1_canonical_json_and_signature() {
    let erasure_id = ErasureId::new([0x42; 16]);
    let report = build_report(erasure_id, 1000, 2000, Vec::new(), 0, 0, true);

    let canonical = canonicalize_report(&report).expect("canonicalization failed");
    assert!(!canonical.is_empty());

    let signed = SignedReport::sign_stub(canonical);
    assert!(!signed.signature.is_empty());
    assert!(!signed.pub_fingerprint.is_empty());
}

#[test]
fn t_restore_1_ledger_and_guard() {
    let mut ledger = InMemoryLedger::new([0x55; 32]);
    ledger
        .append([0x11; 32], vec![1, 2, 3], 50, 1000)
        .expect("append ledger failed");

    assert!(ledger.verify_chain(), "Ledger chain must verify");
    assert_eq!(ledger.entries().len(), 1);

    let guard = RestoreGuard::new(&ledger);
    match guard.check(40) {
        GuardOutcome::TimeRegression { store_seq, ledger_floor } => {
            assert_eq!(store_seq, 40);
            assert_eq!(ledger_floor, 50);
        }
        _ => panic!("Expected time regression for seq < ledger_floor"),
    }

    assert_eq!(guard.check(60), GuardOutcome::Clean);
    assert_eq!(guard.next_safe_seq(40), 51);
    assert_eq!(guard.next_safe_seq(60), 61);
}
