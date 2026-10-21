// FILE-CONTEXT
// STAND: 2026-10-21 (W1-13)
// ZWECK: Stage 5 Tests for Restore Anti-Resurrection Filtering and Fail-Closed Enforcement.

#![allow(deprecated)]

use contextra_checkpoint::{PersistentCheckpointStore, RestorePolicy};
use contextra_ports::{RestoreGuard, StorageEngine};
use contextra_store::erasure_ledger::*;
use contextra_store::{LsmConfig, LsmStorage, StreamingRestoreGuard};
use contextra_types::TxId;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn restore_without_filter_after_erasure_fails_closed() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("db");
    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::open(config).await.unwrap());

    let tx5 = TxId::try_new(5).unwrap();
    lsm.put(tx5, b"key1", b"val1").await.unwrap();
    lsm.commit(tx5).await.unwrap();

    let store = PersistentCheckpointStore::open(lsm.clone(), "test_ns")
        .await
        .unwrap();

    let cp = store
        .create_checkpoint("cp1", "col1", 5, tx5, serde_json::json!({}))
        .await
        .unwrap();

    // Perform another commit at tx 10 so last_tx > cp.tx_id
    let tx10 = TxId::try_new(10).unwrap();
    lsm.put(tx10, b"key2", b"val2").await.unwrap();
    lsm.commit(tx10).await.unwrap();

    // Attempting restore without filter when subsequent tx exists (last_tx > cp.tx_id) fails closed
    let res = store.restore_checkpoint(&cp.name).await;
    assert!(res.is_err(), "Restore without filter after subsequent commits must fail closed");
}

#[tokio::test]
async fn delete_then_restore_keeps_deleted_deleted() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("db");
    tokio::fs::create_dir_all(&db_path).await.unwrap();

    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::open(config).await.unwrap());

    let tx5 = TxId::try_new(5).unwrap();
    lsm.put(tx5, b"user_key", b"user_val").await.unwrap();
    lsm.commit(tx5).await.unwrap();

    let ledger_key = LedgerKey::new([0x66; 32]);
    let (ledger_opt, _) =
        ErasureLedger::open_or_fail_closed(&db_path, ledger_key.clone(), false, None)
            .await
            .unwrap();
    let mut ledger = ledger_opt.unwrap();

    // Record erasure in ledger at tx 10
    let keyed_hash = compute_keyed_hash(&ledger_key, b"lsm", b"user_key");
    let entry = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash,
        erased_at_tx: 10,
        proof_id: [2u8; 16],
    };
    ledger.append(&[entry]).await.unwrap();

    let guard = Arc::new(StreamingRestoreGuard::for_lsm(Arc::new(ledger)));
    let policy = RestorePolicy::Filter(guard);
    assert!(
        policy.drop_record(b"user_key", 5),
        "Record erased in ledger must be marked for drop during restore"
    );
}

#[tokio::test]
async fn restore_rejects_key_erased_after_checkpoint_tx() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("db");
    tokio::fs::create_dir_all(&db_path).await.unwrap();

    let ledger_key = LedgerKey::new([0x88; 32]);
    let (ledger_opt, _) =
        ErasureLedger::open_or_fail_closed(&db_path, ledger_key.clone(), false, None)
            .await
            .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let erased_hash = compute_keyed_hash(&ledger_key, b"lsm", b"erased_key");
    let entry = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: erased_hash,
        erased_at_tx: 10, // Erased at tx 10
        proof_id: [8u8; 16],
    };
    ledger.append(&[entry]).await.unwrap();

    let guard = StreamingRestoreGuard::for_lsm(Arc::new(ledger));
    let tx5 = TxId::try_new(5).unwrap();

    // Key present in checkpoint at tx 5, but erased at tx 10 (erased_at_tx >= source_tx 5) -> MUST BE REJECTED (admit == Reject)
    assert_eq!(guard.admit(b"erased_key", tx5), contextra_ports::RestoreVerdict::Reject);

    // Non-erased key at tx 5 -> MUST BE ALLOWED (admit == Allow)
    assert_eq!(guard.admit(b"active_key", tx5), contextra_ports::RestoreVerdict::Allow);
}
