#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_sign_loss,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;
use std::sync::Arc;

use contextra_license::{
    FeatureRing, LicenseError, LicenseGate, LicensePayload, SignedActivation, SignedLicenseGate,
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

fn generate_keypair(seed_byte: u8) -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let seed = [seed_byte; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

#[test]
fn test_from_activation_constructor_uses_system_clock() {
    let (signing_key, verifying_key) = generate_keypair(1);
    let local_id = [10u8; 32];
    // Far-future expiration so system clock won't expire it
    let activation = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        local_id,
        2_500_000_000,
        &signing_key,
    );

    let gate = SignedLicenseGate::from_activation(activation, verifying_key)
        .with_local_installation_id(local_id);

    assert!(gate.activation().is_some());
    assert!(gate.verifying_key().is_some());
    assert_eq!(gate.check_ring(FeatureRing::Sovereign), Ok(()));
}

#[test]
fn test_negative_1_manipulated_signature_rejected() {
    let (signing_key, verifying_key) = generate_keypair(2);
    let local_id = [11u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let mut activation =
        SignedActivation::create_signed(FeatureRing::Compliance, local_id, 2000, &signing_key);
    // Tamper with signature bytes
    activation.signature[32] ^= 0x42;

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(local_id);

    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::InvalidSignature)
    );
}

#[test]
fn test_negative_2_wrong_verifying_key_rejected() {
    let (signing_key_a, _) = generate_keypair(3);
    let (_, wrong_verifying_key_b) = generate_keypair(4);
    let local_id = [12u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, local_id, 2000, &signing_key_a);

    let gate =
        SignedLicenseGate::from_activation_with_clock(activation, wrong_verifying_key_b, clock)
            .with_local_installation_id(local_id);

    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::InvalidSignature)
    );
}

#[test]
fn test_negative_3_expired_activation_rejected() {
    let (signing_key, verifying_key) = generate_keypair(5);
    let local_id = [13u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 2001,
    });

    let activation =
        SignedActivation::create_signed(FeatureRing::Compliance, local_id, 2000, &signing_key);

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(local_id);

    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::Expired(2000))
    );
}

#[test]
fn test_negative_4_wrong_ring_product_id_rejected() {
    let (signing_key, verifying_key) = generate_keypair(6);
    let local_id = [14u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    // Activated for Compliance ring
    let activation =
        SignedActivation::create_signed(FeatureRing::Compliance, local_id, 2000, &signing_key);

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(local_id);

    // Requesting Sovereign ring fails
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}

#[test]
fn test_negative_5_wrong_installation_id_rejected() {
    let (signing_key, verifying_key) = generate_keypair(7);
    let expected_id = [15u8; 32];
    let wrong_id = [99u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, expected_id, 2000, &signing_key);

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(wrong_id);

    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}

#[test]
fn test_negative_6_truncated_payload_bytes_rejected() {
    let (signing_key, verifying_key) = generate_keypair(8);
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let payload = LicensePayload {
        tenant_id: TenantId::SYSTEM,
        allowed_rings: vec![FeatureRing::Sovereign],
        expires_at: Some(2000),
        feature_flags: BTreeMap::new(),
        installation_id_hash: Some([16u8; 32]),
    };

    let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
    let truncated_bytes = &payload_bytes[..payload_bytes.len() / 2];
    let signature = signing_key.sign(truncated_bytes).to_bytes();

    let res = SignedLicenseGate::from_signed_payload_with_clock(
        truncated_bytes,
        &signature,
        verifying_key,
        clock,
    );

    assert_eq!(res.err(), Some(LicenseError::InvalidSignature));
}

#[test]
fn test_negative_7_empty_payload_bytes_rejected() {
    let (signing_key, verifying_key) = generate_keypair(9);
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let empty_bytes = &[];
    let signature = signing_key.sign(empty_bytes).to_bytes();

    let res = SignedLicenseGate::from_signed_payload_with_clock(
        empty_bytes,
        &signature,
        verifying_key,
        clock,
    );

    assert_eq!(res.err(), Some(LicenseError::InvalidSignature));
}

#[test]
fn test_negative_8_legacy_payload_without_flag_rejected() {
    let (signing_key, verifying_key) = generate_keypair(10);
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let legacy_payload_bytes = bincode::serialize(&(
        TenantId::SYSTEM,
        vec![FeatureRing::Sovereign],
        Some(2000i64),
        BTreeMap::<String, bool>::new(),
    ))
    .expect("serialization succeeds");

    let signature = signing_key.sign(&legacy_payload_bytes).to_bytes();

    // Default opts (allow_legacy: false)
    let res = SignedLicenseGate::from_signed_payload_with_clock(
        &legacy_payload_bytes,
        &signature,
        verifying_key,
        clock,
    );

    assert_eq!(res.err(), Some(LicenseError::InvalidSignature));
}

#[test]
fn test_negative_9_legacy_payload_with_flag_permitted() {
    let (signing_key, verifying_key) = generate_keypair(11);
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let legacy_payload_bytes = bincode::serialize(&(
        TenantId::SYSTEM,
        vec![FeatureRing::Sovereign],
        Some(2000i64),
        BTreeMap::<String, bool>::new(),
    ))
    .expect("serialization succeeds");

    let signature = signing_key.sign(&legacy_payload_bytes).to_bytes();

    let opts = SignedLicenseOptions {
        allow_legacy: true,
        max_payload_bytes: MAX_PAYLOAD_BYTES,
    };

    let gate = SignedLicenseGate::from_signed_payload_with_clock_and_opts(
        &legacy_payload_bytes,
        &signature,
        verifying_key,
        clock,
        opts,
    )
    .expect("legacy payload permitted when flag is true");

    assert_eq!(gate.check_ring(FeatureRing::Sovereign), Ok(()));
}

#[test]
fn test_negative_10_missing_activation_or_payload_gate() {
    let gate = SignedLicenseGate::no_activation();
    assert!(gate.activation().is_none());
    assert!(gate.license_payload().is_none());
    assert!(gate.verifying_key().is_none());

    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::NotActivated(FeatureRing::Compliance))
    );
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}
