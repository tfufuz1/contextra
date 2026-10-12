// FILE-CONTEXT
// ZWECK: Test for golden vectors of version 4 deletion proofs (byte-identity and verification).

use contextra_crypto::deletion_proof::signer::{ProofKeyPairSigner, ProofSigner};
use contextra_crypto::deletion_proof::{ProofBuilder, ProofV4};
use contextra_types::{DocId, TenantId};

#[test]
fn test_golden_v4_byte_identity() {
    let seed = [0x42u8; 32];
    let signer = ProofKeyPairSigner::from_secret_bytes(seed);
    let scope = contextra_crypto::DeletionScope::Document {
        doc_id: DocId(101),
        tenant_id: TenantId::try_new(1).expect("tenant"),
    };

    let proof = ProofBuilder::new(scope, "db-golden-instance-01")
        .with_stage(2)
        .with_storage_scope("lsm+hnsw+kv")
        .with_deleted_after_tx(100)
        .with_deleted_keys_hash([0xAA; 32])
        .with_footprint_hash([0xBB; 32])
        .with_observed_empty(true)
        .with_timestamps(1700000000000, 1)
        .finish(&signer)
        .expect("finish v4 proof");

    let computed_bin = proof.construct_canonical_payload();

    let golden_bin = include_bytes!("golden/p05_v4_valid.bin");
    let golden_json = include_str!("golden/p05_v4_valid.json");

    assert_eq!(
        computed_bin.as_slice(),
        golden_bin.as_slice(),
        "Canonical TLV encoding byte-identity mismatch with golden vector p05_v4_valid.bin"
    );

    let parsed_golden_proof: ProofV4 =
        serde_json::from_str(golden_json).expect("parse golden json");

    assert_eq!(
        proof, parsed_golden_proof,
        "Parsed ProofV4 from golden JSON must match generated ProofV4"
    );

    assert!(
        proof
            .verify_signature(&signer.public_key())
            .expect("verify signature"),
        "Golden proof signature must verify with deterministic signer key"
    );
}
