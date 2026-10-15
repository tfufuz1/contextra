#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::builder::ContextraBuilder;
use contextra::performance_profile::PerformanceProfile;
use contextra_types::ContextraError;
use contextra_license::SignedLicenseGate;
use contextra_ports::clock::ManualClock;
use contextra_ports::license::FeatureRing;
use contextra_ports::rng::SeededRng;
use contextra_ports::Rng;

#[tokio::test]
async fn test_license_wiring_matrix() -> Result<(), Box<dyn std::error::Error>> {
    let seed: u64 = 20261013_01;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000); // fixed deterministic timestamp
    println!("e2e_wiring_license seed: {}, clock_nanos: {}", seed, clock.now_nanos());

    let tmp_dir = std::env::temp_dir().join(format!("e2e_wiring_license_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    // 1. Build Compliance profile without license => Err
    let res_no_lic = ContextraBuilder::new(4)
        .with_storage_path(tmp_dir.join("no_lic"))
        .with_performance_profile(PerformanceProfile::Compliance)
        .build()
        .await;

    assert!(
        res_no_lic.is_err(),
        "Compliance profile without license must fail"
    );
    match res_no_lic.err().unwrap() {
        ContextraError::PolicyViolation(msg) => {
            assert!(
                msg.contains("Sovereign") || msg.contains("requires a valid license"),
                "Unexpected error message: {msg}"
            );
        }
        err => panic!("Expected PolicyViolation, got: {err:?}"),
    }

    // 2. Build Compliance profile with valid signed license => ok
    let (payload_bytes, signature, verifying_key_bytes) =
        SignedLicenseGate::create_test_signed_payload(
            vec![FeatureRing::Sovereign, FeatureRing::Compliance],
            None,
        );

    let db_valid = ContextraBuilder::new(4)
        .with_storage_path(tmp_dir.join("valid_lic"))
        .with_performance_profile(PerformanceProfile::Compliance)
        .with_signed_license(&payload_bytes, &signature, &verifying_key_bytes)
        .build()
        .await
        .expect("Build with valid signed license must succeed");

    assert_ne!(
        db_valid.config().erasure_mode,
        contextra_ports::erasure::ErasureMode::Off,
        "Compliance profile with signed license must activate erasure_mode"
    );
    db_valid.close().await?;

    // 3. Build Compliance profile with expired license => Err
    let past_timestamp = 1000;
    let (exp_payload, exp_sig, exp_key) = SignedLicenseGate::create_test_signed_payload(
        vec![FeatureRing::Sovereign, FeatureRing::Compliance],
        Some(past_timestamp),
    );

    let res_expired = ContextraBuilder::new(4)
        .with_storage_path(tmp_dir.join("expired_lic"))
        .with_performance_profile(PerformanceProfile::Compliance)
        .with_signed_license(&exp_payload, &exp_sig, &exp_key)
        .build()
        .await;

    assert!(res_expired.is_err(), "Expired license must be rejected");
    match res_expired.err().unwrap() {
        ContextraError::PolicyViolation(msg) => {
            assert!(
                msg.contains("expired") || msg.contains("1000"),
                "Unexpected error message: {msg}"
            );
        }
        err => panic!("Expected PolicyViolation, got: {err:?}"),
    }

    // 4. Build Compliance profile with tampered signature => Err
    let (mut tamp_sig_bytes, tamp_sig, tamp_key) = SignedLicenseGate::create_test_signed_payload(
        vec![FeatureRing::Sovereign, FeatureRing::Compliance],
        None,
    );
    tamp_sig_bytes[0] ^= 0xFF; // tamper payload

    let res_tampered = ContextraBuilder::new(4)
        .with_storage_path(tmp_dir.join("tampered_lic"))
        .with_performance_profile(PerformanceProfile::Compliance)
        .with_signed_license(&tamp_sig_bytes, &tamp_sig, &tamp_key)
        .build()
        .await;

    assert!(res_tampered.is_err(), "Tampered license payload must be rejected");

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}
