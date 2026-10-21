// FILE-CONTEXT
// STAND: 2026-10-21 (W1-13)
// ZWECK: Stage 1 Tests for ErasureLedger contract hardening & privacy/idempotency.

#![allow(deprecated, reason = "testing deprecated open_or_fail_closed API")]

use contextra_store::erasure_ledger::*;
use tempfile::tempdir;

#[tokio::test]
async fn ledger_append_idempotent_same_keyed_hash() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([0x42; 32]);

    let (ledger_opt, status) = ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    assert_eq!(status, LedgerOpenStatus::Fresh);
    let mut ledger = ledger_opt.unwrap();

    let entry1 = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: [0xAA; 32],
        erased_at_tx: 10,
        proof_id: [1u8; 16],
    };

    let entry2_same_hash_different_proof_id = LedgerEntry {
        ledger_seq: 2,
        kind: LedgerEntryKind::Key,
        keyed_hash: [0xAA; 32],
        erased_at_tx: 10,
        proof_id: [2u8; 16],
    };

    ledger.append(&[entry1.clone()]).await.unwrap();
    assert_eq!(ledger.entries().len(), 1);

    // Append entry with identical keyed_hash -> should be idempotently ignored
    ledger.append(&[entry2_same_hash_different_proof_id]).await.unwrap();
    assert_eq!(ledger.entries().len(), 1);
}

#[tokio::test]
async fn ledger_privacy_no_plain_keys_on_disk() {
    let dir = tempdir().unwrap();
    let key = LedgerKey::new([0x77; 32]);

    let (ledger_opt, _) = ErasureLedger::open_or_fail_closed(dir.path(), key.clone(), false, None)
        .await
        .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let plain_key = b"super_secret_user_ssn_12345";
    let scope = b"user_records";
    let keyed_hash = compute_keyed_hash(&key, scope, plain_key);

    let entry = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash,
        erased_at_tx: 100,
        proof_id: [0x99; 16],
    };

    ledger.append(&[entry]).await.unwrap();

    // Read raw file on disk and verify cleartext plain_key NEVER appears
    let ledger_path = dir.path().join("erasure.ledger");
    let disk_bytes = tokio::fs::read(&ledger_path).await.unwrap();

    let found = disk_bytes
        .windows(plain_key.len())
        .any(|window| window == plain_key);
    assert!(!found, "Cleartext key was found in erasure.ledger file!");
}

#[tokio::test]
async fn ledger_append_dir_sync_error_propagates() {
    let dir = tempdir().unwrap();
    let ledger_dir = dir.path().join("ledger_sub");
    tokio::fs::create_dir(&ledger_dir).await.unwrap();

    let key = LedgerKey::new([0x12; 32]);

    let (ledger_opt, _) = ErasureLedger::open_or_fail_closed(&ledger_dir, key, false, None)
        .await
        .unwrap();
    assert!(ledger_opt.is_some());
    let mut ledger = ledger_opt.unwrap();

    // Delete directory behind ledger's back to force sync/append error
    tokio::fs::remove_dir_all(&ledger_dir).await.unwrap();

    let entry = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: [0x55; 32],
        erased_at_tx: 50,
        proof_id: [0x11; 16],
    };

    let res = ledger.append(&[entry]).await;
    assert!(res.is_err(), "Append must propagate directory/file sync errors fail-closed");
}
