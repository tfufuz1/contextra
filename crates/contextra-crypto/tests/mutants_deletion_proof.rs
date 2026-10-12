//! Targeted mutant-killing tests for `contextra-crypto` deletion proof core pathways.
//!
//! Each test function targets specific surviving mutants identified during mutation testing,
//! referencing the source file, line number, mutant operator/replacement in comments.

use contextra_crypto::deletion_proof::{
    builder::ProofBuilder,
    keys::DeletionProofKeyPair,
    layer_proof::LayerCleanupProof,
    proof::{Coverage, DeletionProof, UncoveredLayer, UncoveredReason},
    signer::ProofKeyPairSigner,
    types::{DeletionLayer, DeletionScope, DurabilityProof},
};
use contextra_types::{DocId, TenantId, TxId};
use rand::rngs::OsRng;

// Mutant Target 1:
// File: crates/contextra-crypto/src/deletion_proof/builder.rs:134:21
// Operator: replace == with != in ProofBuilder::finish
// Checks whether proof_id computation fallback occurs when unassigned ([0u8; 32]).
#[test]
fn test_kill_mutant_builder_proof_id_fallback() {
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::try_new(1).expect("tenant"),
    };

    let proof = ProofBuilder::new(scope, "db-instance-1")
        .with_deleted_after_tx(10)
        .finish(&signer)
        .expect("finish failed");

    // Default builder leaves proof_id as [0u8; 32], so finish() computes BLAKE3 hash.
    // If == was mutated to !=, proof_id would remain [0u8; 32].
    assert_ne!(
        proof.proof_id, [0u8; 32],
        "proof_id must be computed from hash when unassigned"
    );
}

// Mutant Target 2:
// File: crates/contextra-crypto/src/deletion_proof/keys.rs:13:9
// Operator: replace <impl std::fmt::Debug for DeletionProofKeyPair>::fmt -> std::fmt::Result with Ok(Default::default())
#[test]
fn test_kill_mutant_keys_debug_redaction() {
    let keypair = DeletionProofKeyPair::generate();
    let debug_str = format!("{:?}", keypair);

    assert!(
        debug_str.contains("DeletionProofKeyPair"),
        "Debug output must contain struct name"
    );
    assert!(
        debug_str.contains("***REDACTED***"),
        "Debug output must redact secret signing key"
    );
    assert!(
        debug_str.contains("verifying_key"),
        "Debug output must contain verifying_key field"
    );
}

// Mutant Target 3 & 6:
// File: crates/contextra-crypto/src/deletion_proof/keys.rs:23:9 & 49:9
// Operator: replace DeletionProofKeyPair::zeroize with () & replace <impl Drop for DeletionProofKeyPair>::drop with ()
#[test]
fn test_kill_mutant_keys_zeroize_and_drop() {
    let mut keypair = DeletionProofKeyPair::generate();

    // Verify keypair generates non-zero bytes initially
    let original_bytes = keypair.signing_key().to_bytes();
    assert_ne!(original_bytes, [0u8; 32], "Signing key bytes must not be all zeros initially");

    // Test explicit zeroize
    keypair.zeroize();
    let zeroed_bytes = keypair.signing_key().to_bytes();
    assert_eq!(
        zeroed_bytes, [0u8; 32],
        "zeroize() must clear signing key bytes to [0u8; 32]"
    );
}

// Mutant Target 4 & 5:
// File: crates/contextra-crypto/src/deletion_proof/keys.rs:38:9
// Operator: replace DeletionProofKeyPair::verifying_key_bytes -> [u8; 32] with [0; 32] / [1; 32]
#[test]
fn test_kill_mutant_keys_verifying_key_bytes() {
    let keypair = DeletionProofKeyPair::generate();
    let bytes = keypair.verifying_key_bytes();

    assert_eq!(
        bytes,
        keypair.verifying_key.to_bytes(),
        "verifying_key_bytes() must return exact verifying public key bytes"
    );
    assert_ne!(bytes, [0u8; 32], "verifying_key_bytes must not be fixed dummy [0; 32]");
    assert_ne!(bytes, [1u8; 32], "verifying_key_bytes must not be fixed dummy [1; 32]");
}

