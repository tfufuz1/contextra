#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_sign_loss,
    clippy::indexing_slicing
)]

use std::sync::Arc;

use contextra_license::{
    derive_local_installation_id_hash, FeatureRing, LicenseError, LicenseGate, SignedActivation,
    SignedLicenseGate, StepVerificationError,
};
use contextra_ports::clock::Clock;
use ed25519_dalek::SigningKey;

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

fn generate_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let seed = [99u8; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

#[test]
fn test_copied_activation_to_foreign_host_fails() {
    let (signing_key, verifying_key) = generate_keypair();
    let host_a_id = [0xAA; 32];
    let host_b_id = [0xBB; 32];

    let clock = Arc::new(TestClock {
        now_unix_secs: 1000,
    });

    // Activation generated for Host A
    let activation_host_a = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        host_a_id,
        2000,
        &signing_key,
    );

    // Copying activation record to Host B (which has local installation ID host_b_id)
    let gate_host_b =
        SignedLicenseGate::from_activation_with_clock(activation_host_a, verifying_key, clock)
            .with_local_installation_id(host_b_id);

    let err = gate_host_b
        .check_ring(FeatureRing::Sovereign)
        .expect_err("copying to foreign host must fail");

    assert_eq!(err, LicenseError::NotActivated(FeatureRing::Sovereign));

    // Verify error string formatting does NOT contain raw host_a_id or host_b_id bytes
    let err_fmt = format!("{}", err);
    assert!(!err_fmt.contains("AA"));
    assert!(!err_fmt.contains("BB"));
}

#[test]
fn test_installation_mismatch_step_error_conversion_leaks_no_raw_hash() {
    let step_err = StepVerificationError::InstallationMismatch(FeatureRing::Compliance);
    let lic_err = step_err.into_license_error();

    assert_eq!(lic_err, LicenseError::NotActivated(FeatureRing::Compliance));

    let display_str = format!("{}", lic_err);
    assert!(!display_str.contains("installation.id"));
    assert!(!display_str.contains("hardware"));
}

#[test]
fn test_derive_local_installation_id_hash_stability() {
    let tmp_dir = std::env::temp_dir().join(format!(
        "contextra_inst_test_{}_{}",
        std::process::id(),
        rand::random::<u32>()
    ));

    let hash1 = derive_local_installation_id_hash(Some(&tmp_dir));
    let hash2 = derive_local_installation_id_hash(Some(&tmp_dir));

    assert_eq!(hash1, hash2);

    let fallback_hash1 = derive_local_installation_id_hash(None);
    let fallback_hash2 = derive_local_installation_id_hash(None);

    assert_eq!(fallback_hash1, fallback_hash2);
    assert_ne!(hash1, fallback_hash1);

    let _ = std::fs::remove_dir_all(&tmp_dir);
}
