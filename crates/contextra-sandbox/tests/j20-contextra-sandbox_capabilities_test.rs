//! Integration tests for Pure Merge Operator Capabilities (`pure_merge_operator` production path).

use contextra_sandbox::{WasmCapabilities, WasmMergeFunction};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Verifies that `WasmCapabilities::pure_merge_operator()` sets strict safety defaults.
#[test]
fn test_pure_merge_operator_capabilities_preset() {
    let caps = WasmCapabilities::pure_merge_operator();

    assert!(!caps.allow_stdout, "stdout must be disabled by default for pure capabilities");
    assert!(!caps.allow_stderr, "stderr must be disabled by default for pure capabilities");
    assert!(!caps.allow_filesystem, "filesystem must be disabled");
    assert!(!caps.allow_network, "network must be disabled");
    assert!(!caps.allow_clock, "clock must be disabled");
    assert!(!caps.allow_cloud_egress, "cloud egress must be disabled");
    assert!(caps.random_seed.is_none(), "random_seed must be None");
    assert_eq!(caps.max_fuel, 10_000_000);
    assert_eq!(caps.max_wall_clock_ms, 5_000);
    assert_eq!(caps.max_memory_pages, 16);
}

/// Verifies `WasmMergeFunction` execution using the production capability wiring path.
#[tokio::test]
async fn test_wasm_merge_function_production_capabilities_path() -> TestResult {
    // Pure WASM merge function that reads length-prefixed binary inputs existing/new and appends them
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                ;; Read stdin (first 4 bytes: len(existing), then existing bytes, next 4 bytes: len(new), then new bytes)
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (i32.const 1024))
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 16)))

                ;; Write hardcoded merged marker to stdout
                ;; iovec 0: ptr=200, len=6 ("MERGED")
                (i32.store (i32.const 200) (i32.const 0x4752454d)) ;; "MERG"
                (i32.store (i32.const 204) (i32.const 0x4445))     ;; "ED"
                (i32.store (i32.const 0) (i32.const 200))
                (i32.store (i32.const 4) (i32.const 6))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 16)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;

    // WasmMergeFunction::new uses MergeOperatorCapabilities::pure_with_result_channel(),
    // which internally delegates to WasmCapabilities::pure_merge_operator().
    let merge_fn = WasmMergeFunction::new(wasm_bytes)?;
    let result = merge_fn.merge(b"val1", b"val2").await?;

    assert_eq!(&result[..], b"MERGED");
    Ok(())
}
