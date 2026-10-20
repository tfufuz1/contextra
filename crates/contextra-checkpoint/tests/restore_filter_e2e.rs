// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Ende-zu-Ende Integrationstests für Anti-Resurrection Restore-Filter und Checkpoint-Wiederherstellung.
// INVARIANTEN: Zero-Panic, Fail-Closed ohne Filter bei Löschhistorie, Anti-Resurrection gelöschter Daten.

use contextra_checkpoint::PersistentCheckpointStore;
use contextra_ports::StorageEngine;
use contextra_store::erasure_ledger::{
    compute_keyed_hash, LedgerEntry, LedgerEntryKind, LedgerKey,
};
use contextra_store::lsm::config::EncryptionConfig;
use contextra_store::{LsmConfig, LsmStorage, StreamingRestoreGuard};
use contextra_types::TxId;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_restore_filter_e2e_prevents_resurrection() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        encryption: EncryptionConfig::Passphrase("test_passphrase".to_string()),
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::open(config).await.expect("open lsm storage"));
    let mut store = PersistentCheckpointStore::open(storage.clone(), "agent")
        .await
        .expect("open checkpoint store");

    let ledger_handle = storage
        .erasure_ledger_handle
        .as_ref()
        .expect("erasure ledger handle must be initialized when encryption is active")
        .clone();

    let guard = StreamingRestoreGuard::for_lsm_handle(ledger_handle.clone());
    store.set_restore_filter(Arc::new(guard));

    // 1. Write initial data at tx = 10
    #[allow(deprecated)]
    let tx10 = TxId::new(10);
    storage
        .put(tx10, b"key_normal", b"val_normal")
        .await
        .expect("put normal");
    storage
        .put(tx10, b"key_erased", b"val_erased")
        .await
        .expect("put erased");
    storage.commit(tx10).await.expect("commit tx10");

    // 2. Create checkpoint CP1 at tx = 10
    let cp1 = store
        .create_checkpoint("CP1", "agent_col", 1, tx10, serde_json::json!({}))
        .await
        .expect("create checkpoint CP1");
    assert_eq!(cp1.tx_id, tx10);

    // 3. Perform deletion of key_erased at tx = 20 & record erasure entry in ledger
    #[allow(deprecated)]
    let tx20 = TxId::new(20);
    storage
        .delete(tx20, b"key_erased")
        .await
        .expect("delete erased key");
    storage.commit(tx20).await.expect("commit tx20");

    let salt_bytes = tokio::fs::read(dir.path().join("SALT"))
        .await
        .expect("read SALT file");
    let salt_arr: &[u8; 32] = salt_bytes
        .as_slice()
        .try_into()
        .expect("SALT file must be 32 bytes");
    let ledger_key = LedgerKey::derive_from_master_secret(salt_arr);
    let erased_hash = compute_keyed_hash(&ledger_key, b"lsm", b"key_erased");

    let entry = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: erased_hash,
        erased_at_tx: 20,
        proof_id: [1u8; 16],
    };

    ledger_handle.append(&[entry]).await.expect("append erasure entry");

    // 4. Restore checkpoint CP1
    let restored_meta = store
        .restore_checkpoint("CP1")
        .await
        .expect("restore checkpoint CP1 must succeed");
    assert_eq!(restored_meta.tx_id, tx10);

    // 5. Verify: key_normal is present, key_erased is absent (not resurrected!)
    let val_normal = storage.get(b"key_normal").await.expect("get key_normal");
    assert_eq!(
        val_normal.as_deref(),
        Some(&b"val_normal"[..]),
        "Unerased record must be restored"
    );

    let val_erased = storage.get(b"key_erased").await.expect("get key_erased");
    assert_eq!(
        val_erased, None,
        "Erased record MUST NOT be resurrected on restore"
    );

    storage.close().await.expect("close storage");
}

#[tokio::test]
async fn test_restore_filter_fail_closed_without_filter_when_history_exists() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::open(config).await.expect("open lsm storage"));
    let store = PersistentCheckpointStore::open(storage.clone(), "test_fail_closed")
        .await
        .expect("open checkpoint store");

    // 1. Write data & checkpoint at tx = 10
    #[allow(deprecated)]
    let tx10 = TxId::new(10);
    storage.put(tx10, b"doc1", b"data1").await.expect("put doc1");
    storage.commit(tx10).await.expect("commit tx10");

    store
        .create_checkpoint("CP1", "col", 1, tx10, serde_json::json!({}))
        .await
        .expect("create checkpoint");

    // 2. Perform subsequent transaction at tx = 20
    #[allow(deprecated)]
    let tx20 = TxId::new(20);
    storage.put(tx20, b"doc2", b"data2").await.expect("put doc2");
    storage.commit(tx20).await.expect("commit tx20");

    // 3. Restore CP1 without filter -> MUST fail closed!
    let err = store
        .restore_checkpoint("CP1")
        .await
        .expect_err("restore without filter must fail when subsequent transactions exist");

    let err_msg = err.to_string();
    assert!(
        err_msg.contains("Fail-closed restore barrier"),
        "Error message must indicate fail-closed barrier: {err_msg}"
    );

    storage.close().await.expect("close storage");
}

#[tokio::test]
async fn test_restore_checkpoint_without_deletion_history_succeeds() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::open(config).await.expect("open lsm storage"));
    let store = PersistentCheckpointStore::open(storage.clone(), "test_no_history")
        .await
        .expect("open checkpoint store");

    // 1. Write data & checkpoint at tx = 10
    #[allow(deprecated)]
    let tx10 = TxId::new(10);
    storage.put(tx10, b"doc1", b"data1").await.expect("put doc1");
    storage.commit(tx10).await.expect("commit tx10");

    store
        .create_checkpoint("CP1", "col", 1, tx10, serde_json::json!({}))
        .await
        .expect("create checkpoint");

    // 2. Restore CP1 when no subsequent transactions exist (last_tx == tx10)
    let restored = store
        .restore_checkpoint("CP1")
        .await
        .expect("restore without deletion history must succeed");
    assert_eq!(restored.tx_id, tx10);

    let val = storage.get(b"doc1").await.expect("get doc1");
    assert_eq!(val.as_deref(), Some(&b"data1"[..]));

    storage.close().await.expect("close storage");
}
