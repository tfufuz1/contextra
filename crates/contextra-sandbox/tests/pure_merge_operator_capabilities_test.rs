//! Integration tests for MergeOperatorCapabilities::pure() preset (§4.18).

use contextra_sandbox::{MergeOperatorCapabilities, WasmCapabilities, WasmExecutor};
use std::time::Duration;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Verifies that clock_time_get is denied when allow_clock=false, returning ERRNO_ACCES (2).
#[tokio::test]
async fn test_pure_merge_operator_denies_clock_access() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "clock_time_get"
                (func $clock_time_get (param i32 i64 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (local $res i32)
                (local.set $res (call $clock_time_get (i32.const 1) (i64.const 0) (i32.const 100)))
                ;; Store errno at offset 200
                (i32.store (i32.const 200) (local.get $res))
                ;; iovec 0: ptr=200, len=4
                (i32.store (i32.const 0) (i32.const 200))
                (i32.store (i32.const 4) (i32.const 4))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 300)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;

    // Enable stdout explicitly so we can inspect the WASI return code
    let caps = WasmCapabilities {
        allow_stdout: true,
        ..MergeOperatorCapabilities::pure()
    };

    let output = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await?;

    let errno = i32::from_le_bytes(output.stdout[0..4].try_into()?);
    assert_eq!(errno, 2, "Expected ERRNO_ACCES (2) for clock_time_get under pure capabilities");

    // Under exact MergeOperatorCapabilities::pure() without allow_stdout, output is empty
    let pure_caps = MergeOperatorCapabilities::pure();
    let pure_output = executor
        .execute(&wasm_bytes, b"", &pure_caps, Duration::from_secs(1))
        .await?;
    assert!(pure_output.stdout.is_empty(), "Stdout must be empty when allow_stdout=false");

    Ok(())
}

/// Verifies that random_get returns ERRNO_NOSYS (52) when random_seed=None.
#[tokio::test]
async fn test_pure_merge_operator_denies_random_access() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "random_get"
                (func $random_get (param i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (local $res i32)
                (local.set $res (call $random_get (i32.const 100) (i32.const 16)))
                ;; Store errno at offset 200
                (i32.store (i32.const 200) (local.get $res))
                ;; iovec 0: ptr=200, len=4
                (i32.store (i32.const 0) (i32.const 200))
                (i32.store (i32.const 4) (i32.const 4))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 300)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;

    // Enable stdout explicitly so we can inspect the WASI return code
    let caps = WasmCapabilities {
        allow_stdout: true,
        ..MergeOperatorCapabilities::pure()
    };

    let output = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await?;

    let errno = i32::from_le_bytes(output.stdout[0..4].try_into()?);
    assert_eq!(errno, 52, "Expected ERRNO_NOSYS (52) for random_get under pure capabilities");

    Ok(())
}

/// Verifies that pure merge operator executes pure functional logic successfully.
#[tokio::test]
async fn test_pure_merge_operator_functional_execution() -> TestResult {
    // Pure WASM module that reads 2 integers from stdin and adds them
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                ;; iovec for fd_read: buf_ptr=100, buf_len=8
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (i32.const 8))
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 16)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = MergeOperatorCapabilities::pure();

    let input_bytes = [10u32.to_le_bytes(), 20u32.to_le_bytes()].concat();
    let output = executor
        .execute(&wasm_bytes, &input_bytes, &caps, Duration::from_secs(1))
        .await?;

    assert!(output.fuel_consumed > 0);
    Ok(())
}
