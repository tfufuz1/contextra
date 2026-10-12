//! Policy verifier implementing rules R1 through R13 for version 4 deletion proofs (C7, G5).

use super::proof_v4::ProofClaimV4;
use super::proof_v4::ProofV4;
use crate::error::Result;
use std::collections::HashSet;
use std::sync::Arc;

/// Trait for custom layer evidence invariant verification (R4).
pub trait EvidenceInvariant: Send + Sync {
    /// Verifies evidence invariant for a given claim.
    fn verify_evidence(&self, claim: &ProofClaimV4) -> Result<()>;
}

/// Configuration policy for verifying version 4 deletion proofs.
#[derive(Clone)]
pub struct ProofPolicy {
    /// Allowed claim_text_ids (R9). Default contains CT-S0=1..CT-RAM-Z=8.
    pub allowed_claim_text_ids: HashSet<u16>,
    /// Mandatory excluded scopes required for this scope/stage (R5).
    pub mandatory_excluded_scopes: Vec<u16>,
    /// Required layer IDs that must be present in claims for this scope/stage (R3).
    pub required_layers: Vec<u16>,
    /// Evidence invariant verifier double/impl (R4).
    pub evidence_invariant: Option<Arc<dyn EvidenceInvariant>>,
    /// Whether the chain anchor is trusted for chain link verification (R7).
    pub trusted_anchor: bool,
    /// Expected previous chain hash for R7 verification.
    pub expected_prev_chain_hash: Option<[u8; 32]>,
}

impl Default for ProofPolicy {
    fn default() -> Self {
        let mut allowed = HashSet::new();
        for id in 1..=8 {
            allowed.insert(id);
        }
        Self {
            allowed_claim_text_ids: allowed,
            mandatory_excluded_scopes: Vec::new(),
            required_layers: Vec::new(),
            evidence_invariant: None,
            trusted_anchor: false,
            expected_prev_chain_hash: None,
        }
    }
}

/// Result of evaluating an individual policy rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyRuleResult {
    /// Identifier of rule (e.g. "R1", "R2", ..., "R13").
    pub rule_id: String,
    /// Whether the rule passed.
    pub passed: bool,
    /// Descriptive status or failure message.
    pub message: String,
}

/// Aggregate report returned by `verify_with_policy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyReport {
    /// Overall validity (true only if all evaluated rules passed).
    pub is_valid: bool,
    /// Per-rule evaluation breakdown.
    pub rule_results: Vec<PolicyRuleResult>,
    /// Verifier engine description.
    pub verifier_kind: String,
    /// Indicates if proof was a legacy proof version (v1-v3).
    pub is_legacy: bool,
}

