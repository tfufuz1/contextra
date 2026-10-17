//! Two-stage issuance tests for ProofIssuer v4 (ADR-14, Spec C3, C7).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionScope, IssRequest, ProofIssuer, ProofKeyPairSigner, WitAssurance,
    WitClaimText, WitLayerEvidence, WitStoreGeneration, WitStoreProbe,
};
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_types::{DocId, TenantId, TxId};
use rand::rngs::OsRng;
use std::sync::Arc;
use tempfile::tempdir;

struct MockStoreProbe {
    gen: u64,
    write_seq: u64,
}

impl WitStoreProbe for MockStoreProbe {
    fn current_generation(&self, _layer: &DeletionLayer) -> WitStoreGeneration {
        WitStoreGeneration(self.gen)
    }

    fn scope_write_seq(&self, _scope_hash: &[u8; 32]) -> u64 {
        self.write_seq
    }

    fn manifest_file_count(&self) -> Option<u32> {
        None
    }
}

fn create_valid_witness(layer: DeletionLayer, scope_hash: [u8; 32], tx: u64) -> LayerWitness {
    LayerWitness::issue(WitIssueArgs {
        layer,
        scope_hash,
        scanned_at_tx: TxId(tx),
        as_of_tx: TxId(tx),
        assurance: WitAssurance::S0,
        claim: WitClaimText::S0,
        footprint_hash: [0x33; 32],
        scope_write_seq: 10,
        scanner_name: "test_scanner",
        scanner_version: 1,
        store_generation: WitStoreGeneration(100),
        layer_evidence: WitLayerEvidence::Memtable {
            memtables_scanned: 1,
            entries_found: 0,
            tombstones_found: 0,
        },
        key_identifiers_claimed: false,
    })
    .expect("witness issue failed")
}

#[test]
fn test_physical_stage_requires_valid_logical_supersedes_reference() {
    let dir = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let (issuer, _) = ProofIssuer::open(dir.path(), signer, [1u8; 16]).expect("open issuer");

    let scope_hash = [0x11; 32];
    let probe = MockStoreProbe {
        gen: 100,
        write_seq: 10,
    };
    let probe_ref: &dyn WitStoreProbe = &probe;

    // 1. Issuing Physical stage without supersedes MUST fail
    let witness1 = create_valid_witness(DeletionLayer::LsmMemtable, scope_hash, 50);
    let req_no_sup = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        },
        storage_scope: "doc:1".to_string(),
        deleted_keys_hash: scope_hash,
        footprint_hash: [0x33; 32],
        observed_empty: false,
        deleted_after_tx: 50,
        stage: 2, // Physical
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness1],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };
    assert!(
        issuer.issue(req_no_sup, &[probe_ref]).is_err(),
        "Physical stage without supersedes MUST fail"
    );

    // 2. Issue valid Logical stage 1 proof
    let witness2 = create_valid_witness(DeletionLayer::LsmMemtable, scope_hash, 50);
    let req_logical = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        },
        storage_scope: "doc:1".to_string(),
        deleted_keys_hash: scope_hash,
        footprint_hash: [0x33; 32],
        observed_empty: false,
        deleted_after_tx: 50,
        stage: 1, // Logical
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness2],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let issued_logical = issuer
        .issue(req_logical, &[probe_ref])
        .expect("issue logical proof failed");

    // 3. Issuing Physical stage referencing the valid Logical proof MUST succeed
    let witness3 = create_valid_witness(DeletionLayer::LsmMemtable, scope_hash, 50);
    let req_physical = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        },
        storage_scope: "doc:1".to_string(),
        deleted_keys_hash: scope_hash,
        footprint_hash: [0x33; 32],
        observed_empty: false,
        deleted_after_tx: 50,
        stage: 2, // Physical
        supersedes: Some(issued_logical.proof.proof_id),
        assurance: WitAssurance::S0,
        witnesses: vec![witness3],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let issued_physical = issuer
        .issue(req_physical, &[probe_ref])
        .expect("issue physical proof failed");

    assert_eq!(issued_physical.proof.stage, 2);
    assert_eq!(
        issued_physical.proof.supersedes,
        Some(issued_logical.proof.proof_id)
    );
}
