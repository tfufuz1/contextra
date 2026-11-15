//! Integrationstests für SignedLicenseGate über den LicenseGate-Port.
//! Testet End-to-End-Szenarien mit echten Ed25519-Schlüsselpaaren für erlaubte und gesperrte Ringe.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::sync::Arc;

use acta_node::Clock;
use acta_node::TenantId;
use acta_node::{
    FeatureRing, LicenseError, LicenseGate, LicensePayload, SignedLicenseGate,
};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;

struct TestClock {
    now_unix_secs: u64,
}

impl Clock for TestClock {
    fn now_unix_nanos(&self) -> u64 {
        self.now_unix_secs * 1_000_000_000
    }

    fn monotonic_nanos(&self) -> u64 {
        0
    }
}

fn generate_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

// ANCHOR[acta-node:TEST-GATE-VALID-SOVEREIGN]
// Prüft erfolgreiche Freigabe des Sovereign-Rings bei gültiger Signatur und passender Lizenz.
#[test]
fn test_valid_signature_allowed_ring_sovereign() {
    let (signing_key, verifying_key) = generate_keypair();

    let payload = LicensePayload {
        tenant_id: TenantId::new(42),
        allowed_rings: vec![FeatureRing::Fast, FeatureRing::Sovereign],
        expires_at: Some(2000000000), // Year 2033
        feature_flags: BTreeMap::from([("audit_export".to_string(), true)]),
        installation_id_hash: None,
    };

    let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes);

    let test_clock = Arc::new(TestClock {
        now_unix_secs: 1700000000,
    });

    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature.to_bytes(),
        verifying_key,
        test_clock,
    )
    .expect("gate creation succeeds");

    assert_eq!(gate.check_ring(FeatureRing::Sovereign), Ok(()));
}

// ANCHOR[acta-node:TEST-GATE-MISSING-COMPLIANCE]
// Prüft Abweisung des Compliance-Rings, wenn dieser in der signierten Lizenz nicht enthalten ist.
#[test]
fn test_valid_signature_missing_ring_compliance() {
    let (signing_key, verifying_key) = generate_keypair();

    let payload = LicensePayload {
        tenant_id: TenantId::new(42),
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000000000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: None,
    };

    let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes);

    let test_clock = Arc::new(TestClock {
        now_unix_secs: 1700000000,
    });

    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature.to_bytes(),
        verifying_key,
        test_clock,
    )
    .expect("gate creation succeeds");

    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::NotActivated(FeatureRing::Compliance))
    );
}

// ANCHOR[acta-node:TEST-GATE-INVALID-SIG-REJECTED]
// Prüft Abweisung des Zugriffs bei manipulierter Signatur.
#[test]
fn test_invalid_signature_rejected() {
    let (signing_key, verifying_key) = generate_keypair();

    let payload = LicensePayload {
        tenant_id: TenantId::new(42),
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000000000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: None,
    };

    let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let mut signature = signing_key.sign(&payload_bytes).to_bytes();

    // Tamper with signature
    signature[0] ^= 0xFF;

    let res = SignedLicenseGate::from_signed_payload(&payload_bytes, &signature, verifying_key);

    assert_eq!(res.err(), Some(LicenseError::InvalidSignature));
}

// ANCHOR[acta-node:TEST-GATE-EXPIRED-REJECTED]
// Prüft Abweisung des Zugriffs bei abgelaufener Lizenz.
#[test]
fn test_expired_license() {
    let (signing_key, verifying_key) = generate_keypair();

    let expires_at = 1600000000;
    let payload = LicensePayload {
        tenant_id: TenantId::new(42),
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(expires_at),
        feature_flags: BTreeMap::new(),
        installation_id_hash: None,
    };

    let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes);

    // Test clock after expiration
    let test_clock = Arc::new(TestClock {
        now_unix_secs: 1700000000,
    });

    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature.to_bytes(),
        verifying_key,
        test_clock,
    )
    .expect("gate creation succeeds");

    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::Expired(expires_at))
    );
}

// ANCHOR[acta-node:TEST-GATE-FAST-RING-ALWAYS-OK]
// Prüft Invariante: Fast-Ring ist selbst bei abgelaufener Lizenz immer erlaubt.
#[test]
fn test_inv_license_2_fast_ring_always_allowed() {
    let (signing_key, verifying_key) = generate_keypair();

    let expires_at = 1600000000;
    let payload = LicensePayload {
        tenant_id: TenantId::new(42),
        allowed_rings: vec![], // Fast ring NOT listed, expired
        expires_at: Some(expires_at),
        feature_flags: BTreeMap::new(),
        installation_id_hash: None,
    };

    let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes);

    let test_clock = Arc::new(TestClock {
        now_unix_secs: 1700000000, // Expired time
    });

    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature.to_bytes(),
        verifying_key,
        test_clock,
    )
    .expect("gate creation succeeds");

    // Fast ring MUST pass unconditionally under despite expiration & missing allowed_rings
    assert_eq!(gate.check_ring(FeatureRing::Fast), Ok(()));
}

#[test]
fn test_verify_ring_inherent_method() {
    let gate = SignedLicenseGate::no_activation();
    assert_eq!(gate.verify_ring(FeatureRing::Fast), Ok(()));
    assert_eq!(
        gate.verify_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}

// Invariante: Non-Destructive Degradation (Ablauf führt NIE zu Datenverlust oder Datensperre im Basispfad)
#[test]
fn test_invariant_non_destructive_degradation() {
    use acta_node::signed_gate::VerifyingKey;
    use acta_node::SignedActivation;

    // 1. Initialisiere ein Gate mit abgelaufener Lizenz
    let verifying_key_bytes = [1u8; 32];
    let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes).unwrap();
    let activation = SignedActivation {
        ring: FeatureRing::Sovereign,
        installation_id_hash: [0u8; 32],
        expires_at_unix: 1_000,
        signature: [0u8; 64],
    };
    let expired_clock = std::sync::Arc::new(acta_node::FixedClock(2_000_000_000_000));
    let gate =
        SignedLicenseGate::from_activation_with_clock(activation, verifying_key, expired_clock);

    // 2. Sovereign-Ring scheitert (Fail-Closed)
    assert!(gate.check_ring(FeatureRing::Sovereign).is_err());

    // 3. Fast-Ring bleibt vollkommen funktionsfähig
    assert!(gate.check_ring(FeatureRing::Fast).is_ok());
    assert!(gate.authorize(FeatureRing::Fast).is_ok());
}
