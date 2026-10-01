#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;

use contextra_license::{
    derive_local_installation_id_hash, FeatureRing, LicenseError, LicenseGate, SignedActivation,
    SignedLicenseGate,
};
use contextra_ports::clock::Clock;
use ed25519_dalek::SigningKey;

struct FixedTestClock {
    now_unix_secs: i64,
}

impl Clock for FixedTestClock {
    fn now_unix_nanos(&self) -> u64 {
        if self.now_unix_secs < 0 {
            0
        } else {
            (self.now_unix_secs as u64) * 1_000_000_000
        }
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

/// Step 1: Fast ring always returns Ok unconditionally across corrupt, empty, expired, or tampered activations.
#[test]
fn test_order_step_1_fast_ring_unconditional_ok() {
    let (signing_key, verifying_key) = fixed_keypair();
    let local_id = [10u8; 32];
    let wrong_id = [99u8; 32];

    // 1a: No activation
    let gate_empty = SignedLicenseGate::no_activation();
    assert_eq!(gate_empty.check_ring(FeatureRing::Fast), Ok(()));

    // 1b: Tampered signature, wrong ID, expired
    let clock = Arc::new(FixedTestClock {
        now_unix_secs: 2000,
    });
    let mut activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, local_id, 1000, &signing_key);
    activation.signature[0] ^= 0xFF; // Tamper signature

    let gate_corrupt =
        SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
            .with_local_installation_id(wrong_id);

    assert_eq!(gate_corrupt.check_ring(FeatureRing::Fast), Ok(()));
}

/// Step 2: No activation returns NotActivated.
#[test]
fn test_order_step_2_missing_activation_returns_not_activated() {
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

/// Step 3: Mismatched or missing local installation ID returns NotActivated BEFORE checking signature or expiration.
#[test]
fn test_order_step_3_mismatched_installation_id_runs_before_signature_check() {
    let (signing_key, verifying_key) = fixed_keypair();
    let expected_id = [11u8; 32];
    let wrong_local_id = [99u8; 32];

    let clock = Arc::new(FixedTestClock {
        now_unix_secs: 3000, // Current time is past expiration (1000)
    });

    // Create activation with expected_id, expired (1000), and tampered signature
    let mut activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, expected_id, 1000, &signing_key);
    activation.signature[0] ^= 0xFF; // Tamper signature

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(wrong_local_id);

    // Step 3 MUST return NotActivated (not InvalidSignature or Expired) to prevent info leaks
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}

/// Step 4: Invalid signature returns InvalidSignature after Step 3 passes.
#[test]
fn test_order_step_4_invalid_signature_returns_invalid_signature() {
    let (signing_key, verifying_key) = fixed_keypair();
    let local_id = [22u8; 32];

    let clock = Arc::new(FixedTestClock {
        now_unix_secs: 1000,
    });

    let mut activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, local_id, 2000, &signing_key);
    activation.signature[0] ^= 0xFF; // Tamper signature

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(local_id);

    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::InvalidSignature)
    );
}

/// Step 5: Clock expiration check (now_secs >= expires_at returns Expired).
#[test]
fn test_order_step_5_clock_expiration_edge_cases() {
    let (signing_key, verifying_key) = fixed_keypair();
    let local_id = [33u8; 32];

    // 5a: Exact match now == expires_at -> Expired
    let clock_exact = Arc::new(FixedTestClock {
        now_unix_secs: 1500,
    });
    let act_exact =
        SignedActivation::create_signed(FeatureRing::Sovereign, local_id, 1500, &signing_key);
    let gate_exact =
        SignedLicenseGate::from_activation_with_clock(act_exact, verifying_key, clock_exact)
            .with_local_installation_id(local_id);

    assert_eq!(
        gate_exact.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::Expired(1500))
    );

    // 5b: Negative expiration timestamp
    let clock_neg = Arc::new(FixedTestClock { now_unix_secs: 0 });
    let act_neg =
        SignedActivation::create_signed(FeatureRing::Sovereign, local_id, -100, &signing_key);
    let gate_neg = SignedLicenseGate::from_activation_with_clock(act_neg, verifying_key, clock_neg)
        .with_local_installation_id(local_id);

    assert_eq!(
        gate_neg.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::Expired(-100))
    );

    // 5c: i64::MAX expiration timestamp -> not expired
    let clock_max = Arc::new(FixedTestClock {
        now_unix_secs: 1_700_000_000,
    });
    let act_max =
        SignedActivation::create_signed(FeatureRing::Sovereign, local_id, i64::MAX, &signing_key);
    let gate_max = SignedLicenseGate::from_activation_with_clock(act_max, verifying_key, clock_max)
        .with_local_installation_id(local_id);

    assert_eq!(gate_max.check_ring(FeatureRing::Sovereign), Ok(()));
}

/// Step 6: Ring level requirement check (no automatic hierarchy inheritance).
#[test]
fn test_order_step_6_ring_mismatch_returns_not_activated() {
    let (signing_key, verifying_key) = fixed_keypair();
    let local_id = [44u8; 32];

    let clock = Arc::new(FixedTestClock {
        now_unix_secs: 1000,
    });

    let activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, local_id, 2000, &signing_key);

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(local_id);

    // Sovereign activation does NOT grant Compliance ring
    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::NotActivated(FeatureRing::Compliance))
    );
}

/// Step 7: Success when all conditions pass.
#[test]
fn test_order_step_7_fully_valid_activation_succeeds() {
    let (signing_key, verifying_key) = fixed_keypair();
    let local_id = derive_local_installation_id_hash(None);

    let clock = Arc::new(FixedTestClock {
        now_unix_secs: 1000,
    });

    let activation =
        SignedActivation::create_signed(FeatureRing::Compliance, local_id, 2000, &signing_key);

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(local_id);

    assert_eq!(gate.check_ring(FeatureRing::Compliance), Ok(()));
}
