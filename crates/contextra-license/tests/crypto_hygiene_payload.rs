#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_sign_loss,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;
use std::sync::Arc;

use contextra_license::{
    constant_time_eq_32, FeatureRing, LicenseError, LicensePayload, SignedLicenseGate,
    SignedLicenseOptions, MAX_PAYLOAD_BYTES,
};
use contextra_ports::clock::Clock;
use contextra_types::TenantId;
use ed25519_dalek::{Signer, SigningKey};

struct TestClock {
    now_unix_secs: i64,
}

impl Clock for TestClock {
    fn now_unix_nanos(&self) -> u64 {
        (self.now_unix_secs.max(0) as u64) * 1_000_000_000
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

#[test]
fn test_constant_time_eq_32_matching() {
    let a = [0xAA; 32];
    let b = [0xAA; 32];
    assert!(constant_time_eq_32(&a, &b));
}

#[test]
fn test_constant_time_eq_32_mismatch_single_byte() {
    let a = [0xAA; 32];
    let mut b = [0xAA; 32];
    b[31] = 0xAB;
    assert!(!constant_time_eq_32(&a, &b));
}

#[test]
fn test_constant_time_eq_32_mismatch_all_bytes() {
    let a = [0x00; 32];
    let b = [0xFF; 32];
    assert!(!constant_time_eq_32(&a, &b));
}

#[test]
fn test_payload_exceeding_max_bytes_fails() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let oversized_payload = vec![0x42u8; MAX_PAYLOAD_BYTES + 1];
    let signature = signing_key.sign(&oversized_payload).to_bytes();

    let res = SignedLicenseGate::from_signed_payload_with_clock(
        &oversized_payload,
        &signature,
        verifying_key,
        clock,
    );

    assert_eq!(res.err(), Some(LicenseError::InvalidSignature));
}

#[test]
fn test_custom_max_payload_bytes_limit() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: Some([1u8; 32]),
    };

    let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes).to_bytes();

    // Set custom max payload limit smaller than payload_bytes length
    let opts = SignedLicenseOptions {
        allow_legacy: false,
        max_payload_bytes: payload_bytes.len() - 1,
    };

    let res = SignedLicenseGate::from_signed_payload_with_clock_and_opts(
        &payload_bytes,
        &signature,
        verifying_key,
        clock,
        opts,
    );

    assert_eq!(res.err(), Some(LicenseError::InvalidSignature));
}

#[test]
fn test_canonical_serialization_rejects_trailing_garbage() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Compliance],
        expires_at: Some(2000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: Some([2u8; 32]),
    };

    let mut payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    payload_bytes.push(0x00); // Trailing byte violates canonical bincode serialization

    let signature = signing_key.sign(&payload_bytes).to_bytes();

    let res = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature,
        verifying_key,
        clock,
    );

    assert_eq!(res.err(), Some(LicenseError::InvalidSignature));
}
