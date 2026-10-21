// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Stage 6 mandatory witness footprint and store generation tests (§X-04, WP-12).

use contextra_ports::StorageEngine;
use contextra_types::TxId;
use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_store::compaction::purge_filter::PurgeScope;
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
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");
    (storage, tmp)
}

#[tokio::test]
async fn witness_footprint_nonzero_and_deterministic() {
    let (storage, _tmp) = create_test_storage().await;

    let req1 = ZeuIssueRequest {
        scope_keys: vec![b"keyA".to_vec(), b"keyB".to_vec()],
        scope_prefixes: vec![b"prefix1".to_vec()],
        as_of_tx: TxId::try_new(10).unwrap(),
        scanned_at_tx: TxId::try_new(10).unwrap(),
        assurance: WitAssurance::S2,
        claim: WitClaimText::S2V,
        scanner_name: "test_scanner",
        scanner_version: 1,
        key_identifiers_claimed: false,
    };

    let witnesses1 = storage.issue_witnesses_v2(req1.clone()).await.unwrap();
    let fp1 = witnesses1[0].footprint_hash();

    assert_ne!(fp1, &[0u8; 32], "Footprint hash must be non-zero");

    let witnesses2 = storage.issue_witnesses_v2(req1).await.unwrap();
    let fp2 = witnesses2[0].footprint_hash();

    assert_eq!(fp1, fp2, "Footprint hash must be deterministic for identical scope");
}

#[tokio::test]
async fn witness_footprint_differs_for_different_scope() {
    let (storage, _tmp) = create_test_storage().await;

    let req1 = ZeuIssueRequest {
        scope_keys: vec![b"keyA".to_vec()],
        scope_prefixes: vec![],
        as_of_tx: TxId::try_new(10).unwrap(),
        scanned_at_tx: TxId::try_new(10).unwrap(),
        assurance: WitAssurance::S2,
        claim: WitClaimText::S2V,
        scanner_name: "test_scanner",
        scanner_version: 1,
        key_identifiers_claimed: false,
    };

    let req2 = ZeuIssueRequest {
        scope_keys: vec![b"keyB".to_vec()],
        scope_prefixes: vec![],
        as_of_tx: TxId::try_new(10).unwrap(),
        scanned_at_tx: TxId::try_new(10).unwrap(),
        assurance: WitAssurance::S2,
        claim: WitClaimText::S2V,
        scanner_name: "test_scanner",
        scanner_version: 1,
        key_identifiers_claimed: false,
    };

    let witnesses1 = storage.issue_witnesses_v2(req1).await.unwrap();
    let witnesses2 = storage.issue_witnesses_v2(req2).await.unwrap();

    assert_ne!(
        witnesses1[0].footprint_hash(),
        witnesses2[0].footprint_hash(),
        "Footprint hash must differ for different scopes"
    );
}

#[tokio::test]
async fn store_generation_advances_after_purge() {
    let (storage, _tmp) = create_test_storage().await;

    let probe = storage.store_probe();
    let gen_before = probe.current_generation(&DeletionLayer::SsTableAllLevels);

    // Write and purge a key to advance manifest and store generation
    let tx = TxId::try_new(10).unwrap();
    storage.put(tx, b"key1", b"val1").await.unwrap();
    storage.commit(tx).await.unwrap();

    let scope = PurgeScope::Keys(vec![b"key1".to_vec()]);
    storage.purge_keys(scope, tx.inner(), None).await.unwrap();

    let gen_after = probe.current_generation(&DeletionLayer::SsTableAllLevels);

    assert!(
        gen_after.0 > gen_before.0,
        "Store generation for SsTableAllLevels must advance after purge (before: {}, after: {})",
        gen_before.0,
        gen_after.0
    );
}
