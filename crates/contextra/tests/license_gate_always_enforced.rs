#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::builder::ContextraBuilder;
use contextra::performance_profile::{PerformanceProfile, VectorDeleteMode};
use contextra_core::error::ContextraError;
use contextra_license::SignedLicenseGate;
use contextra_ports::license::FeatureRing;
use contextra_store::lsm::config::DurabilityMode;

#[tokio::test]
async fn open_without_gate_never_grants_sovereign() {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_lic_always_1_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let db = contextra::open(&tmp_path).await.unwrap();
    assert!(!db.config().deletion_proof_active);

    let _ = std::fs::remove_dir_all(&tmp_path);
}

#[tokio::test]
async fn compliance_profile_without_signed_license_returns_error() {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_lic_always_2_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let res = ContextraBuilder::new(4)
        .with_storage_path(&tmp_path)
        .with_performance_profile(PerformanceProfile::Compliance)
        .build()
        .await;

    assert!(res.is_err());
    let err = res.err().unwrap();
    match err {
        ContextraError::PolicyViolation(msg) => {
            assert!(
                msg.contains("Sovereign") || msg.contains("requires a valid license"),
                "Expected license error message, got: {msg}"
            );
        }
        _ => panic!("Expected PolicyViolation error, got: {:?}", err),
    }

    let _ = std::fs::remove_dir_all(&tmp_path);
}

#[tokio::test]
async fn compliance_profile_with_valid_signed_license_activates_deletion_proof() {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_lic_always_3_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let (payload_bytes, signature, verifying_key_bytes) =
        SignedLicenseGate::create_test_signed_payload(
            vec![FeatureRing::Sovereign, FeatureRing::Compliance],
            None,
        );

    let db = ContextraBuilder::new(4)
        .with_storage_path(&tmp_path)
        .with_performance_profile(PerformanceProfile::Compliance)
        .with_signed_license(&payload_bytes, &signature, &verifying_key_bytes)
        .build()
        .await
        .expect("Build should succeed with valid signed license");

    assert!(db.config().deletion_proof_active);
    assert_eq!(db.config().durability_mode, DurabilityMode::Full);
    assert_eq!(
        db.config().vector_delete_mode,
        VectorDeleteMode::SynchronousRepair
    );

    let _ = std::fs::remove_dir_all(&tmp_path);
}

#[tokio::test]
async fn expired_signature_rejected() {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_lic_always_4_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let past_timestamp = 1000;
    let (payload_bytes, signature, verifying_key_bytes) =
        SignedLicenseGate::create_test_signed_payload(
            vec![FeatureRing::Sovereign, FeatureRing::Compliance],
            Some(past_timestamp),
        );

    let res = ContextraBuilder::new(4)
        .with_storage_path(&tmp_path)
        .with_performance_profile(PerformanceProfile::Compliance)
        .with_signed_license(&payload_bytes, &signature, &verifying_key_bytes)
        .build()
        .await;

    assert!(res.is_err());
    let err = res.err().unwrap();
    match err {
        ContextraError::PolicyViolation(msg) => {
            assert!(
                msg.contains("expired") || msg.contains("1000"),
                "Expected expired error message, got: {msg}"
            );
        }
        _ => panic!("Expected PolicyViolation error, got: {:?}", err),
    }

    let _ = std::fs::remove_dir_all(&tmp_path);
}

#[tokio::test]
async fn tampered_signature_rejected() {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_lic_always_5_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let (payload_bytes, mut signature, verifying_key_bytes) =
        SignedLicenseGate::create_test_signed_payload(
            vec![FeatureRing::Sovereign, FeatureRing::Compliance],
            None,
        );
    signature[0] ^= 0xFF; // tamper

    let res = ContextraBuilder::new(4)
        .with_storage_path(&tmp_path)
        .with_performance_profile(PerformanceProfile::Compliance)
        .with_signed_license(&payload_bytes, &signature, &verifying_key_bytes)
        .build()
        .await;

    assert!(res.is_err());
    let err = res.err().unwrap();
    match err {
        ContextraError::PolicyViolation(msg) => {
            assert!(
                msg.contains("verification failed") || msg.contains("signature"),
                "Expected signature verification error message, got: {msg}"
            );
        }
        _ => panic!("Expected PolicyViolation error, got: {:?}", err),
    }

    let _ = std::fs::remove_dir_all(&tmp_path);
}

#[tokio::test]
async fn explicit_conflicting_user_value_to_profile_returns_policy_violation() {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_lic_always_6_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let (payload_bytes, signature, verifying_key_bytes) =
        SignedLicenseGate::create_test_signed_payload(
            vec![FeatureRing::Sovereign, FeatureRing::Compliance],
            None,
        );

    let config = contextra_engine::ContextraConfig {
        durability_mode: DurabilityMode::MemoryOnly,
        ..Default::default()
    };

    let res = ContextraBuilder::new(4)
        .with_storage_path(&tmp_path)
        .with_performance_profile(PerformanceProfile::Compliance)
        .with_signed_license(&payload_bytes, &signature, &verifying_key_bytes)
        .with_config(config)
        .build()
        .await;

    assert!(res.is_err());

    let _ = std::fs::remove_dir_all(&tmp_path);
}
