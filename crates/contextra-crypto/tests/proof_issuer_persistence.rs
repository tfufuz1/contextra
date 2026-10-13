//! Persistence, crash failpoints, idempotency and independent verifier tests for ProofIssuer v4.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::deletion_proof::{
    policy::{verify_with_policy, ProofPolicy},
    DeletionLayer, DeletionScope, IssRequest, ProofIssuer, ProofKeyPairSigner, ProofSigner,
    WitAssurance, WitClaimText, WitLayerEvidence, WitStoreGeneration, WitStoreProbe,
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

fn create_request(doc_id: u64, scope_hash: [u8; 32], tx: u64) -> IssRequest {
    let witness = create_valid_witness(DeletionLayer::LsmMemtable, scope_hash, tx);
    IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(doc_id),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        },
        storage_scope: format!("doc:{}", doc_id),
        deleted_keys_hash: scope_hash,
        footprint_hash: [0x33; 32],
        observed_empty: false,
        deleted_after_tx: tx,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness],
        extra_exclusions: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    }
}

#[test]
fn test_idempotent_reissuance_returns_existing_proof() {
    let dir = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let (issuer, _) = ProofIssuer::open(dir.path(), signer, [1u8; 16]).expect("open issuer");

    let scope_hash = [0x11; 32];
    let probe = MockStoreProbe {
        gen: 100,
        write_seq: 10,
    };
    let probe_ref: &dyn WitStoreProbe = &probe;

    let req1 = create_request(1, scope_hash, 50);
    let first_issued = issuer.issue(req1, &[probe_ref]).expect("first issue failed");

    // Second call with identical request parameters
    let req2 = create_request(1, scope_hash, 50);
    let second_issued = issuer.issue(req2, &[probe_ref]).expect("second issue failed");

    assert_eq!(
        first_issued.proof.proof_id, second_issued.proof.proof_id,
        "Idempotent reissuance MUST return identical proof_id"
    );
    assert_eq!(
        first_issued.chain_hash, second_issued.chain_hash,
        "Idempotent reissuance MUST return identical chain_hash"
    );
}

#[test]
fn test_issued_proof_passes_independent_verifier_and_policy() {
    let dir = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let public_key = signer.public_key();
    let (issuer, _) = ProofIssuer::open(dir.path(), signer, [1u8; 16]).expect("open issuer");

    let scope_hash = [0x11; 32];
    let probe = MockStoreProbe {
        gen: 100,
        write_seq: 10,
    };
    let probe_ref: &dyn WitStoreProbe = &probe;

    let req = create_request(1, scope_hash, 50);
    let issued = issuer.issue(req, &[probe_ref]).expect("issue failed");

    // Policy verification
    let policy = ProofPolicy {
        required_layers: vec![1],
        ..ProofPolicy::default()
    };
    let report = verify_with_policy(&issued.proof, &public_key, &policy);
    assert!(
        report.is_valid,
        "Issued proof MUST pass policy verification: {:?}",
        report
    );
}

#[test]
fn test_slightly_different_requests_never_produce_identical_signed_bytes() {
    let dir = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let (issuer, _) = ProofIssuer::open(dir.path(), signer, [1u8; 16]).expect("open issuer");

    let scope_hash1 = [0x11; 32];
    let scope_hash2 = [0x22; 32];
    let probe = MockStoreProbe {
        gen: 100,
        write_seq: 10,
    };
    let probe_ref: &dyn WitStoreProbe = &probe;

    let req1 = create_request(1, scope_hash1, 50);
    let req2 = create_request(2, scope_hash2, 50);

    let p1 = issuer.issue(req1, &[probe_ref]).expect("issue 1 failed");
    let p2 = issuer.issue(req2, &[probe_ref]).expect("issue 2 failed");

    assert_ne!(
        p1.proof.signed_bytes(),
        p2.proof.signed_bytes(),
        "Different requests MUST never produce identical signed bytes"
    );
}
