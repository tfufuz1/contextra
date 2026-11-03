//! Tests für Report, Ledger und Restore-Guard (T-REPORT-1, T-PROOF-1, T-RESTORE-1..3).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::ledger::InMemoryLedger;
use contextra_erasure::report::{canonicalize_json, ErasureReport, SignedReport};
use contextra_erasure::restore::RestoreGuard;
use contextra_ports::gate::{KeyBuf, Seq};
use contextra_ports::ledger::{LedgerEntry, LedgerSink, RangeSet, Timestamp};
use ed25519_dalek::SigningKey;

#[test]
fn t_report_1_canonical_json_and_signature() {
    let mut report = ErasureReport::new("ctx/report/v1", 1700000000000);
    report.set_field("erasure_id", "0123456789abcdef");
    report.set_field("status", "completed");

    let canonical = canonicalize_json(&report).expect("canonicalization failed");
    assert!(!canonical.contains('\n'));
    assert!(!canonical.contains(": "));

    let signing_key = SigningKey::from_bytes(&[0x42; 32]);
    let signed = SignedReport::sign(report, &signing_key).expect("signing failed");
    assert!(!signed.signature_base64().is_empty());
}

#[test]
fn t_restore_1_ledger_and_guard() {
    let ledger = InMemoryLedger::new();
    let entry = LedgerEntry::compute_chained(
        [0x11; 32],
        RangeSet::new(vec![(KeyBuf::new([1; 16]), KeyBuf::new([2; 16]))]),
        Seq::new(50),
        Timestamp::new(1000),
        [0u8; 32],
        [0x22; 32],
    );

    ledger.append(&entry).expect("append ledger failed");
    let all = ledger.read_all().expect("read_all failed");
    assert_eq!(all.len(), 1);

    let guard = RestoreGuard::new(Seq::new(50));
    assert!(
        guard.is_replay(Seq::new(40)),
        "Seq < Floor muss Replay sein"
    );
    assert!(
        !guard.is_replay(Seq::new(60)),
        "Seq > Floor ist legitime Neuanlage"
    );
}
