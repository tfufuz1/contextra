#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;

use contextra_license::{
    FeatureRing, LicenseError, LicenseGate, SignedActivation, SignedLicenseGate,
};
use contextra_ports::clock::Clock;
use ed25519_dalek::SigningKey;

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

#[test]
fn test_error_display_and_debug_contain_no_sensitive_hashes_or_reasons() {
    let (signing_key, verifying_key) = fixed_keypair();
    let expected_hash = [0xAAu8; 32];
    let local_hash = [0xBBu8; 32];

    let clock = Arc::new(FixedTestClock);
    let activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, expected_hash, 2000, &signing_key);

    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(local_hash);

    let err = gate
        .check_ring(FeatureRing::Sovereign)
        .expect_err("step 3 hash mismatch must fail");

    assert_eq!(err, LicenseError::NotActivated(FeatureRing::Sovereign));

    let debug_str = format!("{err:?}");
    let display_str = format!("{err}");

    let hex_expected = "a".repeat(64);
    let hex_local = "b".repeat(64);

    // Verify neither hex representation nor sensitive mismatch terms are present
    assert!(
        !debug_str.contains(&hex_expected),
        "Debug output leaked expected installation hash"
    );
    assert!(
        !debug_str.contains(&hex_local),
        "Debug output leaked local installation hash"
    );
    assert!(
        !debug_str.contains("mismatch"),
        "Debug output leaked mismatch reason"
    );
    assert!(
        !debug_str.contains("installation_id"),
        "Debug output leaked installation_id field name"
    );

    assert!(
        !display_str.contains(&hex_expected),
        "Display output leaked expected installation hash"
    );
    assert!(
        !display_str.contains(&hex_local),
        "Display output leaked local installation hash"
    );
    assert!(
        !display_str.contains("mismatch"),
        "Display output leaked mismatch reason"
    );
    assert!(
        !display_str.contains("installation_id"),
        "Display output leaked installation_id field name"
    );
}

#[test]
fn test_missing_local_id_error_contains_no_sensitive_info() {
    let (signing_key, verifying_key) = fixed_keypair();
    let expected_hash = [0xCCu8; 32];

    let clock = Arc::new(FixedTestClock);
    let activation =
        SignedActivation::create_signed(FeatureRing::Sovereign, expected_hash, 2000, &signing_key);

    // Gate created WITHOUT calling with_local_installation_id
    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock);

    let err = gate
        .check_ring(FeatureRing::Sovereign)
        .expect_err("missing local installation ID must fail");

    assert_eq!(err, LicenseError::NotActivated(FeatureRing::Sovereign));

    let debug_str = format!("{err:?}");
    let display_str = format!("{err}");

    let hex_expected = "c".repeat(64);

    assert!(
        !debug_str.contains(&hex_expected),
        "Debug output leaked expected hash"
    );
    assert!(
        !debug_str.contains("missing"),
        "Debug output leaked specific missing ID reason"
    );
    assert!(
        !display_str.contains(&hex_expected),
        "Display output leaked expected hash"
    );
    assert!(
        !display_str.contains("missing"),
        "Display output leaked specific missing ID reason"
    );
}
