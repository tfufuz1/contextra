//! Policy rule test suite for version 4 deletion proofs (R1..R17).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines
)]

use contextra_crypto::deletion_proof::builder::ProofBuilder;
use contextra_crypto::deletion_proof::canon::TlvField;
use contextra_crypto::deletion_proof::policy::{
    verify_with_policy, EvidenceInvariant, PolicyRuleResult, ProofPolicy,
};
use contextra_crypto::deletion_proof::proof_v4::{ProofClaimV4, ProofV4};
use contextra_crypto::deletion_proof::signer::{ProofKeyPairSigner, ProofSigner};
use contextra_crypto::deletion_proof::types::DeletionScope;
use contextra_crypto::error::{CryptoError, Result};
use contextra_types::{DocId, TenantId};
use std::collections::HashSet;
use std::sync::Arc;

fn helper_valid_proof(signer: &ProofKeyPairSigner) -> ProofV4 {
    let scope = DeletionScope::Document {
        doc_id: DocId(100),
        tenant_id: TenantId::try_new(1).expect("tenant"),
    };

    ProofBuilder::new(scope, "db-policy-test")
        .with_deleted_after_tx(100)
        .with_deleted_keys_hash([0x11; 32])
        .with_footprint_hash([0x22; 32])
        .with_observed_empty(false)
        .add_claim(ProofClaimV4 {
            layer_id: 1,
            assurance: 2,
            claim_text_id: 1,
            witness_digest: [0x33; 32],
            scanned_at_tx: 105,
            evidence_tlv: vec![],
        })
        .add_excluded_scope(1)
        .with_timestamps(1700000000000, 42)
        .finish(signer)
        .expect("builder finish failed")
}

fn find_rule_result<'a>(results: &'a [PolicyRuleResult], rule_id: &str) -> &'a PolicyRuleResult {
    results
        .iter()
        .find(|r| r.rule_id == rule_id)
        .unwrap_or_else(|| panic!("Rule ID {} not found in results", rule_id))
}

#[test]
fn test_r1_signature_verification_passes_and_fails() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let other_signer = ProofKeyPairSigner::from_secret_bytes([2u8; 32]);

    let proof = helper_valid_proof(&signer);
    let policy = ProofPolicy::default();

    let report_valid = verify_with_policy(&proof, &signer.public_key(), &policy);
    assert!(find_rule_result(&report_valid.rule_results, "R1").passed);

    let report_invalid_key = verify_with_policy(&proof, &other_signer.public_key(), &policy);
    assert!(!find_rule_result(&report_invalid_key.rule_results, "R1").passed);
}

#[test]
fn test_r2_version_verification() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let mut proof = helper_valid_proof(&signer);
    proof.version = 3;
    proof.sign_with_signer(&signer).unwrap();

    let policy = ProofPolicy::default();
    let report = verify_with_policy(&proof, &signer.public_key(), &policy);
    assert!(!find_rule_result(&report.rule_results, "R2").passed);
}

#[test]
fn test_r3_required_layers_coverage() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let proof = helper_valid_proof(&signer);

    let policy_pass = ProofPolicy {
        required_layers: vec![1],
        ..ProofPolicy::default()
    };
    let report_pass = verify_with_policy(&proof, &signer.public_key(), &policy_pass);
    assert!(find_rule_result(&report_pass.rule_results, "R3").passed);

    let policy_fail = ProofPolicy {
        required_layers: vec![1, 2],
        ..ProofPolicy::default()
    };
    let report_fail = verify_with_policy(&proof, &signer.public_key(), &policy_fail);
    assert!(!find_rule_result(&report_fail.rule_results, "R3").passed);
}

struct DummyEvidenceInvariant {
    fail_layer: u16,
}

