//! Tests for Stage 6: Standalone Verifier (w1_06_*).

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::signer::ProofKeyPairSigner;
use contextra_crypto::deletion_proof::{ProofBuilder, ProofClaimV4, ProofSigner};
use contextra_crypto::{
    verify_standalone, DeletionScope, KeyClass, TrustList, VerifyError,
};
use contextra_types::{DocId, TenantId};
use ed25519_dalek::VerifyingKey;
use rand::rngs::OsRng;

#[test]
fn w1_06_verify_standalone_roundtrip() {
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let vk = VerifyingKey::from_bytes(&signer.public_key()).expect("vk");
    let key_id = signer.key_id();

    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: TenantId::try_new(1).expect("tenant"),
    };

    let proof = ProofBuilder::with_required(
        scope,
        "db-standalone-test",
        [0xAA; 32],
        [0xBB; 32],
        100,
        1700000000000,
    )
    .add_claim(ProofClaimV4 {
        layer_id: 1,
        assurance: 1,
        claim_text_id: 1,
        witness_digest: [0xCC; 32],
        scanned_at_tx: 100,
        evidence_tlv: vec![],
    })
    .finish(&signer)
    .expect("proof finish");

    // 1. Valid registration in TrustList -> verify_standalone succeeds with KeyClass::Current
    let mut trust_list = TrustList::new();
    trust_list.register_key(key_id, vk, 1000000000000, 2000000000000, KeyClass::Current);

    let verified = verify_standalone(&proof, &trust_list).expect("verify_standalone ok");
    assert_eq!(verified.key_class, KeyClass::Current);

    // 2. Untrusted key ID -> verify_standalone returns UntrustedKey
    let empty_trust_list = TrustList::new();
    let err_untrusted = verify_standalone(&proof, &empty_trust_list);
    assert!(matches!(err_untrusted, Err(VerifyError::UntrustedKey)));

    // 3. Revoked key ID -> verify_standalone returns RevokedKey
    let mut revoked_trust_list = TrustList::new();
    revoked_trust_list.register_key(key_id, vk, 1000000000000, 2000000000000, KeyClass::Revoked);
    let err_revoked = verify_standalone(&proof, &revoked_trust_list);
    assert!(matches!(err_revoked, Err(VerifyError::RevokedKey)));
}
