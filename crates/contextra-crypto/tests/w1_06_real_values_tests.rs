//! Tests for Stage 3: Real Values (`footprint_hash`, `ProofBuilder`, `ZeuStoreProbe`) (w1_06_*).

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::signer::ProofKeyPairSigner;
use contextra_crypto::deletion_proof::ProofBuilder;
use contextra_crypto::{compute_witness_footprint_hash, DeletionScope};
use contextra_types::{DocId, TenantId};
use rand::rngs::OsRng;

#[test]
fn w1_06_witness_footprint_nonzero_and_deterministic() {
    let units = vec![b"key_a".to_vec(), b"key_b".to_vec()];

    let fp1 = compute_witness_footprint_hash(&units);
    let fp2 = compute_witness_footprint_hash(&units);

    // Must be non-zero
    assert_ne!(fp1, [0u8; 32]);
    // Must be deterministic
    assert_eq!(fp1, fp2);

    // Empty scan yields defined non-zero BLAKE3 hash of empty set
    let empty_units: Vec<Vec<u8>> = vec![];
    let empty_fp = compute_witness_footprint_hash(&empty_units);
    assert_ne!(empty_fp, [0u8; 32]);
    assert_ne!(fp1, empty_fp);
}

#[test]
fn w1_06_witness_footprint_differs_for_different_scope() {
    let units1 = vec![b"tenant_1/doc_10".to_vec()];
    let units2 = vec![b"tenant_2/doc_10".to_vec()];

    let fp1 = compute_witness_footprint_hash(&units1);
    let fp2 = compute_witness_footprint_hash(&units2);

    assert_ne!(fp1, fp2);
}

#[test]
fn w1_06_proof_timestamp_nonzero() {
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let scope = DeletionScope::Document {
        doc_id: DocId(1),
        tenant_id: TenantId::try_new(10).expect("tenant"),
    };

    // Constructing with timestamp = 0 MUST be rejected
    let builder_zero_ts = ProofBuilder::with_required(
        scope.clone(),
        "db-test",
        [1u8; 32],
        [2u8; 32],
        100,
        0, // timestamp == 0
    );

    let res = builder_zero_ts.finish(&signer);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("timestamp"));

    // Constructing with non-zero timestamp MUST succeed
    let builder_valid = ProofBuilder::with_required(
        scope,
        "db-test",
        [1u8; 32],
        [2u8; 32],
        100,
        1700000000000,
    );

    let proof = builder_valid.finish(&signer).expect("valid proof");
    assert_eq!(proof.issued_wall_ms, 1700000000000);
}
