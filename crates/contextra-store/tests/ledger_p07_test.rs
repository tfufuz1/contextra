// FILE-CONTEXT
// STAND: 2026-10-21 (P07)
// ZWECK: Integrationstests für ErasureLedger Idempotenz, Durabilität, Privacy und Query-APIs (WP-8 / P07).

#![forbid(unsafe_code)]

use contextra_store::erasure_ledger::*;
use std::sync::Arc;
use tempfile::tempdir;

static TEST_SERIAL_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
async fn ledger_append_idempotent_across_reopen() {
    let _guard = TEST_SERIAL_LOCK.lock().await;
    let dir = tempdir().unwrap();
    let master_secret = [11u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, status) =
        ErasureLedger::open_or_fail_closed_with_instance_id(dir.path(), key.clone(), false, None, [1u8; 16])
            .await
            .unwrap();
    assert_eq!(status, LedgerOpenStatus::Fresh);
    let mut ledger = ledger_opt.unwrap();

    let entry1 = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: compute_keyed_hash(&key, b"tenant_x", b"user_secret_key_101"),
        erased_at_tx: 100,
        proof_id: [10u8; 16],
    };

    // First append
    ledger.append(&[entry1.clone()]).await.unwrap();
    assert_eq!(ledger.entries().len(), 1);
    let mac1 = *ledger.last_chain_mac();

    // Repeated append on same instance (identical entry & identical keyed_hash)
    ledger.append(&[entry1.clone()]).await.unwrap();
    assert_eq!(ledger.entries().len(), 1, "Duplicate append on same instance must be idempotent");
    assert_eq!(*ledger.last_chain_mac(), mac1, "Chain MAC must not change on no-op append");

    // Repeat append via ErasureLedgerHandle
    let handle = ErasureLedgerHandle::new(ledger);
    handle.append(&[entry1.clone()]).await.unwrap();
    assert_eq!(handle.entries().len(), 1, "Duplicate append via handle must be idempotent");
    assert_eq!(handle.last_chain_mac(), mac1);

    // Reopen ledger from disk
    let (reopened_opt, status2) =
        ErasureLedger::open_or_fail_closed_with_instance_id(dir.path(), key.clone(), true, None, [1u8; 16])
            .await
            .unwrap();
    assert_eq!(status2, LedgerOpenStatus::Ok);
    let mut reopened = reopened_opt.unwrap();
    assert_eq!(reopened.entries().len(), 1);

    // Repeated append after reopen
    reopened.append(&[entry1.clone()]).await.unwrap();
    assert_eq!(reopened.entries().len(), 1, "Duplicate append after reopen must be idempotent");
    assert_eq!(*reopened.last_chain_mac(), mac1);
}

#[tokio::test]
async fn ledger_append_in_batch_deduplication() {
    let _guard = TEST_SERIAL_LOCK.lock().await;
    let dir = tempdir().unwrap();
    let master_secret = [55u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, _) =
        ErasureLedger::open_or_fail_closed_with_instance_id(dir.path(), key.clone(), false, None, [5u8; 16])
            .await
            .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let entry1 = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: compute_keyed_hash(&key, b"tenant_batch", b"key_batch_1"),
        erased_at_tx: 10,
        proof_id: [51u8; 16],
    };

    // Pass batch containing duplicates
    ledger.append(&[entry1.clone(), entry1.clone(), entry1.clone()]).await.unwrap();
    assert_eq!(ledger.entries().len(), 1, "Batch with duplicates must deduplicate to 1 entry");

    let handle = ErasureLedgerHandle::new(ledger);
    handle.append(&[entry1.clone(), entry1.clone()]).await.unwrap();
    assert_eq!(handle.entries().len(), 1, "Handle batch with duplicates must deduplicate to 1 entry");
}

#[tokio::test]
async fn ledger_append_is_durable_after_return() {
    let _guard = TEST_SERIAL_LOCK.lock().await;
    let dir = tempdir().unwrap();
    let master_secret = [22u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, _) =
        ErasureLedger::open_or_fail_closed_with_instance_id(dir.path(), key.clone(), false, None, [2u8; 16])
            .await
            .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let entry = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: compute_keyed_hash(&key, b"tenant_y", b"user_key_202"),
        erased_at_tx: 50,
        proof_id: [20u8; 16],
    };

    // Inject dir-fsync failure
    test_set_ledger_dir_fsync_hook(Some(|_| {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "INJECTED_DIR_FSYNC_FAILURE",
        ))
    }));

    let err_res = ledger.append(&[entry.clone()]).await;
    assert!(
        err_res.is_err(),
        "ledger.append MUST return error when dir fsync fails"
    );
    assert!(
        err_res.unwrap_err().to_string().contains("INJECTED_DIR_FSYNC_FAILURE"),
        "Error message must convey dir fsync failure"
    );

    // Clear fault hook
    test_set_ledger_dir_fsync_hook(None);

    // Append succeeds and persists durably
    ledger.append(&[entry.clone()]).await.unwrap();
    assert_eq!(ledger.entries().len(), 1);

    // Reopen and confirm durability
    let (reopened_opt, status) =
        ErasureLedger::open_or_fail_closed_with_instance_id(dir.path(), key.clone(), true, None, [2u8; 16])
            .await
            .unwrap();
    assert_eq!(status, LedgerOpenStatus::Ok);
    let reopened = reopened_opt.unwrap();
    assert_eq!(reopened.entries().len(), 1);
    assert_eq!(reopened.entries()[0], entry);
}

