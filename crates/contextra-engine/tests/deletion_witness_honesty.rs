//! Integration tests for deletion witness honesty (INV-D5).

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::claim_text::WitClaimText;
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::probe::WitStoreGeneration;
use contextra_crypto::deletion_proof::types::{hash_deleted_keys_length_prefixed, DeletionLayer};
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_engine::deletion::relations_witness::issue_relations_witness;
use contextra_engine::deletion::text_witness::issue_text_index_witness;
use contextra_engine::deletion::{
    forget_document, DpiDeletionRuntime, ErasureFootprint, FprSweepSource, KeyFamilyRegistry, ForgetOptions,
};
use contextra_ports::{DocId, TxId};
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{ContextraError, Result};
use tempfile::TempDir;

struct DummySweepSource;

impl FprSweepSource for DummySweepSource {
    fn for_each_key(&self, _callback: &mut dyn FnMut(&[u8]) -> bool) -> Result<()> {
        Ok(())
    }
}

fn create_full_witness_set(
    doc_id: DocId,
    scope_hash: [u8; 32],
    footprint_hash: [u8; 32],
    as_of_tx: TxId,
    opts: &ForgetOptions,
) -> Result<Vec<LayerWitness>> {
    let hnsw_witness = LayerWitness::issue(WitIssueArgs {
        layer: DeletionLayer::HnswIndex,
        scope_hash,
        scanned_at_tx: as_of_tx,
        as_of_tx,
        assurance: opts.assurance,
        claim: WitClaimText::S1,
        footprint_hash,
        scope_write_seq: 0,
        scanner_name: "hnsw_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(0),
        layer_evidence: WitLayerEvidence::HnswIndex {
            node_id: doc_id.inner(),
            incoming_edges: 0,
            outgoing_edges: 0,
            pinned_by_reader: false,
            marked_zeroed: true,
        },
        key_identifiers_claimed: false,
    })?;

    let text_witness = issue_text_index_witness(doc_id, &[], scope_hash, as_of_tx, Some(0))?;
    let rel_witness = issue_relations_witness(doc_id, 0, scope_hash, as_of_tx, Some(0))?;

    let ledger_witness = LayerWitness::issue(WitIssueArgs {
        layer: DeletionLayer::ErasureLedger,
        scope_hash,
        scanned_at_tx: as_of_tx,
        as_of_tx,
        assurance: opts.assurance,
        claim: WitClaimText::S1,
        footprint_hash,
        scope_write_seq: 0,
        scanner_name: "ledger_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(0),
        layer_evidence: WitLayerEvidence::Memtable {
            memtables_scanned: 1,
            entries_found: 0,
            tombstones_found: 0,
        },
        key_identifiers_claimed: false,
    })?;

    Ok(vec![hnsw_witness, text_witness, rel_witness, ledger_witness])
}

fn setup_runtime(dir: &std::path::Path) -> DpiDeletionRuntime {
    DpiDeletionRuntime::open(dir, None, [1u8; 16]).unwrap()
}

#[tokio::test]
async fn missing_injected_hnsw_witness_returns_err() {
    let temp_dir = TempDir::new().unwrap();
    let db_dir = temp_dir.path().join("db");
    std::fs::create_dir_all(&db_dir).unwrap();

    let storage = LsmStorage::open(LsmConfig {
        path: db_dir.clone(),
        ..Default::default()
    }).await.unwrap();
    let runtime = setup_runtime(&db_dir);
    let registry = KeyFamilyRegistry::new();
    let sweep_source = DummySweepSource;

    let doc_id = DocId::new(42);
    let doc_str = "1/42";
    let footprint = ErasureFootprint::fuer_document(contextra_types::TenantId::SYSTEM, "test", doc_id);
    let opts = ForgetOptions::default();

    let res = forget_document(
        &storage,
        &runtime,
        doc_str,
        TxId(1),
        &opts,
        &sweep_source,
        &footprint,
        &registry,
        vec![],
    )
    .await;

    assert!(res.is_err(), "Expected error when mandatory injected witnesses are missing");
    if let Err(ContextraError::InvalidInput(msg)) = res {
        assert!(msg.contains("Missing required layer witness"), "Unexpected error message: {msg}");
    } else {
        panic!("Expected InvalidInput error, got {:?}", res);
    }
}

