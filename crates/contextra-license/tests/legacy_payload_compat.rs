#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::sync::Arc;

use contextra_license::{
    FeatureRing, LicenseError, LicenseGate, LicensePayload, OpenFastGate, SignedLicenseGate,
};
use contextra_ports::clock::Clock;
use contextra_types::TenantId;
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};

struct FixedTestClock;

impl Clock for FixedTestClock {
    fn now_unix_nanos(&self) -> u64 {
        1_000_000_000
    }

    fn monotonic_nanos(&self) -> u64 {
        0
    }
}

fn fixed_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let secret_bytes = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

#[derive(Serialize, Deserialize)]
struct LegacyPayloadRaw {
    tenant_id: TenantId,
    allowed_rings: Vec<FeatureRing>,
    expires_at: Option<i64>,
    feature_flags: BTreeMap<String, bool>,
}

/// Round-trip test for new LicensePayload schema containing installation_id_hash.
#[test]
fn test_roundtrip_new_payload_with_hash() {
    let (signing_key, verifying_key) = fixed_keypair();
    let local_id = [77u8; 32];

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: Some(local_id),
    };

    let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes).to_bytes();

    let clock = Arc::new(FixedTestClock);
    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature,
        verifying_key,
        clock,
    )
    .expect("gate creation succeeds")
    .with_local_installation_id(local_id);

    assert_eq!(
        gate.license_payload().unwrap().installation_id_hash,
        Some(local_id)
    );
    assert_eq!(gate.check_ring(FeatureRing::Sovereign), Ok(()));
}

/// Round-trip test for legacy payload schema without installation_id_hash.
#[test]
fn test_roundtrip_legacy_payload_without_hash() {
    let (signing_key, verifying_key) = fixed_keypair();

    let legacy_payload = LegacyPayloadRaw {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000),
        feature_flags: BTreeMap::new(),
    };

    let legacy_bytes = bincode::serialize(&legacy_payload).expect("serialization succeeds");
    let signature = signing_key.sign(&legacy_bytes).to_bytes();

    let clock = Arc::new(FixedTestClock);
    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &legacy_bytes,
        &signature,
        verifying_key,
        clock,
    )
    .expect("legacy gate creation succeeds");

    assert_eq!(gate.license_payload().unwrap().installation_id_hash, None);
    assert_eq!(gate.check_ring(FeatureRing::Sovereign), Ok(()));
}

/// Negativtest: A new payload with corrupted/truncated hash bytes must NOT be accepted via legacy fallback.
#[test]
fn test_new_payload_with_corrupted_hash_rejects_legacy_fallback() {
    let (signing_key, verifying_key) = fixed_keypair();

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: Some([88u8; 32]),
    };

    let full_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    // Truncate the payload bytes so the Option<[u8;32]> is incomplete
    let truncated_bytes = &full_bytes[..full_bytes.len() - 16];

    // Sign the truncated bytes
    let signature = signing_key.sign(truncated_bytes).to_bytes();

    let clock = Arc::new(FixedTestClock);
    let res = SignedLicenseGate::from_signed_payload_with_clock(
        truncated_bytes,
        &signature,
        verifying_key,
        clock,
    );

    // Rejection must occur (InvalidSignature error returned when both deserializations fail)
    assert_eq!(res.err(), Some(LicenseError::InvalidSignature));
}

/// Ring Semantik: Compliance-Aktivierung allein schaltet Sovereign NICHT frei.
#[test]
fn test_ring_semantics_compliance_alone_does_not_unlock_sovereign() {
    let (signing_key, verifying_key) = fixed_keypair();

    let payload_compliance_only = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Compliance],
        expires_at: Some(2000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: None,
    };

    let payload_bytes =
        bincode::serialize(&payload_compliance_only).expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes).to_bytes();

    let clock = Arc::new(FixedTestClock);
    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature,
        verifying_key,
        clock,
    )
    .expect("gate creation succeeds");

    // Compliance ring is allowed
    assert_eq!(gate.check_ring(FeatureRing::Compliance), Ok(()));

    // Sovereign ring is NOT allowed
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}

/// OpenFastGate always permits Fast and denies Sovereign/Compliance.
#[test]
fn test_open_fast_gate_semantics() {
    let gate = OpenFastGate;
    assert_eq!(gate.check_ring(FeatureRing::Fast), Ok(()));
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::NotActivated(FeatureRing::Compliance))
    );
}