#[tokio::test]
async fn ledger_privacy_no_plain_keys_on_disk() {
    let _guard = TEST_SERIAL_LOCK.lock().await;
    let dir = tempdir().unwrap();
    let master_secret = [33u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, _) =
        ErasureLedger::open_or_fail_closed_with_instance_id(dir.path(), key.clone(), false, None, [3u8; 16])
            .await
            .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let plaintext_key_1 = b"MARKER_SECRET_PLAIN_KEY_998877";
    let plaintext_prefix = b"MARKER_SECRET_PLAIN_PREFIX_";

    let entry1 = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: compute_keyed_hash(&key, b"tenant_p", plaintext_key_1),
        erased_at_tx: 100,
        proof_id: [31u8; 16],
    };

    let entry2 = LedgerEntry {
        ledger_seq: 2,
        kind: LedgerEntryKind::Prefix { prefix_len: plaintext_prefix.len() as u32 },
        keyed_hash: compute_keyed_hash(&key, b"tenant_p", plaintext_prefix),
        erased_at_tx: 120,
        proof_id: [32u8; 16],
    };

    ledger.append(&[entry1, entry2]).await.unwrap();

    // Raw sweep over all files in the ledger directory
    let mut entries_dir = tokio::fs::read_dir(dir.path()).await.unwrap();
    while let Some(file_entry) = entries_dir.next_entry().await.unwrap() {
        let path = file_entry.path();
        if path.is_file() {
            let bytes = tokio::fs::read(&path).await.unwrap();

            // Check for plaintext key 1
            assert!(
                !bytes.windows(plaintext_key_1.len()).any(|w| w == plaintext_key_1),
                "Plaintext key {:?} MUST NOT exist on disk in {}",
                String::from_utf8_lossy(plaintext_key_1),
                path.display()
            );

            // Check for plaintext prefix
            assert!(
                !bytes.windows(plaintext_prefix.len()).any(|w| w == plaintext_prefix),
                "Plaintext prefix {:?} MUST NOT exist on disk in {}",
                String::from_utf8_lossy(plaintext_prefix),
                path.display()
            );
        }
    }
}

#[tokio::test]
async fn ledger_is_erased_after_reopen() {
    let _guard = TEST_SERIAL_LOCK.lock().await;
    let dir = tempdir().unwrap();
    let master_secret = [44u8; 32];
    let key = LedgerKey::derive_from_master_secret(&master_secret);

    let (ledger_opt, _) =
        ErasureLedger::open_or_fail_closed_with_instance_id(dir.path(), key.clone(), false, None, [4u8; 16])
            .await
            .unwrap();
    let mut ledger = ledger_opt.unwrap();

    let hash_1 = compute_keyed_hash(&key, b"scope_z", b"doc_101");
    let hash_2 = compute_keyed_hash(&key, b"scope_z", b"doc_202");

    let entry1 = LedgerEntry {
        ledger_seq: 1,
        kind: LedgerEntryKind::Key,
        keyed_hash: hash_1,
        erased_at_tx: 100,
        proof_id: [41u8; 16],
    };

    let entry2 = LedgerEntry {
        ledger_seq: 2,
        kind: LedgerEntryKind::Key,
        keyed_hash: hash_2,
        erased_at_tx: 200,
        proof_id: [42u8; 16],
    };

    ledger.append(&[entry1.clone(), entry2.clone()]).await.unwrap();

    // Reopen and test query methods
    let (reopened_opt, status) =
        ErasureLedger::open_or_fail_closed_with_instance_id(dir.path(), key.clone(), true, None, [4u8; 16])
            .await
            .unwrap();
    assert_eq!(status, LedgerOpenStatus::Ok);
    let reopened = reopened_opt.unwrap();

    // Query is_erased
    assert!(reopened.is_erased(&hash_1));
    assert!(reopened.is_erased(&hash_2));

    let unknown_hash = compute_keyed_hash(&key, b"scope_z", b"unknown_doc");
    assert!(!reopened.is_erased(&unknown_hash));

    // Query is_erased_at_tx
    // Document written at tx=50 was erased at tx=100 -> drop/erased
    assert!(reopened.is_erased_at_tx(&hash_1, 50));
    assert!(reopened.is_erased_at_tx(&hash_1, 100));
    // Document written at tx=150 was written AFTER erasure at tx=100 -> not erased
    assert!(!reopened.is_erased_at_tx(&hash_1, 150));

    // Query entries_since_tx
    let entries_since_150 = reopened.entries_since_tx(150);
    assert_eq!(entries_since_150.len(), 1);
    assert_eq!(entries_since_150[0], entry2);

    let entries_since_50 = reopened.entries_since_tx(50);
    assert_eq!(entries_since_50.len(), 2);

    // Query via ErasureLedgerHandle
    let handle = Arc::new(ErasureLedgerHandle::new(reopened));
    assert!(handle.is_erased(&hash_1));
    assert!(handle.is_erased_at_tx(&hash_1, 50));
    assert!(!handle.is_erased_at_tx(&hash_1, 150));
    assert_eq!(handle.entries_since_tx(150).len(), 1);
}
