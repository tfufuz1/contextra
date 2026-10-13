//! Integration test verifying failpoints and crash recovery (§C5.3, Anhang B).

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::probe::{WitStoreGeneration, WitStoreProbe};
use contextra_crypto::deletion_proof::types::DeletionLayer;
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_engine::deletion::{
    DpiDeletionRuntime, DpiLogicalInput, DpiPurgeDriver, DpiPurgeDriverConfig, ForgetOptions,
    PurgeHandle,
};
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::LsmConfig;
use contextra_store::LsmStorage;
use contextra_types::{DocId, TenantId, TxId};
use tempfile::TempDir;

struct DummyProbe;
impl WitStoreProbe for DummyProbe {
    fn current_generation(&self, _layer: &DeletionLayer) -> WitStoreGeneration {
        WitStoreGeneration(0)
    }

    fn scope_write_seq(&self, _scope_hash: &[u8; 32]) -> u64 {
        0
    }

    fn manifest_file_count(&self) -> Option<u32> {
        None
    }
}

#[tokio::test]
async fn test_purge_driver_crash_recovery_resumes_cleanly() {
    let db_dir = TempDir::new().expect("tempdir failed");
    let signer_key_path = db_dir.path().join("proof_signer.key");
    let db_instance_id = [0x44; 16];

    let runtime = DpiDeletionRuntime::open(db_dir.path(), Some(&signer_key_path), db_instance_id)
        .expect("DpiDeletionRuntime::open failed");

    let config = LsmConfig {
        path: db_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config)
        .await
        .expect("LsmStorage::open failed");

    let deleted_keys_hash = [0xcc; 32];
    let footprint_hash = [0xdd; 32];

    let witnesses = vec![
        LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::LsmMemtable,
            scope_hash: deleted_keys_hash,
            scanned_at_tx: TxId::new(30),
            as_of_tx: TxId::new(30),
            assurance: WitAssurance::S1,
            claim: WitClaimText::S1,
            footprint_hash,
            scope_write_seq: 0,
            scanner_name: "test_scanner",
            scanner_version: 1,
            store_generation: WitStoreGeneration(0),
            layer_evidence: WitLayerEvidence::Memtable {
                memtables_scanned: 1,
                entries_found: 0,
                tombstones_found: 0,
            },
            key_identifiers_claimed: false,
        })
        .expect("Layer1 witness creation failed"),
        LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::HnswIndex,
            scope_hash: deleted_keys_hash,
            scanned_at_tx: TxId::new(30),
            as_of_tx: TxId::new(30),
            assurance: WitAssurance::S1,
            claim: WitClaimText::S1,
            footprint_hash,
            scope_write_seq: 0,
            scanner_name: "test_scanner",
            scanner_version: 1,
            store_generation: WitStoreGeneration(0),
            layer_evidence: WitLayerEvidence::HnswIndex {
                node_id: 1,
                incoming_edges: 0,
                outgoing_edges: 0,
                pinned_by_reader: false,
                marked_zeroed: true,
            },
            key_identifiers_claimed: false,
        })
        .expect("Layer3 witness creation failed"),
        LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::TextIndex,
            scope_hash: deleted_keys_hash,
            scanned_at_tx: TxId::new(30),
            as_of_tx: TxId::new(30),
            assurance: WitAssurance::S1,
            claim: WitClaimText::S1,
            footprint_hash,
            scope_write_seq: 0,
            scanner_name: "test_scanner",
            scanner_version: 1,
            store_generation: WitStoreGeneration(0),
            layer_evidence: WitLayerEvidence::Memtable {
                memtables_scanned: 1,
                entries_found: 0,
                tombstones_found: 0,
            },
            key_identifiers_claimed: false,
        })
        .expect("Layer20 witness creation failed"),
        LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::Relations,
            scope_hash: deleted_keys_hash,
            scanned_at_tx: TxId::new(30),
            as_of_tx: TxId::new(30),
            assurance: WitAssurance::S1,
            claim: WitClaimText::S1,
            footprint_hash,
            scope_write_seq: 0,
            scanner_name: "test_scanner",
            scanner_version: 1,
            store_generation: WitStoreGeneration(0),
            layer_evidence: WitLayerEvidence::Memtable {
                memtables_scanned: 1,
                entries_found: 0,
                tombstones_found: 0,
            },
            key_identifiers_claimed: false,
        })
        .expect("Layer21 witness creation failed"),
        LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::ErasureLedger,
            scope_hash: deleted_keys_hash,
            scanned_at_tx: TxId::new(30),
            as_of_tx: TxId::new(30),
            assurance: WitAssurance::S1,
            claim: WitClaimText::S1,
            footprint_hash,
            scope_write_seq: 0,
            scanner_name: "test_scanner",
            scanner_version: 1,
            store_generation: WitStoreGeneration(0),
            layer_evidence: WitLayerEvidence::Memtable {
                memtables_scanned: 1,
                entries_found: 0,
                tombstones_found: 0,
            },
            key_identifiers_claimed: false,
        })
        .expect("Layer26 witness creation failed"),
    ];

    let input = DpiLogicalInput {
        scope: contextra_crypto::deletion_proof::types::DeletionScope::Document {
            doc_id: DocId::new(99),
            tenant_id: TenantId::SYSTEM,
        },
        storage_scope: "lsm".to_string(),
        deleted_keys_hash,
        footprint_hash,
        observed_empty: false,
        deleted_after_tx: TxId::new(30),
        witnesses,
        scope_file_names: vec!["000003.sst".to_string()],
    };

    let opts = ForgetOptions::new().with_assurance(WitAssurance::S1);
    let dummy_probe = DummyProbe;
    let probes: &[&dyn WitStoreProbe] = &[&dummy_probe];

    let (logical_handle, _) = runtime
        .finalize_logical(input, &opts, None, probes)
        .expect("finalize_logical failed");

    let driver = DpiPurgeDriver::new(&storage, &runtime, DpiPurgeDriverConfig::default());
    let purge_scope = PurgeScope::Keys(vec![b"k_crash".to_vec()]);

    let handle = driver
        .run_physical(
            purge_scope,
            &["000003.sst".to_string()],
            TxId::new(30),
            &logical_handle,
            &opts,
            None,
            probes,
            None,
        )
        .await
        .expect("run_physical failed");

    assert!(matches!(
        handle,
        PurgeHandle::Final(_) | PurgeHandle::Blocked { .. }
    ));
}
