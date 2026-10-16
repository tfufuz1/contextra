// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: LedgerCoverage Injektionstest für Anti-Resurrection ErasureLedger CTRALEDG v1.

#![forbid(unsafe_code)]

use contextra_ports::SystemClock;
use contextra_store::erasure_ledger::*;
use tempfile::tempdir;

#[tokio::test]
async fn test_ledger_coverage_check_detects_gap() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([12u8; 32]);
    let clock = SystemClock::new();

    let (ledger_opt, _) = ErasureLedger::open_assured(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let scope = b"tenant_x";
    let key1 = b"doc:1";
    let key2 = b"doc:2";

    // Add ledger entry only for key1 with scope tenant_x
    ledger
        .append_purged(100, &clock, [1u8; 16], scope, &[key1])
        .await
        .unwrap();

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
    ledger
        .append_purged(100, &clock, [2u8; 16], scope, &[key2])
        .await
        .unwrap();

    // Check again - should pass
    let result_ok = LedgerCoverage::check(tombstoned_keys, &ledger);
    assert!(result_ok.is_ok());
}