/// Verifies a `ProofV4` against a `ProofPolicy` and public key, executing rules R1 through R13.
pub fn verify_with_policy(
    proof: &ProofV4,
    public_key: &[u8; 32],
    policy: &ProofPolicy,
) -> PolicyReport {
    let mut results = Vec::new();

    // R1: Ed25519 signature verification
    let r1_passed = proof.verify_signature(public_key).unwrap_or(false);
    results.push(PolicyRuleResult {
        rule_id: "R1".to_string(),
        passed: r1_passed,
        message: if r1_passed {
            "R1 PASS: Ed25519 signature valid over canonical domain tag and payload".to_string()
        } else {
            "R1 FAIL: Ed25519 signature verification failed".to_string()
        },
    });

    // R2: Version verification
    let r2_passed = proof.version == 4;
    results.push(PolicyRuleResult {
        rule_id: "R2".to_string(),
        passed: r2_passed,
        message: if r2_passed {
            "R2 PASS: Proof version is 4".to_string()
        } else {
            format!("R2 FAIL: Expected version 4, got {}", proof.version)
        },
    });

    // R3: Required layers coverage
    let claim_layer_ids: HashSet<u16> = proof.claims.iter().map(|c| c.layer_id).collect();
    let missing_layers: Vec<u16> = policy
        .required_layers
        .iter()
        .copied()
        .filter(|req| !claim_layer_ids.contains(req))
        .collect();
    let r3_passed = missing_layers.is_empty();
    results.push(PolicyRuleResult {
        rule_id: "R3".to_string(),
        passed: r3_passed,
        message: if r3_passed {
            "R3 PASS: All required layers present in claims".to_string()
        } else {
            format!("R3 FAIL: Missing required layers: {:?}", missing_layers)
        },
    });

    // R4: Evidence invariants
    let mut r4_passed = true;
    let mut r4_msg = "R4 PASS: All evidence invariants satisfied".to_string();
    if let Some(ref inv) = policy.evidence_invariant {
        for claim in &proof.claims {
            if let Err(e) = inv.verify_evidence(claim) {
                r4_passed = false;
                r4_msg = format!(
                    "R4 FAIL: Evidence invariant failed for layer {}: {}",
                    claim.layer_id, e
                );
                break;
            }
        }
    }
    results.push(PolicyRuleResult {
        rule_id: "R4".to_string(),
        passed: r4_passed,
        message: r4_msg,
    });

    // R5: Mandatory excluded scopes
    let excluded_set: HashSet<u16> = proof.excluded_scopes.iter().copied().collect();
    let missing_excl: Vec<u16> = policy
        .mandatory_excluded_scopes
        .iter()
        .copied()
        .filter(|req| !excluded_set.contains(req))
        .collect();
    let r5_passed = missing_excl.is_empty();
    results.push(PolicyRuleResult {
        rule_id: "R5".to_string(),
        passed: r5_passed,
        message: if r5_passed {
            "R5 PASS: Mandatory excluded scopes satisfied".to_string()
        } else {
            format!(
                "R5 FAIL: Missing mandatory excluded scopes: {:?}",
                missing_excl
            )
        },
    });

    // R6: Timestamp & monotonic counter validation
    let r6_passed = proof.issued_wall_ms > 0 && proof.issued_mono_ctr > 0;
    results.push(PolicyRuleResult {
        rule_id: "R6".to_string(),
        passed: r6_passed,
        message: if r6_passed {
            "R6 PASS: Wall timestamp and monotonic counter valid".to_string()
        } else {
            "R6 FAIL: Invalid timestamp or zero monotonic counter".to_string()
        },
    });

    // R7: Chain hash link verification (requires trusted_anchor = true)
    let mut r7_passed = true;
    let mut r7_msg = "R7 PASS: Chain link verification satisfied or skipped".to_string();
    if policy.trusted_anchor {
        if let Some(ref expected_prev) = policy.expected_prev_chain_hash {
            if proof.chain.prev_hash != *expected_prev {
                r7_passed = false;
                r7_msg = "R7 FAIL: Chain prev_hash mismatch with trusted anchor".to_string();
            }
        } else {
            r7_passed = false;
            r7_msg = "R7 FAIL: Trusted anchor required expected_prev_chain_hash".to_string();
        }
    }
    results.push(PolicyRuleResult {
        rule_id: "R7".to_string(),
        passed: r7_passed,
        message: r7_msg,
    });

    // R8: Assurance level & claim text ID validity
    let r8_passed = proof
        .claims
        .iter()
        .all(|c| c.assurance <= 4 && (1..=8).contains(&c.claim_text_id));
    results.push(PolicyRuleResult {
        rule_id: "R8".to_string(),
        passed: r8_passed,
        message: if r8_passed {
            "R8 PASS: All claims have valid assurance level S0..S4 and claim_text_id".to_string()
        } else {
            "R8 FAIL: Invalid assurance level or claim_text_id in claims".to_string()
        },
    });

    // R9: Allowed claim text IDs
    let unallowed_text_ids: Vec<u16> = proof
        .claims
        .iter()
        .map(|c| c.claim_text_id)
        .filter(|id| !policy.allowed_claim_text_ids.contains(id))
        .collect();
    let r9_passed = unallowed_text_ids.is_empty();
    results.push(PolicyRuleResult {
        rule_id: "R9".to_string(),
        passed: r9_passed,
        message: if r9_passed {
            "R9 PASS: All claim_text_ids allowed by policy".to_string()
        } else {
            format!(
                "R9 FAIL: Unallowed claim_text_ids: {:?}",
                unallowed_text_ids
            )
        },
    });

    // R10: Signer Key ID match
    let expected_signer_key_id = *blake3::hash(public_key).as_bytes();
    let r10_passed = proof.signer_key_id == expected_signer_key_id;
    results.push(PolicyRuleResult {
        rule_id: "R10".to_string(),
        passed: r10_passed,
        message: if r10_passed {
            "R10 PASS: Signer key ID matches BLAKE3 digest of public key".to_string()
        } else {
            "R10 FAIL: Signer key ID mismatch".to_string()
        },
    });

    // R11: Supersedes reference check
    let r11_passed = match proof.supersedes {
        Some(sup_id) => sup_id != [0u8; 32] && sup_id != proof.proof_id,
        None => true,
    };
    results.push(PolicyRuleResult {
        rule_id: "R11".to_string(),
        passed: r11_passed,
        message: if r11_passed {
            "R11 PASS: Supersedes reference valid".to_string()
        } else {
            "R11 FAIL: Invalid supersedes self-reference or zeroes".to_string()
        },
    });

    // R12: Deleted keys hash vs observed_empty
    let r12_passed = proof.deleted_keys_hash != [0u8; 32] || proof.observed_empty;
    results.push(PolicyRuleResult {
        rule_id: "R12".to_string(),
        passed: r12_passed,
        message: if r12_passed {
            "R12 PASS: Deleted keys hash non-zero or observed_empty is true".to_string()
        } else {
            "R12 FAIL: Zero deleted_keys_hash requires observed_empty == true".to_string()
        },
    });

    // R13: Unknown critical TLV tags check
    let canon = proof.construct_canonical_payload();
    let r13_passed = ProofV4::check_unknown_critical_tags(&canon).is_ok();
    results.push(PolicyRuleResult {
        rule_id: "R13".to_string(),
        passed: r13_passed,
        message: if r13_passed {
            "R13 PASS: No unknown critical TLV tags".to_string()
        } else {
            "R13 FAIL: Unknown critical TLV tag present".to_string()
        },
    });

    let overall_valid = results.iter().all(|r| r.passed);

    PolicyReport {
        is_valid: overall_valid,
        rule_results: results,
        verifier_kind: "v4_policy_verifier".to_string(),
        is_legacy: false,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::super::signer::ProofSigner;
    use super::super::types::DeletionScope;
    use super::*;
    use crate::deletion_proof::proof_v4::ProofChainLink;
    use crate::deletion_proof::signer::ProofKeyPairSigner;
    use contextra_types::{DocId, TenantId};
    use rand::rngs::OsRng;

    fn test_valid_proof(signer: &ProofKeyPairSigner) -> ProofV4 {
        let mut proof = ProofV4 {
            version: 4,
            proof_id: [10u8; 32],
            db_instance_id: "db-1".to_string(),
            stage: 2,
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
            claims: vec![ProofClaimV4 {
                layer_id: 1,
                assurance: 2,
                claim_text_id: 3,
                witness_digest: [40u8; 32],
                scanned_at_tx: 10,
                evidence_tlv: vec![],
            }],
            excluded_scopes: vec![1],
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
        proof.sign_with_signer(signer).expect("sign failed");
        proof
    }

    #[test]
    fn test_valid_proof_passes_policy_r1_to_r13() {
        let signer = ProofKeyPairSigner::generate(&mut OsRng);
        let proof = test_valid_proof(&signer);

        let policy = ProofPolicy {
            required_layers: vec![1],
            mandatory_excluded_scopes: vec![1],
            ..ProofPolicy::default()
        };

        let report = verify_with_policy(&proof, &signer.public_key(), &policy);
        assert!(
            report.is_valid,
            "Valid proof MUST pass all rules; report: {:?}",
            report
        );
        assert_eq!(report.rule_results.len(), 13);
    }

    #[test]
    fn test_empty_proof_passes_signature_check_fails_r3_r12() {
        let signer = ProofKeyPairSigner::generate(&mut OsRng);
        let mut empty_proof = test_valid_proof(&signer);
        empty_proof.claims.clear(); // No layers
        empty_proof.deleted_keys_hash = [0u8; 32];
        empty_proof.observed_empty = false;
        empty_proof.sign_with_signer(&signer).expect("sign failed");

        let policy = ProofPolicy {
            required_layers: vec![1],
            ..ProofPolicy::default()
        };

        let report = verify_with_policy(&empty_proof, &signer.public_key(), &policy);

        assert!(
            !report.is_valid,
            "Empty proof without layers/keys MUST fail policy verification"
        );

        let r1 = report
            .rule_results
            .iter()
            .find(|r| r.rule_id == "R1")
            .unwrap();
        assert!(r1.passed, "R1 signature check MUST pass");

        let r3 = report
            .rule_results
            .iter()
            .find(|r| r.rule_id == "R3")
            .unwrap();
        assert!(!r3.passed, "R3 layer coverage MUST fail");

        let r12 = report
            .rule_results
            .iter()
            .find(|r| r.rule_id == "R12")
            .unwrap();
        assert!(
            !r12.passed,
            "R12 empty key hash without observed_empty MUST fail"
        );
    }
}