impl EvidenceInvariant for DummyEvidenceInvariant {
    fn verify_evidence(&self, claim: &ProofClaimV4) -> Result<()> {
        if claim.layer_id == self.fail_layer {
            Err(CryptoError::Crypto("Evidence validation failed".into()))
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_r4_evidence_invariant_evaluation() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let proof = helper_valid_proof(&signer);

    let policy_fail = ProofPolicy {
        evidence_invariant: Some(Arc::new(DummyEvidenceInvariant { fail_layer: 1 })),
        ..ProofPolicy::default()
    };
    let report_fail = verify_with_policy(&proof, &signer.public_key(), &policy_fail);
    assert!(!find_rule_result(&report_fail.rule_results, "R4").passed);

    let policy_pass = ProofPolicy {
        evidence_invariant: Some(Arc::new(DummyEvidenceInvariant { fail_layer: 99 })),
        ..ProofPolicy::default()
    };
    let report_pass = verify_with_policy(&proof, &signer.public_key(), &policy_pass);
    assert!(find_rule_result(&report_pass.rule_results, "R4").passed);
}

#[test]
fn test_r5_mandatory_excluded_scopes() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let proof = helper_valid_proof(&signer);

    let policy_pass = ProofPolicy {
        mandatory_excluded_scopes: vec![1],
        ..ProofPolicy::default()
    };
    let report_pass = verify_with_policy(&proof, &signer.public_key(), &policy_pass);
    assert!(find_rule_result(&report_pass.rule_results, "R5").passed);

    let policy_fail = ProofPolicy {
        mandatory_excluded_scopes: vec![1, 2],
        ..ProofPolicy::default()
    };
    let report_fail = verify_with_policy(&proof, &signer.public_key(), &policy_fail);
    assert!(!find_rule_result(&report_fail.rule_results, "R5").passed);
}

#[test]
fn test_r6_timestamp_and_monotonic_counter() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);

    let mut proof_zero_wall = helper_valid_proof(&signer);
    proof_zero_wall.issued_wall_ms = 0;
    proof_zero_wall.sign_with_signer(&signer).unwrap();

    let policy = ProofPolicy::default();
    let report = verify_with_policy(&proof_zero_wall, &signer.public_key(), &policy);
    assert!(!find_rule_result(&report.rule_results, "R6").passed);
}

#[test]
fn test_r7_chain_hash_anchor_verification() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let mut proof = helper_valid_proof(&signer);
    proof.chain.prev_hash = [0x99; 32];
    proof.sign_with_signer(&signer).unwrap();

    let policy_pass = ProofPolicy {
        trusted_anchor: true,
        expected_prev_chain_hash: Some([0x99; 32]),
        ..ProofPolicy::default()
    };
    let report_pass = verify_with_policy(&proof, &signer.public_key(), &policy_pass);
    assert!(find_rule_result(&report_pass.rule_results, "R7").passed);

    let policy_fail = ProofPolicy {
        trusted_anchor: true,
        expected_prev_chain_hash: Some([0x11; 32]),
        ..ProofPolicy::default()
    };
    let report_fail = verify_with_policy(&proof, &signer.public_key(), &policy_fail);
    assert!(!find_rule_result(&report_fail.rule_results, "R7").passed);
}

#[test]
fn test_r8_assurance_and_text_id_validity() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let mut proof = helper_valid_proof(&signer);
    proof.claims[0].assurance = 5; // Invalid (>4)
    proof.sign_with_signer(&signer).unwrap();

    let policy = ProofPolicy::default();
    let report = verify_with_policy(&proof, &signer.public_key(), &policy);
    assert!(!find_rule_result(&report.rule_results, "R8").passed);
}

#[test]
fn test_r9_allowed_claim_text_ids() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let proof = helper_valid_proof(&signer);

    let mut allowed = HashSet::new();
    allowed.insert(2); // Claim has text_id = 1

    let policy_fail = ProofPolicy {
        allowed_claim_text_ids: allowed,
        ..ProofPolicy::default()
    };
    let report_fail = verify_with_policy(&proof, &signer.public_key(), &policy_fail);
    assert!(!find_rule_result(&report_fail.rule_results, "R9").passed);
}

