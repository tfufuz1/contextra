// FILE-CONTEXT
// ZWECK: Integration tests for external verification of Ed25519 deletion proofs (version 3).
// INVARIANTEN: Zero internal state required for verification. Injected deterministic RNGs only.

use contextra_crypto::ed25519_proof::{
    sign_deletion_proof_v3, verify_deletion_proof_v3, DeletionProofError, DeletionProofKeyPair,
};
use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};

#[test]
fn test_round_trip_external_verify() {
    let mut rng = StdRng::seed_from_u64(12345);
    let keypair = DeletionProofKeyPair::generate(&mut rng);

    let verifying_key_bytes = keypair.verifying_key_bytes();
    let payload = b"canonical_proof_payload_for_external_auditor_v3";

    let signature = sign_deletion_proof_v3(&keypair, payload);

    // Simulated third-party verification using ONLY verifying_key_bytes, payload, and signature
    let result = verify_deletion_proof_v3(&verifying_key_bytes, payload, &signature);
    assert!(
        result.is_ok(),
        "External verification must succeed for valid signature, payload, and verifying key"
    );
}

#[test]
fn test_negative_tampered_payload_fails_verification() {
    let mut rng = StdRng::seed_from_u64(54321);
    let keypair = DeletionProofKeyPair::generate(&mut rng);

    let verifying_key_bytes = keypair.verifying_key_bytes();
    let mut payload = b"canonical_proof_payload_for_external_auditor_v3".to_vec();

    let signature = sign_deletion_proof_v3(&keypair, &payload);

    // Tamper single byte in payload
    payload[0] ^= 0xFF;

    let result = verify_deletion_proof_v3(&verifying_key_bytes, &payload, &signature);
    assert!(
        matches!(result, Err(DeletionProofError::InvalidSignature(_))),
        "External verification must fail when payload is tampered"
    );
}

#[test]
fn test_negative_wrong_verifying_key_fails_verification() {
    let mut rng = StdRng::seed_from_u64(99999);
    let keypair_signer = DeletionProofKeyPair::generate(&mut rng);
    let keypair_other = DeletionProofKeyPair::generate(&mut rng);

    let wrong_verifying_key_bytes = keypair_other.verifying_key_bytes();
    let payload = b"canonical_proof_payload_for_external_auditor_v3";

    let signature = sign_deletion_proof_v3(&keypair_signer, payload);

    let result = verify_deletion_proof_v3(&wrong_verifying_key_bytes, payload, &signature);
    assert!(
        matches!(result, Err(DeletionProofError::InvalidSignature(_))),
        "External verification must fail when verified against incorrect public key"
    );
}

#[test]
fn test_property_like_random_payloads_with_injected_rng() {
    // Contextra-Konvention P28: Injizierte RNGs, KEIN rand::thread_rng()
    let mut rng = StdRng::seed_from_u64(0xDEADBEEF);
    let keypair = DeletionProofKeyPair::generate(&mut rng);
    let vk_bytes = keypair.verifying_key_bytes();

    for i in 0..10 {
        // Generate random length payload (between 16 and 256 bytes)
        let payload_len = rng.gen_range(16..=256);
        let mut payload = vec![0u8; payload_len];
        rng.fill_bytes(&mut payload);

        let signature = sign_deletion_proof_v3(&keypair, &payload);

        // Positive check
        assert!(
            verify_deletion_proof_v3(&vk_bytes, &payload, &signature).is_ok(),
            "Property test iteration {i} failed positive verification"
        );

        // Negative check: modify single byte
        let mut tampered_payload = payload.clone();
        let flip_idx = rng.gen_range(0..payload_len);
        tampered_payload[flip_idx] ^= 0x01;

        assert!(
            verify_deletion_proof_v3(&vk_bytes, &tampered_payload, &signature).is_err(),
            "Property test iteration {i} failed negative verification on tampered payload"
        );
    }
}
