//! Integration tests for WasmMergeFunction (§4.18).

use contextra_sandbox::{SandboxError, WasmMergeFunction};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Test (a): Adder/Counter WASM module reads two u64 LE values from encoded input
/// and outputs their u64 LE sum.
#[tokio::test]
async fn test_wasm_merge_function_counter_sum() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (local $sum i64)
                ;; Setup iovec for fd_read at memory offset 0: buf_ptr=100, buf_len=24
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (i32.const 24))
                ;; Call fd_read(0, 0, 1, 50)
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 50)))

                ;; Input memory layout at offset 100:
                ;; 100..104: u32 len(existing)
                ;; 104..112: u64 val1 (existing)
                ;; 112..116: u32 len(new)
                ;; 116..124: u64 val2 (new)
                (local.set $sum
                    (i64.add
                        (i64.load (i32.const 104))
                        (i64.load (i32.const 116))
                    )
                )

                ;; Store sum u64 at memory offset 200
                (i64.store (i32.const 200) (local.get $sum))

                ;; Setup iovec for fd_write at memory offset 10: buf_ptr=200, buf_len=8
                (i32.store (i32.const 10) (i32.const 200))
                (i32.store (i32.const 14) (i32.const 8))
                ;; Call fd_write(1, 10, 1, 50)
                (drop (call $fd_write (i32.const 1) (i32.const 10) (i32.const 1) (i32.const 50)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let merge_fn = WasmMergeFunction::new(wasm_bytes)?;

    let existing = 1u64.to_le_bytes();
    let new = 2u64.to_le_bytes();

    let result_bytes = merge_fn.merge(&existing, &new).await?;
    assert_eq!(result_bytes.len(), 8);

    let sum = u64::from_le_bytes(result_bytes[..8].try_into()?);
    assert_eq!(sum, 3u64);

    Ok(())
}

/// Test (b): Infinite loop in WASM module returns SandboxError::FuelExhausted.
#[tokio::test]
async fn test_wasm_merge_function_infinite_loop_fuel_exhausted() -> TestResult {
    let wat = r#"
        (module
            (func (export "_start")
                (loop (br 0))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let merge_fn = WasmMergeFunction::new(wasm_bytes)?;

    let result = merge_fn.merge(b"existing_data", b"new_data").await;

    assert!(
        matches!(result, Err(SandboxError::FuelExhausted { consumed }) if consumed > 0),
        "Expected FuelExhausted error, got: {:?}",
        result
    );

    Ok(())
}

/// Test (c): Two consecutive calls with identical module and inputs produce byte-identical output.
#[tokio::test]
async fn test_wasm_merge_function_deterministic_output() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                ;; Read up to 64 bytes into offset 100
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (i32.const 64))
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 50)))

                ;; Echo back offset 100 (64 bytes) to stdout
                (i32.store (i32.const 10) (i32.const 100))
                (i32.store (i32.const 14) (i32.const 64))
                (drop (call $fd_write (i32.const 1) (i32.const 10) (i32.const 1) (i32.const 50)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let merge_fn = WasmMergeFunction::new(wasm_bytes)?;

    let existing = b"state_v1_payload_alpha";
    let new = b"state_v2_payload_beta";

    let run1 = merge_fn.merge(existing, new).await?;
    let run2 = merge_fn.merge(existing, new).await?;

    assert_eq!(
        run1, run2,
        "Consecutive invocations with same module and inputs must yield byte-identical outputs"
    );

    Ok(())
}

/// Test (d): Module calling clock_time_get receives ERRNO_ACCES (2) / clock access denied.
#[tokio::test]
async fn test_wasm_merge_function_clock_access_denied() -> TestResult {
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
                ;; Store errno u32 at offset 200
                (i32.store (i32.const 200) (local.get $res))
                ;; iovec: ptr=200, len=4
                (i32.store (i32.const 0) (i32.const 200))
                (i32.store (i32.const 4) (i32.const 4))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 300)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let merge_fn = WasmMergeFunction::new(wasm_bytes)?;

    let output_bytes = merge_fn.merge(b"a", b"b").await?;
    let errno = i32::from_le_bytes(output_bytes[0..4].try_into()?);

    assert_eq!(
        errno, 2,
        "Expected ERRNO_ACCES (2) when WASM module attempts clock_time_get under merge function capabilities"
    );

    Ok(())
}

/// Test (e): Output exceeding max_output_bytes returns OutputLimitExceeded error.
#[tokio::test]
async fn test_wasm_merge_function_output_limit_exceeded() -> TestResult {
    // WAT module writing 2 MB in chunks of 64KB (exceeding default 1 MB limit)
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 2)
            (func (export "_start")
                (local $i i32)
                ;; iovec: ptr=1000, len=65536
                (i32.store (i32.const 0) (i32.const 1000))
                (i32.store (i32.const 4) (i32.const 65536))
                (local.set $i (i32.const 0))
                (block $break
                    (loop $top
                        ;; 32 iterations * 64KB = 2 MB > 1 MB
                        (br_if $break (i32.ge_s (local.get $i) (i32.const 32)))
                        (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 500)))
                        (local.set $i (i32.add (local.get $i) (i32.const 1)))
                        (br $top)
                    )
                )
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let merge_fn = WasmMergeFunction::new(wasm_bytes)?;

    let result = merge_fn.merge(b"existing", b"new").await;

    assert!(
        matches!(
            result,
            Err(SandboxError::OutputLimitExceeded {
                stream: "stdout",
                ..
            })
        ),
        "Expected OutputLimitExceeded error for stdout, got: {:?}",
        result
    );

    Ok(())
}

/// Test (f): Invalid WASM module returns SandboxError::InvalidModule on `new()`.
#[test]
fn test_wasm_merge_function_invalid_module_returns_error() {
    let invalid_wasm = b"invalid WASM magic header".as_slice();
    let result = WasmMergeFunction::new(invalid_wasm);

    assert!(
        matches!(result, Err(SandboxError::InvalidModule(_))),
        "Expected InvalidModule error on new() with invalid binary, got: {:?}",
        result
    );
}
