// FILE-CONTEXT
// STAND: 2026-10-21 (W1-13)
// ZWECK: Stage 2 Tests for accelerated tombstone reapplication and truncation anti-resurrection guarantees (E-4 / INV-ERA-1).

#![allow(deprecated, reason = "testing deprecated open_or_fail_closed API")]

use contextra_store::erasure_ledger::*;
use tempfile::tempdir;

#[tokio::test]
async fn reapply_tombstones_accelerated_matches_naive() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([0x33; 32]);

    let (ledger_opt, _) = ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let e1 = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: [0x11; 32],
        erased_at_tx: 10,
        proof_id: [1u8; 16],
    };
    let e2 = LedgerEntry {
        ledger_seq: 2,
        kind: LedgerEntryKind::Key,
        keyed_hash: [0x22; 32],
        erased_at_tx: 20,
        proof_id: [2u8; 16],
    };

    ledger.append(&[e1.clone(), e2.clone()]).await.unwrap();

    let valid_anchor = *ledger.last_chain_mac();
    let (reapplied, status) = ledger.reapply_tombstones_from_ledger(b"lsm", Some(valid_anchor));

    assert_eq!(reapplied.len(), 2);
    assert_eq!(
        status,
        AntiResurrection::Guaranteed {
            tombstones_reapplied: 2
        }
    );

    // Verify accelerated matching entries match naive full vector scan
    let naive_entries = ledger.entries().to_vec();
    assert_eq!(reapplied, naive_entries);
}

#[tokio::test]
async fn truncation_without_anchor_reports_not_guaranteed() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([0x88; 32]);

    let (ledger_opt, _) = ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let e1 = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: [0x99; 32],
        erased_at_tx: 100,
        proof_id: [5u8; 16],
    };
    ledger.append(&[e1]).await.unwrap();

    // 1. None anchor
    let (entries_none, status_none) = ledger.reapply_tombstones_from_ledger(b"lsm", None);
    assert_eq!(entries_none.len(), 1);
    match status_none {
        AntiResurrection::NotGuaranteed { reason } => {
            assert!(reason.contains("Missing or unverified"));
        }
        _ => panic!("Expected NotGuaranteed, got {:?}", status_none),
    }

    // 2. Mismatched anchor
    let wrong_anchor = [0xFFu8; 32];
    let (entries_mismatch, status_mismatch) =
        ledger.reapply_tombstones_from_ledger(b"lsm", Some(wrong_anchor));
    assert_eq!(entries_mismatch.len(), 1);
    match status_mismatch {
        AntiResurrection::NotGuaranteed { reason } => {
            assert!(reason.contains("Missing or unverified"));
        }
        _ => panic!("Expected NotGuaranteed, got {:?}", status_mismatch),
    }
}
