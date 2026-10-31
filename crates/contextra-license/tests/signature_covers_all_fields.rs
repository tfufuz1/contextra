//! Prüfung der kryptografischen Signaturabdeckung aller Felder in SignedActivation.
//! Verifiziert, dass Änderungen an ring, installation_id_hash oder expires_at_unix die Signatur ungültig machen.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_license::{FeatureRing, SignedActivation};
use ed25519_dalek::SigningKey;

fn generate_deterministic_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let seed = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

// ANCHOR[contextra-license:TEST-SIG-COVERS-RING]
// Prüft, dass Modifikationen am FeatureRing die Signaturprüfung brechen.
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

// ANCHOR[contextra-license:TEST-SIG-COVERS-INSTALLATION-ID]
// Prüft, dass Änderungen am installation_id_hash die Signaturprüfung ungültig machen.
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

// ANCHOR[contextra-license:TEST-SIG-COVERS-EXPIRES-AT]
// Prüft, dass Manipulationen am Ablaufzeitstempel die Signaturprüfung brechen.
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
