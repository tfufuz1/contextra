// FILE-CONTEXT
// STAND: 2026-10-11 (T-2026-9103)
// ZWECK: Fail-Closed Opening Matrix-Tests für ErasureLedger (T10 Rollback-Erkennung).

use contextra_store::erasure_ledger::*;
use tempfile::tempdir;

#[tokio::test]
async fn test_fresh_database_creates_fresh_ledger() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([1u8; 32]);

    let (ledger, status) = ErasureLedger::open_or_fail_closed(dir.path(), key, false, None)
        .await
        .unwrap();

    assert_eq!(status, LedgerOpenStatus::Fresh);
    assert!(ledger.is_some());
}

#[tokio::test]
async fn test_missing_ledger_with_existing_db_returns_rollback_suspected() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([2u8; 32]);

    // db_has_data = true, but directory has no erasure.ledger file
    let (ledger, status) = ErasureLedger::open_or_fail_closed(dir.path(), key, true, None)
        .await
        .unwrap();

    assert!(ledger.is_none());
    match status {
        LedgerOpenStatus::RollbackSuspected { reason } => {
            assert!(reason.contains("missing"));
        }
        _ => panic!("Expected RollbackSuspected, got {:?}", status),
    }
}

#[tokio::test]
async fn test_head_mac_mismatch_returns_rollback_suspected() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([3u8; 32]);

    let (ledger_opt, _) = ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let entry = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: [0xAA; 32],
        erased_at_tx: 10,
        proof_id: [1u8; 16],
    };
    ledger.append(&[entry]).await.unwrap();

    // Now tamper with ledger.head marker
    let head_path = dir.path().join("ledger.head");
    let mut marker_bytes = tokio::fs::read(&head_path).await.unwrap();
    marker_bytes[40] ^= 0xFF; // Mismatch head MAC
    tokio::fs::write(&head_path, &marker_bytes).await.unwrap();

    // Reopen
    let (reopened, status) = ErasureLedger::open_or_fail_closed(dir.path(), key, true, None)
        .await
        .unwrap();

    assert!(reopened.is_none());
    match status {
        LedgerOpenStatus::RollbackSuspected { reason } => {
            assert!(reason.contains("MAC"));
        }
        _ => panic!("Expected RollbackSuspected, got {:?}", status),
    }
}

#[tokio::test]
async fn test_replaced_older_ledger_fails_closed() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([4u8; 32]);

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
    ledger.append(&[e1]).await.unwrap();

    // Save copy of erasure.ledger after entry 1
    let ledger_path = dir.path().join("erasure.ledger");
    let older_ledger_bytes = tokio::fs::read(&ledger_path).await.unwrap();

    let e2 = LedgerEntry {
        ledger_seq: 2,
        kind: LedgerEntryKind::Key,
        keyed_hash: [0x22; 32],
        erased_at_tx: 20,
        proof_id: [2u8; 16],
    };
    ledger.append(&[e2]).await.unwrap();

    let head_after_e2 = ledger.last_chain_mac().clone();

    // Revert erasure.ledger to older_ledger_bytes (simulating rollback of ledger file)
    tokio::fs::write(&ledger_path, &older_ledger_bytes)
        .await
        .unwrap();

    // Reopen expecting head_after_e2
    let (reopened, status) =
        ErasureLedger::open_or_fail_closed(dir.path(), key, true, Some(head_after_e2))
            .await
            .unwrap();

    assert!(reopened.is_none());
    match status {
        LedgerOpenStatus::RollbackSuspected { reason } => {
            assert!(reason.contains("head MAC mismatch") || reason.contains("does not match"));
        }
        _ => panic!("Expected RollbackSuspected, got {:?}", status),
    }
}
