// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Verifikation von strikten ProofPolicy-Defaults (Fail-Closed, CWE-636-Schutz) und Issuer-Layer-Exklusion unbelegter Layer.
// INVARIANTEN:
//   - INV-DELETION-1: Beweise ohne alle geforderten Pflicht-Layer werden abgetrennt/abgelehnt.
//   - Zero-Panic: Kein unwrap/expect in Produktionspfaden.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::exclusions::mandatory_ids;
use contextra_crypto::deletion_proof::issuer::{IssRequest, ProofIssuer};
use contextra_crypto::deletion_proof::policy::{verify_with_policy, PolicyRelaxed, ProofPolicy};
use contextra_crypto::deletion_proof::probe::{WitStoreGeneration, WitStoreProbe};
use contextra_crypto::deletion_proof::proof_v4::{ProofChainLink, ProofClaimV4, ProofV4};
use contextra_crypto::deletion_proof::signer::{ProofKeyPairSigner, ProofSigner};
use contextra_crypto::deletion_proof::types::{DeletionLayer, DeletionScope, ExcludedScope};
use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_types::{DocId, TenantId, TxId};
use rand::rngs::OsRng;
use std::sync::Arc;
use tempfile::tempdir;

struct TestProbe;

impl WitStoreProbe for TestProbe {
    fn current_generation(&self, _layer: &DeletionLayer) -> WitStoreGeneration {
        WitStoreGeneration(1)
    }

    fn scope_write_seq(&self, _scope_hash: &[u8; 32]) -> u64 {
        0
    }

    fn manifest_file_count(&self) -> Option<u32> {
        None
    }
}

fn create_valid_proof(signer: &ProofKeyPairSigner, claims: Vec<ProofClaimV4>) -> ProofV4 {
    let mut proof = ProofV4 {
        version: 4,
        proof_id: [10u8; 32],
        db_instance_id: "db-1".to_string(),
        stage: 1,
        supersedes: None,
        scope: DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        },
        storage_scope: "doc:1".to_string(),
        deleted_keys_hash: [20u8; 32],
        footprint_hash: [30u8; 32],
        observed_empty: false,
        deleted_after_tx: 10,
        claims,
        excluded_scopes: mandatory_ids(),
        limitations_text_ids: vec![],
        environment_tlv: vec![],
        wal_chain_receipt: None,
        chain: ProofChainLink {
            seq: 1,
            prev_hash: [0u8; 32],
        },
        issued_wall_ms: 1700000000000,
        issued_mono_ctr: 1,
        signer_key_id: signer.key_id(),
        signature: [0u8; 64],
    };
    proof.sign_with_signer(signer).expect("signing failed");
    proof
}

#[test]
fn test_default_policy_rejects_empty_proof_claims() {
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let empty_proof = create_valid_proof(&signer, vec![]);

    let default_policy = ProofPolicy::default();
    let report = verify_with_policy(&empty_proof, &signer.public_key(), &default_policy);

    assert!(
        !report.is_valid,
        "ProofPolicy::default() MUST reject proof with empty required layers (CWE-636)"
    );

    let r3 = report
        .rule_results
        .iter()
        .find(|r| r.rule_id == "R3")
        .expect("R3 rule result must exist");
    assert!(
        !r3.passed,
        "Rule R3 MUST fail when required layers are missing"
    );
}

#[test]
fn test_default_policy_accepts_proof_with_all_required_layers() {
    let signer = ProofKeyPairSigner::generate(&mut OsRng);

    // Default policy requires layers [1, 3, 20, 21, 26]
    let required = vec![1, 3, 20, 21, 26];
    let claims: Vec<ProofClaimV4> = required
        .into_iter()
        .map(|layer_id| ProofClaimV4 {
            layer_id,
            assurance: 1,
            claim_text_id: 1,
            witness_digest: [55u8; 32],
            scanned_at_tx: 10,
            evidence_tlv: vec![],
        })
        .collect();

    let valid_proof = create_valid_proof(&signer, claims);

    let default_policy = ProofPolicy::default();
    let report = verify_with_policy(&valid_proof, &signer.public_key(), &default_policy);

    assert!(
        report.is_valid,
        "ProofPolicy::default() MUST accept proof containing all required layers; report: {:?}",
        report
    );
}

#[test]
fn test_relaxed_policy_allows_empty_required_layers_for_tests() {
    let relaxed_policy = PolicyRelaxed::for_tests_only();

    assert!(
        relaxed_policy.required_layers.is_empty(),
        "PolicyRelaxed::for_tests_only MUST have empty required_layers"
    );

    let relaxed_policy_method = ProofPolicy::relaxed_for_tests_only();
    assert!(
        relaxed_policy_method.required_layers.is_empty(),
        "ProofPolicy::relaxed_for_tests_only MUST have empty required_layers"
    );
}

#[test]
fn test_issuer_excludes_unbacked_layer() {
    let tmp = tempdir().expect("tempdir failed");
    let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));
    let db_instance_id = [1u8; 16];

    let (issuer, _) =
        ProofIssuer::open(tmp.path(), signer.clone(), db_instance_id).expect("open issuer");

    let scope_hash = [11u8; 32];
    let footprint_hash = [22u8; 32];

    // Create a witness with valid evidence for Layer 1 (LSM Memtable)
    let witness_backed = LayerWitness::issue(WitIssueArgs {
        layer: DeletionLayer::LsmMemtable,
        assurance: WitAssurance::S0,
        claim: WitClaimText::S0,
        scanned_at_tx: TxId(15),
        as_of_tx: TxId(10),
        scope_hash,
        footprint_hash,
        store_generation: WitStoreGeneration(1),
        scope_write_seq: 0,
        scanner_name: "test",
        scanner_version: 1,
        layer_evidence: WitLayerEvidence::Memtable {
            memtables_scanned: 1,
            entries_found: 0,
            tombstones_found: 0,
        },
        key_identifiers_claimed: false,
    })
    .expect("create backed witness");

    let req = IssRequest {
        scope: DeletionScope::Document {
            doc_id: DocId(100),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        },
        storage_scope: "doc:100".to_string(),
        deleted_keys_hash: scope_hash,
        footprint_hash,
        observed_empty: false,
        deleted_after_tx: 10,
        stage: 1,
        supersedes: None,
        assurance: WitAssurance::S0,
        witnesses: vec![witness_backed],
        extra_exclusions: vec![ExcludedScope::UnregisteredBackups],
        environment_tlv: vec![],
        wal_chain_receipt: None,
    };

    let probe = TestProbe;
    let issued = issuer.issue(req, &[&probe]).expect("issuance must succeed");

    // The backed layer (1) must be in claims
    assert!(
        issued.proof.claims.iter().any(|c| c.layer_id == 1),
        "Backed layer 1 MUST be present in proof claims"
    );

    // Unbacked layers (e.g. 5) are not in claims and are present in excluded_scopes
    assert!(
        !issued.proof.claims.iter().any(|c| c.layer_id == 5),
        "Unbacked layer 5 MUST NOT be present in proof claims"
    );
}