#[tokio::test]
async fn invalid_doc_id_returns_err_never_doc_id_zero() {
    let temp_dir = TempDir::new().unwrap();
    let db_dir = temp_dir.path().join("db");
    std::fs::create_dir_all(&db_dir).unwrap();

    let storage = LsmStorage::open(LsmConfig {
        path: db_dir.clone(),
        ..Default::default()
    }).await.unwrap();
    let runtime = setup_runtime(&db_dir);
    let registry = KeyFamilyRegistry::new();
    let sweep_source = DummySweepSource;

    let footprint = ErasureFootprint::fuer_document(contextra_types::TenantId::SYSTEM, "test", DocId::new(1));
    let opts = ForgetOptions::default();

    let res = forget_document(
        &storage,
        &runtime,
        "invalid_doc_id_format",
        TxId(1),
        &opts,
        &sweep_source,
        &footprint,
        &registry,
        vec![],
    )
    .await;

    assert!(res.is_err(), "Expected error on malformed doc ID");
    if let Err(ContextraError::InvalidInput(msg)) = res {
        assert!(msg.contains("Invalid document id"), "Unexpected error message: {msg}");
    } else {
        panic!("Expected InvalidInput error for malformed doc ID, got {:?}", res);
    }
}

#[tokio::test]
async fn issue_witnesses_v2_error_propagated() {
    let temp_dir = TempDir::new().unwrap();
    let db_dir = temp_dir.path().join("db");
    std::fs::create_dir_all(&db_dir).unwrap();

    let storage = LsmStorage::open(LsmConfig {
        path: db_dir.clone(),
        ..Default::default()
    }).await.unwrap();
    let runtime = setup_runtime(&db_dir);
    let registry = KeyFamilyRegistry::new();
    let sweep_source = DummySweepSource;

    let doc_id = DocId::new(100);
    let doc_str = "1/100";
    let footprint = ErasureFootprint::fuer_document(contextra_types::TenantId::SYSTEM, "test", doc_id);
    let opts = ForgetOptions::default();

    // Corrupt pending_delete.journal to cause store scan failure in issue_witnesses_v2
    std::fs::write(db_dir.join("pending_delete.journal"), b"CORRUPTED_JOURNAL_DATA").unwrap();

    let scope_hash = hash_deleted_keys_length_prefixed(&footprint.keys.explicit);
    let footprint_hash = footprint.hash;
    let injected = create_full_witness_set(doc_id, scope_hash, footprint_hash, TxId(1), &opts).unwrap();

    let res = forget_document(
        &storage,
        &runtime,
        doc_str,
        TxId(1),
        &opts,
        &sweep_source,
        &footprint,
        &registry,
        injected,
    )
    .await;

    assert!(res.is_err(), "issue_witnesses_v2 error must be propagated");
}

#[tokio::test]
async fn store_with_residual_version_returns_err() {
    let temp_dir = TempDir::new().unwrap();
    let db_dir = temp_dir.path().join("db");
    std::fs::create_dir_all(&db_dir).unwrap();

    let storage = LsmStorage::open(LsmConfig {
        path: db_dir.clone(),
        ..Default::default()
    }).await.unwrap();
    let runtime = setup_runtime(&db_dir);
    let registry = KeyFamilyRegistry::new();
    let sweep_source = DummySweepSource;

    let doc_id = DocId::new(101);
    let doc_str = "1/101";
    let footprint = ErasureFootprint::fuer_document(contextra_types::TenantId::SYSTEM, "test", doc_id);

    // Put a residual active record into storage for this document key
    if let Some(key) = footprint.keys.explicit.first() {
        storage.put(key.clone(), b"RESIDUAL_RECORD_DATA".to_vec(), TxId(1)).await.unwrap();
    }

    let opts = ForgetOptions::default();
    let scope_hash = hash_deleted_keys_length_prefixed(&footprint.keys.explicit);
    let footprint_hash = footprint.hash;
    let injected = create_full_witness_set(doc_id, scope_hash, footprint_hash, TxId(1), &opts).unwrap();

    let res = forget_document(
        &storage,
        &runtime,
        doc_str,
        TxId(1),
        &opts,
        &sweep_source,
        &footprint,
        &registry,
        injected,
    )
    .await;

    assert!(res.is_err(), "Expected error when residual version exists in store");
}

#[test]
fn ratchet_test_unmeasured_evidence_literals_removed() {
    let purge_driver_code = include_str!("../src/deletion/purge_driver.rs");

    assert!(
        !purge_driver_code.contains("marked_zeroed: true"),
        "Ratchet failed: `marked_zeroed: true` literal still exists in purge_driver.rs"
    );

    assert!(
        !purge_driver_code.contains("entries_found: 0"),
        "Ratchet failed: `entries_found: 0` literal still exists in purge_driver.rs"
    );
}
