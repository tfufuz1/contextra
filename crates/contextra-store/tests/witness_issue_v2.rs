//! Integration tests for issue_witnesses_v2 and store_probe (T-2026-9220).

use contextra_core::TxId;
use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_store::compaction::CompactionConfig;
use contextra_store::lsm::witness::ZeuIssueRequest;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::time::Duration;
use tempfile::TempDir;

async fn create_test_storage() -> (LsmStorage, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");
    (storage, tmp)
}

#[tokio::test]
async fn test_issue_witnesses_v2_clean_roundtrip() {
    let (storage, _tmp) = create_test_storage().await;
    let key = b"test_key_v2".to_vec();

    let req = ZeuIssueRequest {
        scope_keys: vec![key.clone()],
        scope_prefixes: vec![],
        as_of_tx: TxId::new(10),
        scanned_at_tx: TxId::new(10),
        assurance: WitAssurance::S2,
        claim: WitClaimText::S2V,
        scanner_name: "zeu_scanner_v2",
        scanner_version: 1,
        key_identifiers_claimed: false,
    };

    let witnesses = storage
        .issue_witnesses_v2(req)
        .await
        .expect("issue_witnesses_v2 clean roundtrip should succeed");

    assert!(witnesses.len() >= 3, "Expected witnesses for Memtable, SSTables, and WAL");

    let layers: Vec<_> = witnesses.iter().map(|w| w.layer()).collect();
    assert!(layers.contains(&&DeletionLayer::LsmMemtable));
    assert!(layers.contains(&&DeletionLayer::SsTableAllLevels));
}

#[tokio::test]
async fn test_store_probe_interface() {
    let (storage, _tmp) = create_test_storage().await;

    let probe = storage.store_probe();
    let gen_mem = probe.current_generation(&DeletionLayer::LsmMemtable);
    let gen_sst = probe.current_generation(&DeletionLayer::SsTableAllLevels);

    assert_eq!(gen_mem.0, 0);
    assert_eq!(gen_sst.0, 1);

    let seq = probe.scope_write_seq(&[0u8; 32]);
    assert_eq!(seq, 0);

    let count = probe.manifest_file_count();
    assert_eq!(count, Some(0));
}

#[tokio::test]
async fn test_legacy_issue_layer_witness_rejects_scanless_layers() {
    let (storage, _tmp) = create_test_storage().await;
    let key = b"legacy_key".to_vec();
    let tx = TxId::new(10);

    // Memtable remains allowed for legacy issue_layer_witness
    let mem_res = storage
        .issue_layer_witness(DeletionLayer::LsmMemtable, &[key.clone()], &[], tx)
        .await;
    assert!(mem_res.is_ok());

    // SsTableAllLevels MUST fail in issue_layer_witness to prevent scan-less false positives
    let sst_res = storage
        .issue_layer_witness(DeletionLayer::SsTableAllLevels, &[key.clone()], &[], tx)
        .await;
    assert!(
        sst_res.is_err(),
        "issue_layer_witness for SsTableAllLevels MUST be rejected without scan"
    );

    // WalAllSegments MUST fail in issue_layer_witness to prevent scan-less false positives
    let wal_res = storage
        .issue_layer_witness(
            DeletionLayer::WalAllSegments { seq_after: 1 },
            &[key.clone()],
            &[],
            tx,
        )
        .await;
    assert!(
        wal_res.is_err(),
        "issue_layer_witness for WalAllSegments MUST be rejected without scan"
    );
}
