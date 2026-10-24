#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;

use contextra_license::{
    FeatureRing, LicenseError, LicenseGate, SignedActivation, SignedLicenseGate,
};
use contextra_ports::clock::Clock;
use ed25519_dalek::SigningKey;

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

/// Step 1: Fast ring returns Ok unconditionally even if activation is missing, hash mismatch, signature corrupted, or expired.
#[test]
fn test_order_step_1_fast_ring_bypass() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let (foreign_signing_key, _) = {
        let seed = [99u8; 32];
        let sk = SigningKey::from_bytes(&seed);
        let vk = sk.verifying_key();
        (sk, vk)
    };

    let expected_id = [1u8; 32];
    let wrong_id = [2u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 2000,
    });

    // 1a. Missing activation -> Fast is Ok
    let gate_no_act = SignedLicenseGate::no_activation();
    assert_eq!(gate_no_act.check_ring(FeatureRing::Fast), Ok(()));

    // 1b. Corrupted signature -> Fast is Ok
    let mut act_corrupt =
        SignedActivation::create_signed(FeatureRing::Sovereign, expected_id, 1000, &signing_key);
    act_corrupt.signature[0] ^= 0xFF;
    let gate_corrupt =
        SignedLicenseGate::from_activation_with_clock(act_corrupt, verifying_key, clock.clone())
            .with_local_installation_id(wrong_id);
    assert_eq!(gate_corrupt.check_ring(FeatureRing::Fast), Ok(()));

    // 1c. Foreign signed activation -> Fast is Ok
    let act_foreign = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        expected_id,
        3000,
        &foreign_signing_key,
    );
    let gate_foreign =
        SignedLicenseGate::from_activation_with_clock(act_foreign, verifying_key, clock)
            .with_local_installation_id(wrong_id);
    assert_eq!(gate_foreign.check_ring(FeatureRing::Fast), Ok(()));
}

#[test]
fn test_ring_semantics_compliance_does_not_unlock_sovereign_and_open_fast_gate() {
    use contextra_license::OpenFastGate;

    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let local_id = [1u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    // Activation for Compliance ring only
    let activation =
        SignedActivation::create_signed(FeatureRing::Compliance, local_id, 2000, &signing_key);

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(local_id);

    // Compliance passes
    assert_eq!(gate.check_ring(FeatureRing::Compliance), Ok(()));
    // Sovereign fails (no automatic hierarchy upwards)
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );

    // OpenFastGate only permits Fast
    let open_fast = OpenFastGate;
    assert_eq!(open_fast.check_ring(FeatureRing::Fast), Ok(()));
    assert_eq!(
        open_fast.check_ring(FeatureRing::Compliance),
        Err(LicenseError::NotActivated(FeatureRing::Compliance))
    );
    assert_eq!(
        open_fast.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}

/// Step 2: Missing activation returns NotActivated before hash/sig/expiration/ring checks.
#[test]
fn test_order_step_2_missing_activation() {
    let gate = SignedLicenseGate::no_activation();
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}

/// Step 3: Installation hash mismatch returns NotActivated before signature check (Step 4).
#[test]
fn test_order_step_3_hash_mismatch_before_signature_check() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let expected_id = [1u8; 32];
    let wrong_id = [2u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let mut activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, expected_id, 2000, &signing_key);
    activation.signature[0] ^= 0xFF; // tampered signature

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(wrong_id);

    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}

/// Step 4: Invalid signature returns InvalidSignature before expiration check (Step 5).
#[test]
fn test_order_step_4_signature_check_before_expiration() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let local_id = [1u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 3000, // expired
    });

    let mut activation = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        local_id,
        2000, // expires at 2000
        &signing_key,
    );
    activation.signature[0] ^= 0xFF; // tampered signature

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(local_id);

    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::InvalidSignature)
    );
}

/// Step 5: Expired activation returns Expired before ring level check (Step 6).
#[test]
fn test_order_step_5_expiration_before_ring_level_check() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let local_id = [1u8; 32];
    let expires_at = 2000i64;
    let clock = Arc::new(TestClock {
        now_unix_secs: 2500, // past expires_at
    });

    let activation = SignedActivation::create_signed(
        FeatureRing::Sovereign, // activated for Sovereign
        local_id,
        expires_at,
        &signing_key,
    );

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(local_id);

    // Requesting Compliance ring (which does not match Sovereign)
    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::Expired(expires_at))
    );
}

/// Step 6: Ring requirement check returns NotActivated when activation ring != requested ring.
#[test]
fn test_order_step_6_ring_mismatch() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let local_id = [1u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, local_id, 2000, &signing_key);

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(local_id);

    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::NotActivated(FeatureRing::Compliance))
    );
}

/// Step 7: Valid activation returns Ok.
#[test]
fn test_order_step_7_success() {
    let (signing_key, verifying_key) = generate_deterministic_keypair();
    let local_id = [1u8; 32];
    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    let activation =
        SignedActivation::create_signed(FeatureRing::Compliance, local_id, 2000, &signing_key);

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(local_id);

    assert_eq!(gate.check_ring(FeatureRing::Compliance), Ok(()));
}
