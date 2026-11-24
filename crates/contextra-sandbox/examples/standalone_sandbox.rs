//! Standalone example demonstrating contextra-sandbox (Ring 2).
//!
//! Demonstrates approval workflow and pure WASM execution with fuel/wall-clock enforcement.

use contextra_sandbox::{
    classify_risk, ApprovalRequest, ApprovalRisk, ApprovalStatus, WasmCapabilities,
};

#[cfg(feature = "wasm")]
use contextra_sandbox::{WasmExecutor, WasmMergeFunction};
#[cfg(feature = "wasm")]
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== contextra-sandbox standalone demonstration ===");

    // 1. Capability & Approval Workflow (Zero heavyweight dependencies required)
    let default_caps = WasmCapabilities::default();
    println!(
        "Default capabilities risk: {:?}",
        classify_risk(&default_caps)
    );
    assert_eq!(classify_risk(&default_caps), ApprovalRisk::Low);

    let elevated_caps = WasmCapabilities {
        allow_network: true,
        ..Default::default()
    };
    println!(
        "Elevated capabilities risk: {:?}",
        classify_risk(&elevated_caps)
    );
    assert_eq!(classify_risk(&elevated_caps), ApprovalRisk::High);

    let mut req = ApprovalRequest::new(
        "req-standalone-001".to_string(),
        &elevated_caps,
        1_000,
        10_000,
    );
    assert!(req.requires_approval());
    println!(
        "Created approval request: id={}, risk={:?}, summary={}",
        req.request_id, req.risk, req.requested_capabilities_summary
    );

    req = req.approve("security_auditor".to_string(), 2_000)?;
    assert!(matches!(req.status, ApprovalStatus::Approved { .. }));
    println!("Approval request transitioned to: {:?}", req.status);

    // 2. WASM Execution (available when feature = "wasm" is enabled)
    #[cfg(feature = "wasm")]
    {
        println!("\nExecuting pure WASM guest code via WasmExecutor...");
        let executor = WasmExecutor::new()?;

        // Minimal pure WASM binary (WAT: exports _start, does nothing)
        let wat_bytes = r#"
            (module
                (func (export "_start"))
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_bytes)?;

        let caps = WasmCapabilities::default();
        let output = executor
            .execute(&wasm_bytes, b"input", &caps, Duration::from_secs(2))
            .await?;

        println!(
            "Execution succeeded: fuel_consumed={}, stdout_len={}",
            output.fuel_consumed,
            output.stdout.len()
        );

        // Pure merge operator demonstration
        println!("Demonstrating WasmMergeFunction...");
        let merge_wat = r#"
            (module
                (import "wasi_snapshot_preview1" "fd_read" (func $fd_read (param i32 i32 i32 i32) (result i32)))
                (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (func (export "_start"))
            )
        "#;
        let merge_wasm = wat::parse_str(merge_wat)?;
        let merge_fn = WasmMergeFunction::new(merge_wasm)?;
        let merged = merge_fn.merge(b"v1", b"v2").await?;
        println!("Merge finished, output bytes: {:?}", merged);
    }

    #[cfg(not(feature = "wasm"))]
    {
        println!("\nNote: 'wasm' feature is disabled; runtime execution bypassed.");
    }

    println!("\ncontextra-sandbox demonstration completed successfully.");
    Ok(())
}
