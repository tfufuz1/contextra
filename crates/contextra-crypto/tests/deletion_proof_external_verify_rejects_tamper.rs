use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope,
    LayerCleanupProof,
};
use contextra_crypto::CryptoError;
use contextra_types::{DocId, TenantId, TxId};

#[test]
fn verify_external_accepts_valid_proof() {
    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(1).unwrap();
    let scope = DeletionScope::Document {
        doc_id: DocId::new(42),
        tenant_id,
    };
    let layer_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap();

    let proof = DeletionProof::create_v3(
        scope,
        vec![b"key1".to_vec(), b"key2".to_vec()],
        TxId::new(100),
        vec![layer_proof],
        vec![ExcludedScope::LlmParameterMemory],
        keypair.signing_key(),
    )
    .expect("Failed to create valid v3 proof");

    let result = proof.verify_external(&keypair.verifying_key);
    assert!(
        result.is_ok(),
        "Valid v3 DeletionProof MUST be accepted by verify_external"
    );
}

#[test]
fn verify_external_rejects_tampered_timestamp() {
    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(1).unwrap();
    let scope = DeletionScope::Document {
        doc_id: DocId::new(42),
        tenant_id,
    };
    let layer_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap();

    let mut proof = DeletionProof::create_v3(
        scope,
        vec![b"key1".to_vec()],
        TxId::new(100),
        vec![layer_proof],
        vec![ExcludedScope::LlmParameterMemory],
        keypair.signing_key(),
    )
    .expect("Failed to create v3 proof");

    // Tamper with timestamp post-signing
    proof.timestamp ^= 0xDEADBEEF;

    let result = proof.verify_external(&keypair.verifying_key);
    assert!(
        matches!(result, Err(CryptoError::InvalidProofSignature)),
        "Proof with tampered timestamp MUST return Err(CryptoError::InvalidProofSignature), got {:?}",
        result
    );
}

#[test]
fn verify_external_rejects_tampered_deleted_keys_hash() {
    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(1).unwrap();
    let scope = DeletionScope::Document {
        doc_id: DocId::new(42),
        tenant_id,
    };
    let layer_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap();

    let mut proof = DeletionProof::create_v3(
        scope,
        vec![b"key1".to_vec()],
        TxId::new(100),
        vec![layer_proof],
        vec![ExcludedScope::LlmParameterMemory],
        keypair.signing_key(),
    )
    .expect("Failed to create v3 proof");

    // Tamper with deleted_keys_hash post-signing
    proof.deleted_keys_hash[0] ^= 0xFF;

    let result = proof.verify_external(&keypair.verifying_key);
    assert!(
        matches!(result, Err(CryptoError::InvalidProofSignature)),
        "Proof with tampered deleted_keys_hash MUST return Err(CryptoError::InvalidProofSignature), got {:?}",
        result
    );
}

#[test]
fn verify_external_rejects_wrong_public_key() {
    let keypair1 = DeletionProofKeyPair::generate();
    let keypair2 = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(1).unwrap();
    let scope = DeletionScope::Tenant { tenant_id };

    let proof = DeletionProof::create_v3(
        scope,
        vec![b"key1".to_vec()],
        TxId::new(100),
        vec![],
        vec![],
        keypair1.signing_key(),
    )
    .expect("Failed to create v3 proof");

    // Attempt verification using keypair2's verifying key instead of keypair1's
    let result = proof.verify_external(&keypair2.verifying_key);
    assert!(
        matches!(result, Err(CryptoError::InvalidProofSignature)),
        "Verification with wrong public key MUST return Err(CryptoError::InvalidProofSignature), got {:?}",
        result
    );
}

#[test]
fn verify_external_rejects_legacy_hmac_proof() {
    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(1).unwrap();
    let scope = DeletionScope::Tenant { tenant_id };
    let hmac_key = vec![0u8; 32];

    // Create a v2 HMAC proof
    let proof_v2 = DeletionProof::create(
        scope,
        vec![b"key1".to_vec()],
        TxId::new(100),
        vec![],
        vec![],
        &hmac_key,
    )
    .expect("Failed to create v2 HMAC proof");

    assert_eq!(proof_v2.signature_version, 2);

    let result = proof_v2.verify_external(&keypair.verifying_key);
    assert!(
        matches!(result, Err(CryptoError::UnsupportedSignatureVersion(2))),
        "v2 HMAC proof MUST return UnsupportedSignatureVersion(2), got {:?}",
        result
    );
    assert!(
        !matches!(result, Err(CryptoError::InvalidProofSignature)),
        "v2 HMAC proof MUST NOT return InvalidProofSignature"
    );
}
