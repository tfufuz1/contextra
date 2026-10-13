//! Integration test for two-stage proof issuance (§C3, §C7, ADR-14).

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::policy::{verify_with_policy, ProofPolicy};
use contextra_crypto::deletion_proof::probe::{WitStoreGeneration, WitStoreProbe};
use contextra_crypto::deletion_proof::types::{DeletionLayer, DeletionScope};
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_engine::deletion::{DpiDeletionRuntime, DpiLogicalInput, ForgetOptions};
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

#[test]
fn test_proof_issue_two_stage_logical_and_physical() {
    let db_dir = TempDir::new().expect("tempdir failed");
    let signer_key_path = db_dir.path().join("proof_signer.key");
    let db_instance_id = [0x11; 16];

    let runtime = DpiDeletionRuntime::open(db_dir.path(), Some(&signer_key_path), db_instance_id)
        .expect("DpiDeletionRuntime::open failed");

    let doc_id = DocId::new(42);
    let tenant_id = TenantId::SYSTEM;
    let scope = DeletionScope::Document { doc_id, tenant_id };

    let deleted_keys_hash = [0xaa; 32];
    let footprint_hash = [0xbb; 32];

    let witnesses = vec![
        LayerWitness::issue(WitIssueArgs {
            layer: DeletionLayer::LsmMemtable,
            scope_hash: deleted_keys_hash,
            scanned_at_tx: TxId::new(100),
            as_of_tx: TxId::new(100),
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
            scanned_at_tx: TxId::new(100),
            as_of_tx: TxId::new(100),
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
            scanned_at_tx: TxId::new(100),
            as_of_tx: TxId::new(100),
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
            scanned_at_tx: TxId::new(100),
            as_of_tx: TxId::new(100),
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
            scanned_at_tx: TxId::new(100),
            as_of_tx: TxId::new(100),
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
        scope,
        storage_scope: "lsm".to_string(),
        deleted_keys_hash,
        footprint_hash,
        observed_empty: false,
        deleted_after_tx: TxId::new(100),
        witnesses,
        scope_file_names: vec!["000001.sst".to_string()],
    };

    let opts = ForgetOptions::new().with_assurance(WitAssurance::S1);
    let dummy_probe = DummyProbe;
    let probes: &[&dyn WitStoreProbe] = &[&dummy_probe];

    let (_logical_handle, issued_proof) = runtime
        .finalize_logical(input, &opts, None, probes)
        .expect("finalize_logical failed");

    assert_eq!(issued_proof.proof.stage, 1);
    assert_eq!(issued_proof.proof.deleted_after_tx, 100);

    // Verify stage 1 policy S1 passes
    let policy_s1 = ProofPolicy {
        required_layers: vec![1, 3, 20, 21, 26],
        ..Default::default()
    };

    let pubkey = runtime.signer().public_key();
    let report_s1 = verify_with_policy(&issued_proof.proof, &pubkey, &policy_s1);
    assert!(
        report_s1.is_valid,
        "Stage 1 proof must pass policy S1: {:?}",
        report_s1
    );
}
