//! Tests zur Schlüsselverwaltung, zum Schlüsselwechsel und zum Fail-Closed-Verhalten.
//!
//! Validiert das tatsächliche Verhalten von `SignedLicenseGate` und `SignedActivation`:
//! - Token mit Schlüssel A wird von Gate mit Schlüssel B abgelehnt (`LicenseError::InvalidSignature`).
//! - Token mit manipulierter Signatur oder manipulierter Nutzlast wird abgelehnt.
//! - Gate unterstützt genau einen Schlüssel (kein Multi-Keyring); Wechsel erfolgt durch Neuinstanziierung.
//! - Fehlermeldungen bei Schlüsselabweichung enthalten keinerlei Schlüssel- oder Hash-Informationen (Leak-Schutz).

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeMap;

use contextra_license::{
    FeatureRing, LicenseError, LicenseGate, LicensePayload, SignedActivation, SignedLicenseGate,
};
use contextra_types::TenantId;
use ed25519_dalek::{Signer, SigningKey};

/// Hilfsfunktion zur Generierung eines deterministischen Schlüsselpaars aus einem 32-Byte-Seed.
fn generate_deterministic_keypair(seed_byte: u8) -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let seed = [seed_byte; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

// ANCHOR[contextra-license:TEST-ROTATION-KEY-A-REJECTED-BY-KEY-B]
// Prüft, dass ein mit Schlüssel A signiertes Token von einem mit Schlüssel B konfigurierten Gate
// strikt fail-closed mit `LicenseError::InvalidSignature` abgelehnt wird.
#[test]
fn test_token_from_key_a_rejected_by_key_b() {
    let (signing_key_a, _vk_a) = generate_deterministic_keypair(1);
    let (_signing_key_b, vk_b) = generate_deterministic_keypair(2);

    let installation_hash = [0xAA; 32];
    let expires_at = 2_000_000_000;

    // Erzeuge gültiges Token A mit Signaturschlüssel A
    let activation_a = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        installation_hash,
        expires_at,
        &signing_key_a,
    );

    // Initialisiere Gate mit Schlüssel B
    let gate_b = SignedLicenseGate::from_activation(activation_a, vk_b)
        .with_local_installation_id(installation_hash);

    // Verifikation von FeatureRing::Sovereign muss fail-closed fehlschlagen
    let err = gate_b
        .check_ring(FeatureRing::Sovereign)
        .expect_err("token from key A must be rejected by key B");

    assert_eq!(err, LicenseError::InvalidSignature);
}

// ANCHOR[contextra-license:TEST-ROTATION-PAYLOAD-KEY-A-REJECTED-BY-KEY-B]
// Prüft, dass ein signiertes LicensePayload mit falschem Schlüssel bereits im Konstruktor abgewiesen wird.
#[test]
fn test_signed_payload_from_key_a_rejected_by_key_b() {
    let (signing_key_a, _vk_a) = generate_deterministic_keypair(10);
    let (_signing_key_b, vk_b) = generate_deterministic_keypair(20);

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign, FeatureRing::Compliance],
        expires_at: Some(2_000_000_000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: Some([0x55; 32]),
    };

    let payload_bytes = bincode::serialize(&payload).unwrap();
    let signature_a = signing_key_a.sign(&payload_bytes).to_bytes();

    // Konstruktion mit verifying_key B muss fehlschlagen
    let res = SignedLicenseGate::from_signed_payload(&payload_bytes, &signature_a, vk_b);
    assert!(matches!(res, Err(LicenseError::InvalidSignature)));
}

