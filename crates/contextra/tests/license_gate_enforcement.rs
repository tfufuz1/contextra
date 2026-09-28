#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use contextra::builder::ContextraBuilder;
use contextra::performance_profile::PerformanceProfile;
use contextra_core::error::ContextraError;
use contextra_license::OpenFastGate;
use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};

/// Test case a: Compliance profile without explicit with_license_gate fails using default OpenFastGate.
#[tokio::test]
async fn test_compliance_profile_without_gate_fails() {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_lic_test_a_{}_{}",
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
                "Expected license error message, got: {}",
                msg
            );
        }
        _ => panic!("Expected PolicyViolation error, got: {:?}", err),
    }

    let _ = std::fs::remove_dir_all(&tmp_path);
}

/// Test case b: Compliance profile with explicit OpenFastGate fails (regression protection).
#[tokio::test]
async fn test_compliance_profile_with_explicit_open_fast_gate_fails() {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_lic_test_b_{}_{}",
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
        .with_license_gate(Arc::new(OpenFastGate))
        .build()
        .await;

    assert!(res.is_err());
    let err = res.err().unwrap();
    match err {
        ContextraError::PolicyViolation(msg) => {
            assert!(
                msg.contains("Sovereign") || msg.contains("requires a valid license"),
                "Expected license error message, got: {}",
                msg
            );
        }
        _ => panic!("Expected PolicyViolation error, got: {:?}", err),
    }

    let _ = std::fs::remove_dir_all(&tmp_path);
}

/// Test case c: Builder without with_performance_profile builds successfully (backward compatibility).
#[tokio::test]
async fn test_builder_without_performance_profile_succeeds() {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_lic_test_c_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&tmp_path);

    let res = ContextraBuilder::new(4)
        .with_storage_path(&tmp_path)
        .build()
        .await;

    assert!(
        res.is_ok(),
        "Expected build to succeed, got: {:?}",
        res.err()
    );
    let db = res.unwrap();
    assert_eq!(db.len().await.unwrap(), 0);

    let _ = std::fs::remove_dir_all(&tmp_path);
}

struct AllowAllGate;

impl LicenseGate for AllowAllGate {
    fn check_ring(&self, _ring: FeatureRing) -> Result<(), LicenseError> {
        Ok(())
    }
}

/// Test case d: Compliance profile with custom LicenseGate test-double allowing Compliance builds successfully.
#[tokio::test]
async fn test_compliance_profile_with_allow_all_gate_succeeds() {
    let tmp_path = std::env::temp_dir().join(format!(
        "contextra_lic_test_d_{}_{}",
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
        .with_license_gate(Arc::new(AllowAllGate))
        .build()
        .await;

    assert!(
        res.is_ok(),
        "Expected build to succeed with AllowAllGate, got: {:?}",
        res.err()
    );
    let db = res.unwrap();
    assert_eq!(db.len().await.unwrap(), 0);

    let _ = std::fs::remove_dir_all(&tmp_path);
}
