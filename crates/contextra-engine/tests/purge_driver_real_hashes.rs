// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Integrationstest für DpiPurgeDriver Stage 2 Stage-2-Löschnachweise: echte Hashes, Tenant-Prüfung und Fail-Closed-Verhalten bei Blockierung.
// INVARIANTEN: Stage 2 Hashes ungleich 0, Tenant-Zuordnung laut Scope/DocId, Blockierung bei Unimplemented.

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::probe::{WitStoreGeneration, WitStoreProbe};
use contextra_crypto::deletion_proof::types::{DeletionLayer, DeletionScope};
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_engine::deletion::{
    DpiDeletionRuntime, DpiLogicalInput, DpiPurgeDriver, DpiPurgeDriverConfig, FonLogicalHandle,
    ForgetOptions, PurgeHandle, PurgeMode,
};
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::LsmConfig;
use contextra_store::LsmStorage;
use contextra_types::{ContextraError, DocId, TenantId, TxId};
use tempfile::TempDir;

struct TestProbe;
impl WitStoreProbe for TestProbe {
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
async fn test_purge_driver_stage2_real_hashes_and_tenant() {
    let db_dir = TempDir::new().expect("tempdir failed");
    let signer_key_path = db_dir.path().join("proof_signer.key");
    let db_instance_id = [0x77; 16];

    let runtime = DpiDeletionRuntime::open(db_dir.path(), Some(&signer_key_path), db_instance_id)
        .expect("DpiDeletionRuntime::open failed");

    let config = LsmConfig {
        path: db_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config)
        .await
        .expect("LsmStorage::open failed");

    let deleted_keys_hash = [0x11; 32];
    let footprint_hash = [0x22; 32];

    let witnesses = vec![
        LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::LsmMemtable,
            scope_hash: deleted_keys_hash,
            scanned_at_tx: TxId::new(50),
            as_of_tx: TxId::new(50),
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
            scanned_at_tx: TxId::new(50),
            as_of_tx: TxId::new(50),
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
            scanned_at_tx: TxId::new(50),
            as_of_tx: TxId::new(50),
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
            scanned_at_tx: TxId::new(50),
            as_of_tx: TxId::new(50),
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
            scanned_at_tx: TxId::new(50),
            as_of_tx: TxId::new(50),
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

    let tenant_id = TenantId::try_new(42).expect("valid tenant_id");
    let doc_id = DocId::new(101);

    let input = DpiLogicalInput {
        scope: DeletionScope::Document { doc_id, tenant_id },
        storage_scope: "lsm".to_string(),
        deleted_keys_hash,
        footprint_hash,
        observed_empty: false,
        deleted_after_tx: TxId::new(50),
        witnesses,
        scope_file_names: vec!["42/101".to_string()],
    };

    let opts = ForgetOptions::new().with_assurance(WitAssurance::S1);
    let probe = TestProbe;
    let probes: &[&dyn WitStoreProbe] = &[&probe];

    let (logical_handle, _) = runtime
        .finalize_logical(input, &opts, None, probes)
        .expect("finalize_logical failed");

    let driver = DpiPurgeDriver::new(&storage, &runtime, DpiPurgeDriverConfig::default());
    let purge_scope = PurgeScope::Keys(vec![b"__col:test:\x00101".to_vec()]);

    let handle = driver
        .run_physical(
            purge_scope,
            &["42/101".to_string()],
            TxId::new(50),
            &logical_handle,
            &opts,
            None,
            probes,
            None,
        )
        .await
        .expect("run_physical failed");

    match handle {
        PurgeHandle::Pending { .. } => panic!("Expected Final handle, got Pending"),
        PurgeHandle::Final(phys_handle) => {
            let proof_data =
                std::fs::read(&phys_handle.proof_path).expect("Failed to read Stage 2 proof file");
            let proof_json: serde_json::Value =
                serde_json::from_slice(&proof_data).expect("Invalid JSON in proof file");

            let deleted_keys_hash_json = proof_json["deleted_keys_hash"]
                .as_array()
                .expect("deleted_keys_hash array");
            let footprint_hash_json = proof_json["footprint_hash"]
                .as_array()
                .expect("footprint_hash array");

            let deleted_keys_bytes: Vec<u8> = deleted_keys_hash_json
                .iter()
                .map(|v| v.as_u64().unwrap() as u8)
                .collect();
            let footprint_bytes: Vec<u8> = footprint_hash_json
                .iter()
                .map(|v| v.as_u64().unwrap() as u8)
                .collect();

            assert_ne!(
                deleted_keys_bytes,
                vec![0u8; 32],
                "deleted_keys_hash must not be zero"
            );
            assert_ne!(
                footprint_bytes,
                vec![0u8; 32],
                "footprint_hash must not be zero"
            );

            // Scope check: tenant_id must be 42
            let scope_val = &proof_json["scope"]["Document"];
            let tenant_val = scope_val["tenant_id"].as_u64().expect("tenant_id numeric");
            assert_eq!(
                tenant_val, 42,
                "Tenant ID in physical proof must match scope"
            );
        }
        PurgeHandle::Blocked { reason, .. } => {
            panic!("Expected PurgeHandle::Final, got Blocked: {}", reason);
        }
    }
}

#[tokio::test]
async fn test_run_physical_invalid_tenant_fails() {
    let db_dir = TempDir::new().expect("tempdir failed");
    let signer_key_path = db_dir.path().join("proof_signer.key");
    let db_instance_id = [0x77; 16];

    let runtime = DpiDeletionRuntime::open(db_dir.path(), Some(&signer_key_path), db_instance_id)
        .expect("DpiDeletionRuntime::open failed");

    let config = LsmConfig {
        path: db_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config)
        .await
        .expect("LsmStorage::open failed");

    let driver = DpiPurgeDriver::new(&storage, &runtime, DpiPurgeDriverConfig::default());

    let logical_handle = FonLogicalHandle {
        proof_id: [0x11; 16],
        proof_path: db_dir.path().join("proofs").join("01.json"),
        full_proof_id: [0x11; 32],
    };

    let opts = ForgetOptions::new().with_assurance(WitAssurance::S1);
    let probe = TestProbe;
    let probes: &[&dyn WitStoreProbe] = &[&probe];

    // Attempting to pass tenant 0 in doc_id_str format (e.g. "0/101") violates INV-TENANT-1 in parse_doc_id_and_tenant
    let res = driver
        .run_physical(
            PurgeScope::Keys(vec![b"k1".to_vec()]),
            &["0/101".to_string()],
            TxId::new(10),
            &logical_handle,
            &opts,
            None,
            probes,
            None,
        )
        .await;

    assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
}

#[tokio::test]
async fn test_run_physical_checked_validations() {
    let db_dir = TempDir::new().expect("tempdir failed");
    let signer_key_path = db_dir.path().join("proof_signer.key");
    let db_instance_id = [0x88; 16];

    let runtime = DpiDeletionRuntime::open(db_dir.path(), Some(&signer_key_path), db_instance_id)
        .expect("DpiDeletionRuntime::open failed");

    let config = LsmConfig {
        path: db_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config)
        .await
        .expect("LsmStorage::open failed");

    let driver = DpiPurgeDriver::new(&storage, &runtime, DpiPurgeDriverConfig::default());

    let logical_handle = FonLogicalHandle {
        proof_id: [0x11; 16],
        proof_path: db_dir.path().join("proofs").join("01.json"),
        full_proof_id: [0x11; 32],
    };

    let opts_scrub = ForgetOptions::new().with_purge(PurgeMode::PurgeWithScrub);
    let probe = TestProbe;
    let probes: &[&dyn WitStoreProbe] = &[&probe];

    // Empty scope_file_names -> InvalidInput
    let err_empty = driver
        .run_physical_checked(
            PurgeScope::Keys(vec![b"k1".to_vec()]),
            &[],
            TxId::new(10),
            &logical_handle,
            &opts_scrub,
            None,
            probes,
            None,
        )
        .await;

    assert!(matches!(err_empty, Err(ContextraError::InvalidInput(_))));

    // PurgeWithScrub without copy_registry -> InvalidInput
    let err_no_reg = driver
        .run_physical_checked(
            PurgeScope::Keys(vec![b"k1".to_vec()]),
            &["doc1".to_string()],
            TxId::new(10),
            &logical_handle,
            &opts_scrub,
            None,
            probes,
            None,
        )
        .await;

    assert!(matches!(err_no_reg, Err(ContextraError::InvalidInput(_))));
}