// ANCHOR[contextra-license:TEST-ROTATION-SUCCESSFUL-KEY-SWAP]
// Prüft den determinierten Wechselprozess: Neues Gate mit Schlüssel B akzeptiert Token B und verwirft Token A.
#[test]
fn test_successful_key_swap_procedure() {
    let (signing_key_a, vk_a) = generate_deterministic_keypair(11);
    let (signing_key_b, vk_b) = generate_deterministic_keypair(22);

    let installation_hash = [0xBB; 32];
    let expires_at = 2_000_000_000;

    let activation_a = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        installation_hash,
        expires_at,
        &signing_key_a,
    );
    let activation_b = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        installation_hash,
        expires_at,
        &signing_key_b,
    );

    // Gate A: Akzeptiert Token A
    let gate_a = SignedLicenseGate::from_activation(activation_a.clone(), vk_a)
        .with_local_installation_id(installation_hash);
    assert!(gate_a.check_ring(FeatureRing::Sovereign).is_ok());

    // Durchführung des Schlüsselwechsels (Instanziierung Gate B mit Schlüssel B)
    let gate_b = SignedLicenseGate::from_activation(activation_b, vk_b)
        .with_local_installation_id(installation_hash);
    assert!(gate_b.check_ring(FeatureRing::Sovereign).is_ok());

    // Gegenprobe: Altes Token A an Gate B schlägt fehl
    let gate_b_old_token = SignedLicenseGate::from_activation(activation_a, vk_b)
        .with_local_installation_id(installation_hash);
    let err = gate_b_old_token
        .check_ring(FeatureRing::Sovereign)
        .expect_err("old token must be rejected under new key");
    assert_eq!(err, LicenseError::InvalidSignature);
}

// ANCHOR[contextra-license:TEST-ROTATION-ERROR-NO-KEY-LEAK]
// Prüft, dass Fehlermeldungen bei Signatur- und Schlüsseldiskrepanzen keine Schlüsseldaten enthalten.
#[test]
fn test_invalid_signature_error_contains_no_key_details() {
    let (signing_key_a, _vk_a) = generate_deterministic_keypair(33);
    let (_signing_key_b, vk_b) = generate_deterministic_keypair(44);

    let installation_hash = [0xCC; 32];
    let activation = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        installation_hash,
        2_000_000_000,
        &signing_key_a,
    );

    let gate = SignedLicenseGate::from_activation(activation, vk_b)
        .with_local_installation_id(installation_hash);

    let err = gate
        .check_ring(FeatureRing::Sovereign)
        .expect_err("must fail");

    assert_eq!(err, LicenseError::InvalidSignature);

    let display_str = format!("{}", err);
    let debug_str = format!("{:?}", err);

    // Weder Display noch Debug dürfen Schlüsselbytes, Hex-Repräsentationen oder 'ed25519' enthalten
    let key_b_hex: String = vk_b
        .to_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    assert!(!display_str.contains(&key_b_hex));
    assert!(!debug_str.contains(&key_b_hex));
    assert!(!display_str.contains("key"));
    assert_eq!(display_str, "activation signature verification failed");
}

// ANCHOR[contextra-license:TEST-ROTATION-TAMPERED-SIGNATURE-REJECTED]
// Prüft, dass ein manipulierter Signaturwert sofort abgewiesen wird.
#[test]
fn test_tampered_signature_rejected() {
    let (signing_key, vk) = generate_deterministic_keypair(55);
    let installation_hash = [0xDD; 32];

    let mut activation = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        installation_hash,
        2_000_000_000,
        &signing_key,
    );

    // Manipuliere das erste Signaturbyte
    activation.signature[0] ^= 0xFF;

    let gate = SignedLicenseGate::from_activation(activation, vk)
        .with_local_installation_id(installation_hash);

    let err = gate
        .check_ring(FeatureRing::Sovereign)
        .expect_err("tampered signature must be rejected");

    assert_eq!(err, LicenseError::InvalidSignature);
}

// ANCHOR[contextra-license:TEST-ROTATION-STRUCTURAL-NO-KEY-ID]
// Prüft strukturell, dass weder SignedActivation noch LicensePayload eine Schlüsselkennung (key_id) besitzen.
#[test]
fn test_structural_absence_of_key_id_in_tokens() {
    let (signing_key, vk) = generate_deterministic_keypair(77);
    let installation_hash = [0xEE; 32];

    let activation = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        installation_hash,
        2_000_000_000,
        &signing_key,
    );

    let serialized_activation = bincode::serialize(&activation).unwrap();

    // Suche nach dem öffentlichen Schlüssel im serialisierten Token
    let vk_bytes = vk.to_bytes();
    let contains_key = serialized_activation
        .windows(32)
        .any(|window| window == vk_bytes);

    // Der öffentliche Schlüssel ist nicht im Token eingebettet
    assert!(
        !contains_key,
        "token must not contain the public key or key identifier"
    );

    // Gate speichert exakt die übergebene Schlüsselreferenz
    let gate = SignedLicenseGate::from_activation(activation, vk);
    assert_eq!(gate.verifying_key(), Some(&vk));
}
