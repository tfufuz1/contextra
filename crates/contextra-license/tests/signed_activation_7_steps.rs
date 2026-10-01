#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;

use contextra_license::{
    derive_local_installation_id_hash, FeatureRing, LicenseError, LicenseGate, SignedActivation,
    SignedLicenseGate,
};
use contextra_ports::clock::Clock;
use ed25519_dalek::SigningKey;
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

/// Prüfstufe 1: Fast-Ring ohne jede Aktivierung ist Ok (INV-LICENSE-2)
#[test]
fn test_step_1_fast_ring_without_any_activation_is_ok() {
    let gate = SignedLicenseGate::no_activation();
    assert_eq!(gate.check_ring(FeatureRing::Fast), Ok(()));
}

/// Prüfstufe 2: Fehlende Aktivierung für Sovereign/Compliance ist NotActivated
#[test]
fn test_step_2_missing_activation_returns_not_activated() {
    let gate = SignedLicenseGate::no_activation();
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::NotActivated(FeatureRing::Compliance))
    );
}

/// Prüfstufe 3: Falscher installation_id_hash ist NotActivated (nicht InvalidSignature)
#[test]
fn test_step_3_mismatched_installation_id_returns_not_activated_before_signature_check() {
    let (signing_key, verifying_key) = generate_keypair();
    let expected_id = [11u8; 32];
    let wrong_local_id = [99u8; 32];

    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    // Create an activation for expected_id with tampered signature to verify Step 3 runs BEFORE Step 4
    let mut activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, expected_id, 2000, &signing_key);
    activation.signature[0] ^= 0xFF; // Tamper signature

    let gate = SignedLicenseGate::from_activation_with_clock(
        activation,
        verifying_key,
        clock,
    )
    .with_local_installation_id(wrong_local_id);

    // Step 3 MUST return NotActivated (not InvalidSignature) to prevent info leaks
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}

/// Prüfstufe 4: Manipulierte Signatur ist InvalidSignature
#[test]
fn test_step_4_tampered_signature_returns_invalid_signature() {
    let (signing_key, verifying_key) = generate_keypair();
    let local_id = [22u8; 32];

    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let mut activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, local_id, 2000, &signing_key);
    activation.signature[0] ^= 0xFF; // Tamper signature

    let gate = SignedLicenseGate::from_activation_with_clock(
        activation,
        verifying_key,
        clock,
    )
    .with_local_installation_id(local_id);

    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::InvalidSignature)
    );
}

/// Prüfstufe 5: Abgelaufene Aktivierung ist Expired
#[test]
fn test_step_5_expired_activation_returns_expired() {
    let (signing_key, verifying_key) = generate_keypair();
    let local_id = [33u8; 32];
    let expires_at_unix = 1500;

    let clock = Arc::new(TestClock {
        now_unix_secs: 1600, // Current time is past expiration
    });

    let activation = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        local_id,
        expires_at_unix,
        &signing_key,
    );

    let gate = SignedLicenseGate::from_activation_with_clock(
        activation,
        verifying_key,
        clock,
    )
    .with_local_installation_id(local_id);

    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::Expired(expires_at_unix))
    );
}

/// Prüfstufe 6: Zu niedriger aktivierter Ring ist NotActivated ohne automatische Höherstufung
#[test]
fn test_step_6_insufficient_activated_ring_returns_not_activated() {
    let (signing_key, verifying_key) = generate_keypair();
    let local_id = [44u8; 32];

    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    // Activated ring is Sovereign, but Compliance ring is requested
    let activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, local_id, 2000, &signing_key);

    let gate = SignedLicenseGate::from_activation_with_clock(
        activation,
        verifying_key,
        clock,
    )
    .with_local_installation_id(local_id);

    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::NotActivated(FeatureRing::Compliance))
    );
}

/// Prüfstufe 7: Vollständig korrekte Aktivierung ist Ok
#[test]
fn test_step_7_fully_valid_activation_returns_ok() {
    let (signing_key, verifying_key) = generate_keypair();
    let local_id = derive_local_installation_id_hash(None);

    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let activation =
        SignedActivation::create_signed(FeatureRing::Compliance, local_id, 2000, &signing_key);

    let gate = SignedLicenseGate::from_activation_with_clock(
        activation,
        verifying_key,
        clock,
    )
    .with_local_installation_id(local_id);

    assert_eq!(gate.check_ring(FeatureRing::Compliance), Ok(()));
}
