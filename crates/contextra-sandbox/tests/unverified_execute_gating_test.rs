//! Integration test for `unverified-execute` feature gating (§4.18).

use contextra_sandbox::{MergeOperatorCapabilities, WasmCapabilities, WasmExecutor};
#[cfg(not(feature = "unverified-execute"))]
use contextra_sandbox::SandboxError;
use std::time::Duration;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_unverified_execute_denies_high_risk_without_feature() -> TestResult {
    let wat = r#"
        (module
            (func (export "_start"))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;

    // Low-risk capabilities (default / pure merge) MUST succeed when unverified-execute feature is NOT enabled or IS enabled
    let pure_caps = MergeOperatorCapabilities::pure();
    let pure_res = executor
        .execute(&wasm_bytes, b"", &pure_caps, Duration::from_secs(1))
        .await;
    assert!(
        pure_res.is_ok(),
        "Pure/low-risk execution MUST succeed regardless of feature, got: {:?}",
        pure_res
    );

    // Elevated-risk capability (filesystem) MUST fail without unverified-execute feature and succeed with feature
    let fs_caps = WasmCapabilities {
        allow_filesystem: true,
        ..Default::default()
    };
    let fs_res = executor
        .execute(&wasm_bytes, b"", &fs_caps, Duration::from_secs(1))
        .await;
    #[cfg(not(feature = "unverified-execute"))]
    assert!(
        matches!(
            fs_res,
            Err(SandboxError::CapabilityViolation { ref capability })
                if capability.contains("unverified execution")
        ),
        "Elevated-risk filesystem execution MUST be denied without feature 'unverified-execute', got: {:?}",
        fs_res
    );
    #[cfg(feature = "unverified-execute")]
    assert!(
        fs_res.is_ok(),
        "Elevated-risk filesystem execution MUST succeed with feature 'unverified-execute', got: {:?}",
        fs_res
    );

    // High-risk capability (network) MUST fail without unverified-execute feature and succeed with feature
    let net_caps = WasmCapabilities {
        allow_network: true,
        ..Default::default()
    };
    let net_res = executor
        .execute(&wasm_bytes, b"", &net_caps, Duration::from_secs(1))
        .await;
    #[cfg(not(feature = "unverified-execute"))]
    assert!(
        matches!(
            net_res,
            Err(SandboxError::CapabilityViolation { ref capability })
                if capability.contains("unverified execution")
        ),
        "High-risk network execution MUST be denied without feature 'unverified-execute', got: {:?}",
        net_res
    );
    #[cfg(feature = "unverified-execute")]
    assert!(
        net_res.is_ok(),
        "High-risk network execution MUST succeed with feature 'unverified-execute', got: {:?}",
        net_res
    );

    // High-risk capability (cloud egress) MUST fail without unverified-execute feature and succeed with feature
    let egress_caps = WasmCapabilities {
        allow_cloud_egress: true,
        ..Default::default()
    };
    let egress_res = executor
        .execute(&wasm_bytes, b"", &egress_caps, Duration::from_secs(1))
        .await;
    #[cfg(not(feature = "unverified-execute"))]
    assert!(
        matches!(
            egress_res,
            Err(SandboxError::CapabilityViolation { ref capability })
                if capability.contains("unverified execution")
        ),
        "High-risk cloud egress execution MUST be denied without feature 'unverified-execute', got: {:?}",
        egress_res
    );
    #[cfg(feature = "unverified-execute")]
    assert!(
        egress_res.is_ok(),
        "High-risk cloud egress execution MUST succeed with feature 'unverified-execute', got: {:?}",
        egress_res
    );

    Ok(())
}
