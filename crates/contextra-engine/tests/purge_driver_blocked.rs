//! Integration test verifying PurgeDriver blocking behavior (§C5.3, C4.8).

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_engine::deletion::{
    DpiDeletionRuntime, DpiPurgeDriver, DpiPurgeDriverConfig, FonLogicalHandle, ForgetOptions,
    PurgeHandle, PurgeMode,
};
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::copy_registry::{CopyKind, CopyRegEntry, CopyRegistry};
use contextra_store::lsm::LsmConfig;
use contextra_store::LsmStorage;
use contextra_types::TxId;
use tempfile::TempDir;

#[tokio::test]
async fn test_purge_driver_blocked_on_unauthorized_copies() {
    let db_dir = TempDir::new().expect("tempdir failed");
    let signer_key_path = db_dir.path().join("proof_signer.key");
    let db_instance_id = [0x22; 16];

    let runtime = DpiDeletionRuntime::open(db_dir.path(), Some(&signer_key_path), db_instance_id)
        .expect("DpiDeletionRuntime::open failed");

    let config = LsmConfig {
        path: db_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await.expect("LsmStorage::open failed");

    // Register a copy holding 000001.sst
    let copy_reg_dir = db_dir.path().join("copy_reg");
    std::fs::create_dir_all(&copy_reg_dir).expect("create_dir_all failed");
    let mut copy_reg = CopyRegistry::open(&copy_reg_dir)
        .await
        .expect("CopyRegistry::open failed");

    let scope_file_names = vec!["000001.sst".to_string()];
    let fid = contextra_checkpoint::copy_registry::file_id_for_name("000001.sst");

    let entry = CopyRegEntry {
        copy_id: [0x01; 16],
        kind: CopyKind::Hardlink,
        file_ids: vec![fid],
        created_tx: 10,
    };
    copy_reg.register(entry).await.expect("register copy failed");

    let logical_handle = FonLogicalHandle {
        proof_id: [0x55; 16],
        proof_path: db_dir.path().join("proofs").join("00.json"),
        full_proof_id: [0x55; 32],
    };

    let driver = DpiPurgeDriver::new(&storage, &runtime, DpiPurgeDriverConfig::default());

    // Request Purge without PurgeWithScrub authorization -> Must return Blocked
    let opts = ForgetOptions::new()
        .with_assurance(WitAssurance::S1)
        .with_purge(PurgeMode::Purge);

    let purge_scope = PurgeScope::Keys(vec![b"k1".to_vec()]);

    let handle = driver
        .run_physical(
            purge_scope,
            &scope_file_names,
            TxId::new(10),
            &logical_handle,
            &opts,
            None,
            &[],
            Some(&copy_reg),
        )
        .await
        .expect("run_physical failed");

    match handle {
        PurgeHandle::Blocked { reason, retry_hint } => {
            assert!(reason.contains("Registered checkpoint copies"));
            assert!(retry_hint.contains("Authorize purge_checkpoint"));
        }
        other => panic!("Expected PurgeHandle::Blocked, got {:?}", other),
    }
}
