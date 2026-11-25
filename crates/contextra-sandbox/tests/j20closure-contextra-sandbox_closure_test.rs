//! Closure integration tests for `requires_approval` and `pure_merge_operator` in `contextra-sandbox`.

use contextra_sandbox::{
    ApprovalRequest, ApprovalRisk, ApprovalStatus, MergeOperatorCapabilities, WasmCapabilities,
    WasmMergeFunction,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_requires_approval_direct_and_auto_approve_path() -> TestResult {
    // 1. Low risk: default capabilities do not require approval
    let low_caps = WasmCapabilities::default();
    let low_req = ApprovalRequest::new("req-low".to_string(), &low_caps, 1000, 5000);
    assert_eq!(low_req.risk, ApprovalRisk::Low);
    assert!(
        !low_req.requires_approval(),
        "Low risk requests must not require approval"
    );

    let auto_approved = low_req.auto_approve_if_low_risk("system_daemon".to_string(), 1100)?;
    assert_eq!(
        auto_approved.status,
        ApprovalStatus::Approved {
            approved_by: "system_daemon".to_string(),
            approved_at_unix_ms: 1100,
        }
    );

    // 2. Elevated risk: filesystem access requires approval
    let fs_caps = WasmCapabilities {
        allow_filesystem: true,
        ..Default::default()
    };
    let fs_req = ApprovalRequest::new("req-elevated".to_string(), &fs_caps, 1000, 5000);
    assert_eq!(fs_req.risk, ApprovalRisk::Elevated);
    assert!(
        fs_req.requires_approval(),
        "Elevated risk requests must require approval"
    );

    let fs_auto = fs_req.auto_approve_if_low_risk("system_daemon".to_string(), 1100)?;
    assert_eq!(
        fs_auto.status,
        ApprovalStatus::Pending,
        "Elevated risk request must remain Pending"
    );

    // 3. High risk: network and egress require approval
    let net_caps = WasmCapabilities {
        allow_network: true,
        ..Default::default()
    };
    let net_req = ApprovalRequest::new("req-high-net".to_string(), &net_caps, 1000, 5000);
    assert_eq!(net_req.risk, ApprovalRisk::High);
    assert!(
        net_req.requires_approval(),
        "High risk network requests must require approval"
    );

    let egress_caps = WasmCapabilities {
        allow_cloud_egress: true,
        ..Default::default()
    };
    let egress_req = ApprovalRequest::new("req-high-egress".to_string(), &egress_caps, 1000, 5000);
    assert_eq!(egress_req.risk, ApprovalRisk::High);
    assert!(
        egress_req.requires_approval(),
        "High risk cloud egress requests must require approval"
    );

    Ok(())
}

#[tokio::test]
async fn test_pure_merge_operator_capabilities_and_execution() -> TestResult {
    // Direct invocation of WasmCapabilities::pure_merge_operator()
    let pure_caps = WasmCapabilities::pure_merge_operator();
    assert!(!pure_caps.allow_stdout);
    assert!(!pure_caps.allow_stderr);
    assert!(!pure_caps.allow_filesystem);
    assert!(!pure_caps.allow_network);
    assert!(!pure_caps.allow_clock);
    assert!(!pure_caps.allow_cloud_egress);
    assert!(pure_caps.random_seed.is_none());
    assert_eq!(pure_caps.max_memory_pages, 16);
    assert_eq!(pure_caps.max_fuel, 10_000_000);
    assert_eq!(pure_caps.max_wall_clock_ms, 5_000);

    // Delegated call via MergeOperatorCapabilities::pure()
    let delegated_caps = MergeOperatorCapabilities::pure();
    assert_eq!(pure_caps.allow_stdout, delegated_caps.allow_stdout);
    assert_eq!(pure_caps.allow_clock, delegated_caps.allow_clock);
    assert_eq!(pure_caps.max_fuel, delegated_caps.max_fuel);

    // End-to-end WasmMergeFunction execution path
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                ;; Read stdin
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (i32.const 1024))
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 16)))

                ;; Write result to stdout
                (i32.store (i32.const 200) (i32.const 0x4b4f534f)) ;; "OSOK"
                (i32.store (i32.const 0) (i32.const 200))
                (i32.store (i32.const 4) (i32.const 4))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 16)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;

    let merge_fn = WasmMergeFunction::new(wasm_bytes)?;
    let output = merge_fn.merge(b"left", b"right").await?;
    assert_eq!(&output[..], b"OSOK");

    Ok(())
}
