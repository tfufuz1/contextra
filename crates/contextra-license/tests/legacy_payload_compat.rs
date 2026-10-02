#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::sync::Arc;

use bincode::Options;
use contextra_license::{
    FeatureRing, LicenseError, LicenseGate, LicensePayload, SignedLicenseGate,
};
use contextra_ports::clock::Clock;
use contextra_types::TenantId;
use ed25519_dalek::{Signer, SigningKey};
use serde::Serialize;

struct TestClock {
    now_unix_secs: i64,
}

impl Clock for TestClock {
    fn now_unix_nanos(&self) -> u64 {
        (self.now_unix_secs * 1_000_000_000) as u64
    }

    fn monotonic_nanos(&self) -> u64 {
        0
    }
}

fn generate_deterministic_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let seed = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

#[derive(Serialize)]
struct LegacyPayloadSchema {
    tenant_id: TenantId,
    allowed_rings: Vec<FeatureRing>,
    expires_at: Option<i64>,
    feature_flags: BTreeMap<String, bool>,
}

#[test]
fn test_legacy_payload_roundtrip_deserialization() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let legacy = LegacyPayloadSchema {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000),
        feature_flags: BTreeMap::new(),
    };

    let bincode_opts = bincode::options().with_fixint_encoding();
    let legacy_bytes = bincode_opts
        .serialize(&legacy)
        .expect("serialization succeeds");
    let signature = signing_key.sign(&legacy_bytes).to_bytes();

    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &legacy_bytes,
        &signature,
        verifying_key,
        clock,
    )
    .expect("gate accepts legacy payload");

    assert_eq!(gate.license_payload().unwrap().installation_id_hash, None);
    assert_eq!(gate.check_ring(FeatureRing::Sovereign), Ok(()));
}

#[test]
fn test_new_payload_roundtrip_deserialization() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let hash = [7u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: Some(hash),
    };

    let bincode_opts = bincode::options().with_fixint_encoding();
    let payload_bytes = bincode_opts
        .serialize(&payload)
        .expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes).to_bytes();

    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature,
        verifying_key,
        clock,
    )
    .expect("gate accepts new payload")
    .with_local_installation_id(hash);

    assert_eq!(
        gate.license_payload().unwrap().installation_id_hash,
        Some(hash)
    );
    assert_eq!(gate.check_ring(FeatureRing::Sovereign), Ok(()));
}

#[test]
fn test_corrupted_new_payload_not_accepted_as_legacy() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let hash = [7u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: Some(hash),
    };

    let bincode_opts = bincode::options().with_fixint_encoding();
    let mut payload_bytes = bincode_opts
        .serialize(&payload)
        .expect("serialization succeeds");

    // Truncate or append trailing garbage bytes to simulate corrupted payload
    payload_bytes.extend_from_slice(&[0xFF, 0xEE]);

    let signature = signing_key.sign(&payload_bytes).to_bytes();

    let res = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature,
        verifying_key,
        clock,
    );

    assert_eq!(res.err(), Some(LicenseError::InvalidSignature));
}