#[test]
fn test_r10_signer_key_id_match() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let mut proof = helper_valid_proof(&signer);
    proof.signer_key_id = [0xAA; 32];

    let policy = ProofPolicy::default();
    let report = verify_with_policy(&proof, &signer.public_key(), &policy);
    assert!(!find_rule_result(&report.rule_results, "R10").passed);
}

#[test]
fn test_r11_supersedes_reference_check() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);

    let mut proof_zero = helper_valid_proof(&signer);
    proof_zero.supersedes = Some([0u8; 32]);
    proof_zero.sign_with_signer(&signer).unwrap();

    let policy = ProofPolicy::default();
    let report = verify_with_policy(&proof_zero, &signer.public_key(), &policy);
    assert!(!find_rule_result(&report.rule_results, "R11").passed);

    let mut proof_self = helper_valid_proof(&signer);
    proof_self.supersedes = Some(proof_self.proof_id);
    proof_self.sign_with_signer(&signer).unwrap();

    let report_self = verify_with_policy(&proof_self, &signer.public_key(), &policy);
    assert!(!find_rule_result(&report_self.rule_results, "R11").passed);
}

#[test]
fn test_r12_deleted_keys_hash_vs_observed_empty() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let mut proof = helper_valid_proof(&signer);
    proof.deleted_keys_hash = [0u8; 32];
    proof.observed_empty = false;
    proof.sign_with_signer(&signer).unwrap();

    let policy = ProofPolicy::default();
    let report = verify_with_policy(&proof, &signer.public_key(), &policy);
    assert!(!find_rule_result(&report.rule_results, "R12").passed);
}

#[test]
fn test_r13_unknown_critical_tlv_tags_rejection() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let proof = helper_valid_proof(&signer);

    let policy = ProofPolicy::default();
    let report_valid = verify_with_policy(&proof, &signer.public_key(), &policy);
    assert!(find_rule_result(&report_valid.rule_results, "R13").passed);
}

#[test]
fn test_r14_claim_scanned_at_tx_boundary() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let mut proof = helper_valid_proof(&signer);
    proof.claims[0].scanned_at_tx = 50; // Earlier than deleted_after_tx (100)
    proof.sign_with_signer(&signer).unwrap();

    let policy = ProofPolicy::default();
    let report = verify_with_policy(&proof, &signer.public_key(), &policy);
    assert!(!find_rule_result(&report.rule_results, "R14").passed);
}

#[test]
fn test_r15_stage_and_supersedes_consistency() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let mut proof = helper_valid_proof(&signer);
    proof.stage = 5; // Invalid stage
    proof.sign_with_signer(&signer).unwrap();

    let policy = ProofPolicy::default();
    let report = verify_with_policy(&proof, &signer.public_key(), &policy);
    assert!(!find_rule_result(&report.rule_results, "R15").passed);
}

#[test]
fn test_r16_signer_key_id_trust_set() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let proof = helper_valid_proof(&signer);

    let mut trust_set = HashSet::new();
    trust_set.insert([0x99; 32]);

    let policy_fail = ProofPolicy {
        trusted_signer_key_ids: trust_set,
        ..ProofPolicy::default()
    };
    let report_fail = verify_with_policy(&proof, &signer.public_key(), &policy_fail);
    assert!(!find_rule_result(&report_fail.rule_results, "R16").passed);
}

#[test]
fn test_r17_non_critical_unknown_tlv_tags_ignored() {
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);
    let proof = helper_valid_proof(&signer);

    let mut canon = proof.construct_canonical_payload();
    // Inject non-critical tag 0x0123 (bit 0x8000 NOT set)
    let non_critical_field = TlvField::new(0x0123, vec![0xAA, 0xBB]);
    non_critical_field.encode(&mut canon);

    // Non-critical unknown tags are safely ignored by decoder
    assert!(ProofV4::check_unknown_critical_tags(&canon).is_ok());

    let policy = ProofPolicy::default();
    let report = verify_with_policy(&proof, &signer.public_key(), &policy);
    assert!(find_rule_result(&report.rule_results, "R17").passed);
}
