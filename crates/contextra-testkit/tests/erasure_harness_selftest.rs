// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Selbsttest fuer erasure_harness (ReferenceStore, Rohsweep, TenantMatrixRunner, Failpoints).
// INVARIANTEN: #![forbid(unsafe_code)], Verifiziert Selbsttest-Akzeptanzkriterien.

#![forbid(unsafe_code)]

use contextra_testkit::erasure_harness::{
    raw_sweep, KeyState, ReferenceStore, TenantMatrixMode, TenantMatrixRunner,
};
use contextra_types::TxId;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_raw_sweep_finds_known_value() {
    let dir = tempdir().expect("Failed to create temp dir");
    let file_path = dir.path().join("wal_001.log");
    let mut file = File::create(&file_path).expect("Failed to create file");

    let header = b"HEADER_RECORD_";
    let target_needle = b"CONFIDENTIAL_TENANT_KEY_999";
    let footer = b"_TRAILER_END";

    file.write_all(header).expect("write header");
    file.write_all(target_needle).expect("write needle");
    file.write_all(footer).expect("write footer");
    file.flush().expect("flush file");

    let needles = vec![target_needle.to_vec(), b"ABSENT_NEEDLE".to_vec()];
    let report = raw_sweep(dir.path(), &needles).expect("raw_sweep execution failed");

    assert_eq!(report.files_scanned, 1);
    assert_eq!(report.hit_count(), 1);
    assert!(!report.is_clean());

    let hit = &report.hits[0];
    assert_eq!(hit.needle_index, 0);
    assert_eq!(hit.offset, header.len() as u64);
    assert_eq!(hit.relative_path, PathBuf::from("wal_001.log"));
}

#[test]
fn test_reference_store_handcheck() {
    let mut ref_store = ReferenceStore::new();
    assert!(ref_store.is_empty());

    // Hand-check step 1: Put keys
    ref_store.put(b"user:100", b"alice");
    ref_store.put(b"user:101", b"bob");
    ref_store.put(b"doc:001", b"document_data");

    assert_eq!(ref_store.get(b"user:100"), Some(&b"alice"[..]));
    assert_eq!(ref_store.get(b"user:101"), Some(&b"bob"[..]));
    assert_eq!(ref_store.active_count(), 3);
    assert_eq!(ref_store.total_entries(), 3);

    // Hand-check step 2: Delete key
    ref_store.delete(b"user:100");
    assert_eq!(ref_store.get(b"user:100"), None);
    assert_eq!(ref_store.state_of(b"user:100"), Some(&KeyState::Tombstone));
    assert_eq!(ref_store.active_count(), 2);
    assert_eq!(ref_store.total_entries(), 3);

    // Hand-check step 3: Purge key
    ref_store.purge(b"user:100");
    assert_eq!(ref_store.state_of(b"user:100"), Some(&KeyState::Purged));

    // Hand-check step 4: Prefix scan
    let users = ref_store.scan_prefix(b"user:");
    assert_eq!(users, vec![(b"user:101".to_vec(), b"bob".to_vec())]);

    // Hand-check step 5: Active snapshot
    let snapshot = ref_store.active_snapshot();
    assert_eq!(snapshot.len(), 2);
    assert_eq!(snapshot.get(&b"user:101"[..]), Some(&b"bob".to_vec()));
    assert_eq!(
        snapshot.get(&b"doc:001"[..]),
        Some(&b"document_data".to_vec())
    );
}

#[tokio::test]
async fn test_tenant_matrix_runner_lsm_and_tenant_scoped() {
    TenantMatrixRunner::run_async(|ctx| {
        let mode = ctx.mode;
        let storage = ctx.storage;
        async move {
            let tx1 = TxId::try_new(100).unwrap();
            storage
                .put(tx1, b"harness_key_1", b"harness_val_1")
                .await
                .unwrap();
            storage.commit(tx1).await.unwrap();

            let val = storage.get(b"harness_key_1").await.unwrap();
            assert_eq!(val.as_deref(), Some(&b"harness_val_1"[..]));

            match mode {
                TenantMatrixMode::LsmDirect => {
                    assert!(ctx.dir_path.exists());
                }
                TenantMatrixMode::TenantScoped(tid) => {
                    assert_eq!(tid.0, 101);
                }
            }
        }
    })
    .await;
}
