#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_license::{FeatureRing, LicenseError, LicenseGate, SignedLicenseGate};

#[test]
fn test_not_activated_error_formatting_contains_no_hashes_or_local_ids() {
    let gate = SignedLicenseGate::no_activation();

    let err = gate
        .check_ring(FeatureRing::Sovereign)
        .expect_err("should return NotActivated");

    assert!(matches!(
        err,
        LicenseError::NotActivated(FeatureRing::Sovereign)
    ));

    let display_str = format!("{}", err);
    let debug_str = format!("{:?}", err);

    let hex_hash = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    assert!(!display_str.contains("installation"));
    assert!(!display_str.contains("hash"));
    assert!(!display_str.contains(hex_hash));

    assert!(!debug_str.contains("installation"));
    assert!(!debug_str.contains("hash"));
    assert!(!debug_str.contains(hex_hash));
}
