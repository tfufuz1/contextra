// FILE-CONTEXT
// STAND: 2026-10-11 (T-2026-9103)
// ZWECK: Tests für Etappe 5 — LedgerCoverage Injektionstest Zeile 27.

use contextra_store::erasure_ledger::*;
use tempfile::tempdir;

#[tokio::test]
async fn test_ledger_coverage_check_detects_gap() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([12u8; 32]);

    let (ledger_opt, _) = ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let scope = b"tenant_x";
    let key1 = b"doc:1";
    let key2 = b"doc:2";

    // Add ledger entry only for key1
    let entry1 = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: compute_keyed_hash(&key, scope, key1),
        erased_at_tx: 100,
        proof_id: [1u8; 16],
    };
    ledger.append(&[entry1]).await.unwrap();

    // Tombstones exist for key1 and key2
    let tombstoned_keys: &[(&[u8], &[u8], u64)] = &[
        (scope, key1, 50),
        (scope, key2, 50), // Missing in ledger!
    ];

    let result = LedgerCoverage::check(tombstoned_keys, &ledger);
    assert!(result.is_err());
    match result {
        Err(LedgerGap::MissingLedgerEntry { erased_tx, .. }) => {
            assert_eq!(erased_tx, 50);
        }
        _ => panic!("Expected MissingLedgerEntry error"),
    }

    // Now add ledger entry for key2
    let entry2 = LedgerEntry {
        ledger_seq: 2,
        kind: LedgerEntryKind::Key,
        keyed_hash: compute_keyed_hash(&key, scope, key2),
        erased_at_tx: 100,
        proof_id: [2u8; 16],
    };
    ledger.append(&[entry2]).await.unwrap();

    // Check again - should pass
    let result_ok = LedgerCoverage::check(tombstoned_keys, &ledger);
    assert!(result_ok.is_ok());
}