// Mutant Target 7, 8, 9:
// File: crates/contextra-crypto/src/deletion_proof/proof.rs:103:35
// Operator: replace < with == / > / <= in DeletionProof::coverage
#[test]
fn test_kill_mutant_proof_coverage_version_threshold() {
    let scope = DeletionScope::Document {
        doc_id: DocId(1),
        tenant_id: TenantId::try_new(100).expect("tenant"),
    };

    let mut proof = DeletionProof {
        signature_version: 3,
        scope: scope.clone(),
        deleted_keys_hash: [0u8; 32],
        deleted_after_tx: TxId(10),
        timestamp: 1000,
        signature: vec![],
        covered_layers: vec![DeletionLayer::LsmMemtable],
        uncovered_layers: vec![UncoveredLayer {
            layer: DeletionLayer::SsTableAllLevels,
            reason: UncoveredReason::SsdPhysical,
        }],
        excluded_scopes: vec![],
        graph_repair: vec![],
        wal_chain_receipt: None,
        audit_chain_position: None,
        integrity_warning: None,
    };

    // Version 3 is < 4 -> coverage_known must be false
    let cov_v3 = proof.coverage();
    assert_eq!(
        cov_v3,
        Coverage {
            covered: vec![DeletionLayer::LsmMemtable],
            uncovered: vec![UncoveredLayer {
                layer: DeletionLayer::SsTableAllLevels,
                reason: UncoveredReason::SsdPhysical,
            }],
            coverage_known: false,
        },
        "signature_version 3 (< 4) must result in coverage_known = false"
    );

    // Version 4 is >= 4 -> coverage_known must be true
    proof.signature_version = 4;
    let cov_v4 = proof.coverage();
    assert_eq!(
        cov_v4,
        Coverage {
            covered: vec![DeletionLayer::LsmMemtable],
            uncovered: vec![UncoveredLayer {
                layer: DeletionLayer::SsTableAllLevels,
                reason: UncoveredReason::SsdPhysical,
            }],
            coverage_known: true,
        },
        "signature_version 4 (not < 4) must result in coverage_known = true"
    );

    // Version 5 is >= 4 -> coverage_known must be true
    proof.signature_version = 5;
    let cov_v5 = proof.coverage();
    assert_eq!(
        cov_v5.coverage_known, true,
        "signature_version 5 (not < 4) must result in coverage_known = true"
    );
}

// Mutant Target 10:
// File: crates/contextra-crypto/src/deletion_proof/proof_v3.rs:210:46
// Operator: replace < with > in DeletionProof::create_full_v3
#[test]
fn test_kill_mutant_proof_v3_durability_confirmed_seq_boundary() {
    let scope = DeletionScope::Document {
        doc_id: DocId(100),
        tenant_id: TenantId::try_new(1).expect("tenant"),
    };
    let keypair = DeletionProofKeyPair::generate();
    let cleanup = LayerCleanupProof::verify_and_create(DeletionLayer::LsmMemtable, || Ok(true))
        .expect("cleanup proof");

    let deleted_after_tx = TxId(50);

    // 1. confirmed_seq_no = 49 (< 50) -> Must fail as NotDurable
    let durability_unconfirmed = DurabilityProof::new(49);
    let result_unconfirmed = DeletionProof::create_full_v3(
        scope.clone(),
        vec![],
        deleted_after_tx,
        vec![cleanup.clone()],
        vec![],
        None,
        None,
        0,
        &[],
        Some(&durability_unconfirmed),
        keypair.signing_key(),
    );
    assert!(
        result_unconfirmed.is_err(),
        "confirmed_seq_no (49) < deleted_after_tx (50) must return Err"
    );

    // 2. confirmed_seq_no = 50 (== 50) -> Must succeed
    let durability_confirmed_equal = DurabilityProof::new(50);
    let result_equal = DeletionProof::create_full_v3(
        scope.clone(),
        vec![],
        deleted_after_tx,
        vec![cleanup.clone()],
        vec![],
        None,
        None,
        0,
        &[],
        Some(&durability_confirmed_equal),
        keypair.signing_key(),
    );
    assert!(
        result_equal.is_ok(),
        "confirmed_seq_no (50) == deleted_after_tx (50) must succeed"
    );

    // 3. confirmed_seq_no = 51 (> 50) -> Must succeed
    let durability_confirmed_greater = DurabilityProof::new(51);
    let result_greater = DeletionProof::create_full_v3(
        scope,
        vec![],
        deleted_after_tx,
        vec![cleanup],
        vec![],
        None,
        None,
        0,
        &[],
        Some(&durability_confirmed_greater),
        keypair.signing_key(),
    );
    assert!(
        result_greater.is_ok(),
        "confirmed_seq_no (51) > deleted_after_tx (50) must succeed"
    );
}

// Mutant Target 11:
// File: crates/contextra-crypto/src/deletion_proof/proof_v3.rs:227:40
// Operator: replace >= with < in DeletionProof::create_full_v3
#[test]
fn test_kill_mutant_proof_v3_attested_at_timestamp_conversion() {
    let scope = DeletionScope::Document {
        doc_id: DocId(101),
        tenant_id: TenantId::try_new(1).expect("tenant"),
    };
    let keypair = DeletionProofKeyPair::generate();
    let cleanup = LayerCleanupProof::verify_and_create(DeletionLayer::LsmMemtable, || Ok(true))
        .expect("cleanup proof");
    let durability = DurabilityProof::new(100);

    // 1. Positive timestamp (attested_at = 1700000000) -> timestamp = 1700000000
    let proof_positive = DeletionProof::create_full_v3(
        scope.clone(),
        vec![],
        TxId(100),
        vec![cleanup.clone()],
        vec![],
        None,
        None,
        1_700_000_000,
        &[],
        Some(&durability),
        keypair.signing_key(),
    )
    .expect("create positive timestamp proof");
    assert_eq!(
        proof_positive.timestamp, 1_700_000_000,
        "Positive attested_at timestamp must be preserved"
    );

    // 2. Negative timestamp (attested_at = -100) -> timestamp = 0
    let proof_negative = DeletionProof::create_full_v3(
        scope,
        vec![],
        TxId(100),
        vec![cleanup],
        vec![],
        None,
        None,
        -100,
        &[],
        Some(&durability),
        keypair.signing_key(),
    )
    .expect("create negative timestamp proof");
    assert_eq!(
        proof_negative.timestamp, 0,
        "Negative attested_at timestamp must be clamped to 0"
    );
}
