#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration & Property-Based Tests for memfuse-sandbox (§4.18, §10.14).

use memfuse_sandbox::{SandboxError, WasmCapabilities, WasmExecutor};
use std::time::Duration;

#[tokio::test]
async fn test_wasm_memory_isolation_property_variations() {
    let executor = WasmExecutor::new().expect("WasmExecutor init");

    let page_requests = vec![(1, 1, true), (2, 1, false), (10, 5, false), (4, 16, true)];

    for (requested_pages, max_pages, should_succeed) in page_requests {
        let wat = format!(
            r#"
            (module
                (memory {})
                (func (export "_start"))
            )
        "#,
            requested_pages
        );

        let wasm_bytes = wat::parse_str(&wat).expect("parse WAT");
        let caps = WasmCapabilities {
            max_memory_pages: max_pages,
            ..Default::default()
        };

        let res = executor
            .execute(&wasm_bytes, b"", &caps, Duration::from_secs(2))
            .await;

        if should_succeed {
            assert!(
                res.is_ok(),
                "Execution should succeed for requested {} pages <= max {}",
                requested_pages,
                max_pages
            );
        } else {
            assert!(
                matches!(
                    res,
                    Err(SandboxError::MemoryExceeded { .. }) | Err(SandboxError::Runtime(_))
                ),
                "Execution should fail for requested {} pages > max {}",
                requested_pages,
                max_pages
            );
        }
    }
}

#[tokio::test]
async fn test_wasm_fuel_exhaustion_returns_error_property() {
    let wat = r#"
        (module
            (func (export "_start")
                (loop (br 0))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("parse WAT");
    let executor = WasmExecutor::new().expect("WasmExecutor init");

    for fuel_limit in [100u64, 1_000u64, 50_000u64] {
        let caps = WasmCapabilities {
            max_fuel: fuel_limit,
            ..Default::default()
        };

        let res = executor
            .execute(&wasm_bytes, b"", &caps, Duration::from_secs(5))
            .await;

        assert!(
            matches!(res, Err(SandboxError::FuelExhausted { consumed }) if consumed > 0),
            "Expected FuelExhausted for fuel_limit {}, got {:?}",
            fuel_limit,
            res
        );
    }
}

#[tokio::test]
async fn test_cloud_egress_strict_capability_isolation() {
    let wat = r#"
        (module
            (import "memfuse" "host_cloud_query" (func $host_cloud_query (result i32)))
            (func (export "_start")
                (drop (call $host_cloud_query))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("parse WAT");
    let executor = WasmExecutor::new().expect("WasmExecutor init");

    // Disabled capability
    let caps_denied = WasmCapabilities {
        allow_cloud_egress: false,
        ..Default::default()
    };
    let res_denied = executor
        .execute(&wasm_bytes, b"", &caps_denied, Duration::from_secs(2))
        .await;

    assert!(
        matches!(
            res_denied,
            Err(SandboxError::CapabilityViolation { ref capability }) if capability == "allow_cloud_egress"
        ),
        "Expected CapabilityViolation for allow_cloud_egress, got {:?}",
        res_denied
    );

    // Enabled capability
    let caps_allowed = WasmCapabilities {
        allow_cloud_egress: true,
        ..Default::default()
    };
    let res_allowed = executor
        .execute(&wasm_bytes, b"", &caps_allowed, Duration::from_secs(2))
        .await;

    assert!(
        res_allowed.is_ok(),
        "Expected execution to succeed when allow_cloud_egress is true, got {:?}",
        res_allowed
    );
}
