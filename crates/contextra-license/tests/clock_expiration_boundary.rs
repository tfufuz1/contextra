#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_sign_loss,
    clippy::indexing_slicing
)]

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use contextra_license::{
    FeatureRing, LicenseError, LicenseGate, SignedActivation, SignedLicenseGate,
};
use contextra_ports::clock::Clock;
use ed25519_dalek::SigningKey;

struct AdjustableClock {
    unix_secs: AtomicI64,
}

impl AdjustableClock {
    fn new(secs: i64) -> Self {
        Self {
            unix_secs: AtomicI64::new(secs),
        }
    }

    fn set_secs(&self, secs: i64) {
        self.unix_secs.store(secs, Ordering::SeqCst);
    }
}

impl Clock for AdjustableClock {
    fn now_unix_nanos(&self) -> u64 {
        let secs = self.unix_secs.load(Ordering::SeqCst);
        (secs.max(0) as u64) * 1_000_000_000
    }

    fn monotonic_nanos(&self) -> u64 {
        0
    }
}

fn generate_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let seed = [123u8; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

#[test]
fn test_expiration_exact_boundary_conditions() {
    let (signing_key, verifying_key) = generate_keypair();
    let local_id = [10u8; 32];
    let expires_at = 1_700_000_000i64;

    let activation = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        local_id,
        expires_at,
        &signing_key,
    );

    let clock = Arc::new(AdjustableClock::new(1_600_000_000));

    let gate = SignedLicenseGate::from_activation_with_clock(
        activation,
        verifying_key,
        clock.clone(),
    )
    .with_local_installation_id(local_id);

    // 1 second before expiration -> Ok(())
    clock.set_secs(expires_at - 1);
    assert_eq!(gate.check_ring(FeatureRing::Sovereign), Ok(()));

    // Exact expiration boundary (now == expires_at) -> Expired
    clock.set_secs(expires_at);
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::Expired(expires_at))
    );

    // 1 second after expiration -> Expired
    clock.set_secs(expires_at + 1);
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::Expired(expires_at))
    );
}

#[test]
fn test_clock_drift_and_skew_tolerance() {
    let (signing_key, verifying_key) = generate_keypair();
    let local_id = [20u8; 32];
    let expires_at = 2_000_000_000i64;

    let activation = SignedActivation::create_signed(
        FeatureRing::Compliance,
        local_id,
        expires_at,
        &signing_key,
    );

    let clock = Arc::new(AdjustableClock::new(1_900_000_000));

    let gate = SignedLicenseGate::from_activation_with_clock(
        activation,
        verifying_key,
        clock.clone(),
    )
    .with_local_installation_id(local_id);

    // Positive clock skew (+300 seconds) still valid if well before expires_at
    clock.set_secs(1_900_000_000 + 300);
    assert_eq!(gate.check_ring(FeatureRing::Compliance), Ok(()));

    // Skew pushing clock past expires_at -> Expired
    clock.set_secs(expires_at + 300);
    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::Expired(expires_at))
    );
}

#[test]
fn test_system_clock_rollback_behavior() {
    let (signing_key, verifying_key) = generate_keypair();
    let local_id = [30u8; 32];
    let expires_at = 1_800_000_000i64;

    let activation = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        local_id,
        expires_at,
        &signing_key,
    );

    let clock = Arc::new(AdjustableClock::new(1_800_000_005)); // Initially past expiration

    let gate = SignedLicenseGate::from_activation_with_clock(
        activation,
        verifying_key,
        clock.clone(),
    )
    .with_local_installation_id(local_id);

    // Initially expired
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::Expired(expires_at))
    );

    // System clock rolled back to before expiration timestamp
    clock.set_secs(1_700_000_000);
    // Evaluating with adjusted clock uses injected Clock port value
    assert_eq!(gate.check_ring(FeatureRing::Sovereign), Ok(()));
}
