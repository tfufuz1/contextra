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
use serde::{Deserialize, Serialize};

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

/// (1) check_ring(Fast) == Ok for every state (correct ID, wrong ID, missing ID, expired)
#[test]
fn test_fast_ring_always_allowed_across_all_states() {
    let (signing_key, verifying_key) = generate_keypair();
    let local_id = [1u8; 32];
    let wrong_id = [2u8; 32];

    let expired_at = 1600000000i64; // Expired
    let clock = Arc::new(TestClock {
        now_unix_secs: 1700000000,
    });

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![],
        expires_at: Some(expired_at),
        feature_flags: BTreeMap::new(),
        installation_id_hash: Some(local_id),
    };

    let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes).to_bytes();

    // 1a: Matching local ID, expired -> Fast ring passes
    let gate1 = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature,
        verifying_key,
        clock.clone(),
    )
    .expect("gate creation succeeds")
    .with_local_installation_id(local_id);

    assert_eq!(gate1.check_ring(FeatureRing::Fast), Ok(()));

    // 1b: Wrong local ID, expired -> Fast ring passes
    let gate2 = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature,
        verifying_key,
        clock.clone(),
    )
    .expect("gate creation succeeds")
    .with_local_installation_id(wrong_id);

    assert_eq!(gate2.check_ring(FeatureRing::Fast), Ok(()));

    // 1c: Missing local ID in gate, expired -> Fast ring passes
    let gate3 = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature,
        verifying_key,
        clock,
    )
    .expect("gate creation succeeds");

    assert_eq!(gate3.check_ring(FeatureRing::Fast), Ok(()));
}

/// (2) ManualClock after expires_at -> Expired
#[test]
fn test_expired_after_expiration_time() {
    let (signing_key, verifying_key) = generate_keypair();
    let expires_at = 1600000000i64;

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(expires_at),
        feature_flags: BTreeMap::new(),
        installation_id_hash: None,
    };

    let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes).to_bytes();

    let clock = Arc::new(TestClock {
        now_unix_secs: 1700000000,
    });

    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature,
        verifying_key,
        clock,
    )
    .expect("gate creation succeeds");

    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::Expired(expires_at))
    );
}

/// (3) Flipped bit in payload -> construction fails with InvalidSignature
#[test]
fn test_flipped_bit_in_payload_fails_construction() {
    let (signing_key, verifying_key) = generate_keypair();

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000000000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: Some([7u8; 32]),
    };

    let mut payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes).to_bytes();

    // Flip a bit in payload_bytes
    if let Some(byte) = payload_bytes.get_mut(0) {
        *byte ^= 0xFF;
    }

    let clock = Arc::new(TestClock {
        now_unix_secs: 1700000000,
    });

    let res = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature,
        verifying_key,
        clock,
    );

    assert_eq!(res.err(), Some(LicenseError::InvalidSignature));
}

/// (4) Wrong installation ID -> NotActivated
#[test]
fn test_mismatched_installation_id_returns_not_activated() {
    let (signing_key, verifying_key) = generate_keypair();
    let expected_id = [10u8; 32];
    let wrong_id = [99u8; 32];

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000000000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: Some(expected_id),
    };

    let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes).to_bytes();

    let clock = Arc::new(TestClock {
        now_unix_secs: 1700000000,
    });

    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature,
        verifying_key,
        clock,
    )
    .expect("gate creation succeeds")
    .with_local_installation_id(wrong_id);

    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}

/// (5) Payload without installation_id_hash (legacy bytes) behaves as before
#[test]
fn test_legacy_payload_bytes_without_installation_id_hash() {
    #[derive(Serialize, Deserialize)]
    struct LegacyPayload {
        pub tenant_id: TenantId,
        pub allowed_rings: Vec<FeatureRing>,
        pub expires_at: Option<i64>,
        pub feature_flags: BTreeMap<String, bool>,
    }

    let (signing_key, verifying_key) = generate_keypair();

    let legacy_payload = LegacyPayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000000000),
        feature_flags: BTreeMap::new(),
    };

    let legacy_bytes = bincode::serialize(&legacy_payload).expect("serialization succeeds");
    let signature = signing_key.sign(&legacy_bytes).to_bytes();

    let clock = Arc::new(TestClock {
        now_unix_secs: 1700000000,
    });

    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &legacy_bytes,
        &signature,
        verifying_key,
        clock,
    )
    .expect("legacy gate creation succeeds");

    assert_eq!(gate.license_payload().installation_id_hash, None);
    assert_eq!(gate.check_ring(FeatureRing::Sovereign), Ok(()));
}

/// (6) Payload with Hash, but gate without local installation ID -> NotActivated
#[test]
fn test_payload_with_hash_gate_without_local_id_returns_not_activated() {
    let (signing_key, verifying_key) = generate_keypair();
    let expected_id = [42u8; 32];

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Compliance],
        expires_at: Some(2000000000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: Some(expected_id),
    };

    let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let signature = signing_key.sign(&payload_bytes).to_bytes();

    let clock = Arc::new(TestClock {
        now_unix_secs: 1700000000,
    });

    // Gate created without calling with_local_installation_id
    let gate = SignedLicenseGate::from_signed_payload_with_clock(
        &payload_bytes,
        &signature,
        verifying_key,
        clock,
    )
    .expect("gate creation succeeds");

    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::NotActivated(FeatureRing::Compliance))
    );
}
