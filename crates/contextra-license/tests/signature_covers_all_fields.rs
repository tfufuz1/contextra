#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_license::{FeatureRing, SignedActivation};
use ed25519_dalek::SigningKey;

fn fixed_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let secret_bytes = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

#[test]
fn test_signature_covers_all_three_fields() {
    let (signing_key, verifying_key) = fixed_keypair();

    let orig_ring = FeatureRing::Sovereign;
    let orig_hash = [0x55u8; 32];
    let orig_expires = 1_800_000_000i64;

    let valid_activation =
        SignedActivation::create_signed(orig_ring, orig_hash, orig_expires, &signing_key);

    // Baseline: unmodified activation verifies successfully
    assert!(valid_activation.verify_signature(&verifying_key));

    // 1. Single field mutation: ring
    let mut mutated_ring = valid_activation.clone();
    mutated_ring.ring = FeatureRing::Compliance;
    assert!(
        !mutated_ring.verify_signature(&verifying_key),
        "Mutating ring field must invalidate signature"
    );

    // 2. Single field mutation: installation_id_hash
    let mut mutated_hash = valid_activation.clone();
    mutated_hash.installation_id_hash[0] ^= 0x01;
    assert!(
        !mutated_hash.verify_signature(&verifying_key),
        "Mutating installation_id_hash field must invalidate signature"
    );

    // 3. Single field mutation: expires_at_unix
    let mut mutated_expires = valid_activation.clone();
    mutated_expires.expires_at_unix += 1;
    assert!(
        !mutated_expires.verify_signature(&verifying_key),
        "Mutating expires_at_unix field must invalidate signature"
    );
}
