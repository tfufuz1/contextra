//! Integrationstests für SignedLicenseGate über den LicenseGate-Port.
//! Testet End-to-End-Szenarien mit echten Ed25519-Schlüsselpaaren für erlaubte und gesperrte Ringe.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::sync::Arc;

use contextra_license::{
    FeatureRing, LicenseError, LicenseGate, LicensePayload, SignedLicenseGate,
};
use contextra_ports::clock::Clock;
use contextra_types::TenantId;
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

// ANCHOR[contextra-license:TEST-GATE-VALID-SOVEREIGN]
// Prüft erfolgreiche Freigabe des Sovereign-Rings bei gültiger Signatur und passender Lizenz.
#[test]
fn test_valid_signature_allowed_ring_sovereign() {
    let (signing_key, verifying_key) = generate_keypair();

    let payload = LicensePayload {
        tenant_id: TenantId::try_new(42).expect("valid tenant"),
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

// ANCHOR[contextra-license:TEST-GATE-MISSING-COMPLIANCE]
// Prüft Abweisung des Compliance-Rings, wenn dieser in der signierten Lizenz nicht enthalten ist.
#[test]
fn test_valid_signature_missing_ring_compliance() {
    let (signing_key, verifying_key) = generate_keypair();

    let payload = LicensePayload {
        tenant_id: TenantId::try_new(42).expect("valid tenant"),
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

// ANCHOR[contextra-license:TEST-GATE-INVALID-SIG-REJECTED]
// Prüft Abweisung des Zugriffs bei manipulierter Signatur.
#[test]
fn test_invalid_signature_rejected() {
    let (signing_key, verifying_key) = generate_keypair();

    let payload = LicensePayload {
        tenant_id: TenantId::try_new(42).expect("valid tenant"),
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

// ANCHOR[contextra-license:TEST-GATE-EXPIRED-REJECTED]
// Prüft Abweisung des Zugriffs bei abgelaufener Lizenz.
#[test]
fn test_expired_license() {
    let (signing_key, verifying_key) = generate_keypair();

    let expires_at = 1600000000;
    let payload = LicensePayload {
        tenant_id: TenantId::try_new(42).expect("valid tenant"),
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

// ANCHOR[contextra-license:TEST-GATE-FAST-RING-ALWAYS-OK]
// Prüft Invariante INV-LICENSE-2: Fast-Ring ist selbst bei abgelaufener Lizenz immer erlaubt.
#[test]
fn test_inv_license_2_fast_ring_always_allowed() {
    let (signing_key, verifying_key) = generate_keypair();

    let expires_at = 1600000000;
    let payload = LicensePayload {
        tenant_id: TenantId::try_new(42).expect("valid tenant"),
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

    // Fast ring MUST pass unconditionally under INV-LICENSE-2 despite expiration & missing allowed_rings
    assert_eq!(gate.check_ring(FeatureRing::Fast), Ok(()));
}
