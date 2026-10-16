#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_license::{FeatureRing, SignedActivation};
use ed25519_dalek::SigningKey;

fn generate_deterministic_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let seed = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

#[test]
fn test_signature_covers_ring() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let hash = [1u8; 32];
    let expires = 2000i64;

    let mut act =
        SignedActivation::create_signed(FeatureRing::Sovereign, hash, expires, &signing_key);
    assert!(act.verify_signature(&verifying_key));

    // Alter ring field
    act.ring = FeatureRing::Compliance;
    assert!(!act.verify_signature(&verifying_key));
}

#[test]
fn test_signature_covers_installation_id_hash() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let hash = [1u8; 32];
    let expires = 2000i64;

    let mut act =
        SignedActivation::create_signed(FeatureRing::Sovereign, hash, expires, &signing_key);
    assert!(act.verify_signature(&verifying_key));

    // Alter installation_id_hash field
    act.installation_id_hash[0] ^= 0xFF;
    assert!(!act.verify_signature(&verifying_key));
}

#[test]
fn test_signature_covers_expires_at_unix() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let hash = [1u8; 32];
    let expires = 2000i64;

    let mut act =
        SignedActivation::create_signed(FeatureRing::Sovereign, hash, expires, &signing_key);
    assert!(act.verify_signature(&verifying_key));

    // Alter expires_at_unix field
    act.expires_at_unix += 1;
    assert!(!act.verify_signature(&verifying_key));
}
