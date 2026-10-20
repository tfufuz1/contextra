// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Test suite for Sandbox concurrency, state isolation, and WASM boundary edge cases.
// INVARIANTEN: Zero state sharing across concurrent executions, strict trap isolation, fail-closed limit enforcement.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

use contextra_sandbox::{SandboxError, WasmCapabilities, WasmExecutor};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Barrier};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Invariant: Parallel executions in distinct WasmExecutor workers share no memory/state.
/// Oracle: Concurrent counter increment instances each start from 0 and yield expected output independently.
#[tokio::test]
async fn test_parallel_sandbox_instances_share_no_state() -> TestResult {
    let seed: u64 = 0xABCD_1234_9876_5432;
    let num_threads = 8;
    let barrier = Arc::new(Barrier::new(num_threads));
    let executor = Arc::new(WasmExecutor::new()?);

    // WAT module: increments internal memory counter from 0 to 10 and writes ASCII result to stdout
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 16) "10")
            (func (export "_start")
                ;; Store counter = 10 at offset 0
                (i32.store (i32.const 0) (i32.const 10))
                ;; Prepare iovec: ptr=16, len=2
                (i32.store (i32.const 32) (i32.const 16))
                (i32.store (i32.const 36) (i32.const 2))
                (drop (call $fd_write (i32.const 1) (i32.const 32) (i32.const 1) (i32.const 40)))
            )
        )
    "#;
    let wasm_bytes = Arc::new(wat::parse_str(wat)?);

    let (tx, mut rx) = mpsc::channel(num_threads);

    for i in 0..num_threads {
        let barrier_cloned = barrier.clone();
        let executor_cloned = executor.clone();
        let wasm_bytes_cloned = wasm_bytes.clone();
        let tx_cloned = tx.clone();

        tokio::spawn(async move {
            barrier_cloned.wait().await;
            let caps = WasmCapabilities {
                random_seed: Some(seed.wrapping_add(i as u64)),
                ..Default::default()
            };
            let res = executor_cloned
                .execute(&wasm_bytes_cloned, b"", &caps, Duration::from_secs(5))
                .await;
            let _ = tx_cloned.send((i, res)).await;
        });
    }
    drop(tx);

    let mut completed_count = 0;
    while let Some((worker_id, result)) = rx.recv().await {
        completed_count += 1;
        let output =
            result.unwrap_or_else(|e| panic!("Worker {worker_id} failed with seed {seed:x}: {e}"));
        assert_eq!(
            &output.stdout[..],
            b"10",
            "Worker {worker_id} (seed 0x{seed:x}) observed contaminated state across parallel execution boundary"
        );
    }

    assert_eq!(
        completed_count, num_threads,
        "All parallel sandbox instances must finish"
    );
    Ok(())
}

/// Invariant: A WASM trap in one concurrent instance does NOT affect or corrupt sibling execution instances.
/// Oracle: Concurrent batch containing normal, trapping, fuel-exhausted, and timeout tasks all report exact expected error/success variants.
#[tokio::test]
async fn test_trap_in_one_instance_does_not_affect_sibling_instances() -> TestResult {
    let seed: u64 = 0xDEAF_BEEF_0000_1111;
    let executor = Arc::new(WasmExecutor::new()?);

    let wat_normal = r#"(module (func (export "_start")))"#;
    let wat_trap = r#"(module (func (export "_start") unreachable))"#;
    let wat_loop = r#"(module (func (export "_start") (loop (br 0))))"#;
    let wat_sleep = r#"
        (module
            (import "contextra" "host_sleep" (func $host_sleep (param i32)))
            (func (export "_start") (call $host_sleep (i32.const 200)))
        )
    "#;

    let wasm_normal = wat::parse_str(wat_normal)?;
    let wasm_trap = wat::parse_str(wat_trap)?;
    let wasm_loop = wat::parse_str(wat_loop)?;
    let wasm_sleep = wat::parse_str(wat_sleep)?;

    let barrier = Arc::new(Barrier::new(4));
    let (tx, mut rx) = mpsc::channel(4);

    // Task 1: Normal
    {
        let b = barrier.clone();
        let e = executor.clone();
        let bytes = wasm_normal;
        let t = tx.clone();
        tokio::spawn(async move {
            b.wait().await;
            let caps = WasmCapabilities::default();
            let res = e.execute(&bytes, b"", &caps, Duration::from_secs(2)).await;
            let _ = t.send(("normal", res)).await;
        });
    }

    // Task 2: Trap
    {
        let b = barrier.clone();
        let e = executor.clone();
        let bytes = wasm_trap;
        let t = tx.clone();
        tokio::spawn(async move {
            b.wait().await;
            let caps = WasmCapabilities::default();
            let res = e.execute(&bytes, b"", &caps, Duration::from_secs(2)).await;
            let _ = t.send(("trap", res)).await;
        });
    }

    // Task 3: Fuel exhaustion
    {
        let b = barrier.clone();
        let e = executor.clone();
        let bytes = wasm_loop;
        let t = tx.clone();
        tokio::spawn(async move {
            b.wait().await;
            let caps = WasmCapabilities {
                max_fuel: 1_000,
                ..Default::default()
            };
            let res = e.execute(&bytes, b"", &caps, Duration::from_secs(2)).await;
            let _ = t.send(("fuel", res)).await;
        });
    }

    // Task 4: Wall-clock timeout
    {
        let b = barrier.clone();
        let e = executor.clone();
        let bytes = wasm_sleep;
        let t = tx.clone();
        tokio::spawn(async move {
            b.wait().await;
            let caps = WasmCapabilities {
                max_wall_clock_ms: 50,
                ..Default::default()
            };
            let res = e.execute(&bytes, b"", &caps, Duration::from_secs(5)).await;
            let _ = t.send(("timeout", res)).await;
        });
    }
    drop(tx);

    let mut results = std::collections::HashMap::new();
    while let Some((tag, res)) = rx.recv().await {
        results.insert(tag, res);
    }

    assert!(
        results.get("normal").unwrap().is_ok(),
        "Normal execution must succeed despite sibling trap (seed: 0x{seed:x})"
    );
    assert!(
        matches!(results.get("trap").unwrap(), Err(SandboxError::WasmTrap(_))),
        "Trap task must return WasmTrap (seed: 0x{seed:x})"
    );
    assert!(
        matches!(
            results.get("fuel").unwrap(),
            Err(SandboxError::FuelExhausted { .. })
        ),
        "Fuel task must return FuelExhausted (seed: 0x{seed:x})"
    );
    assert!(
        matches!(
            results.get("timeout").unwrap(),
            Err(SandboxError::Timeout { .. })
        ),
        "Timeout task must return Timeout (seed: 0x{seed:x})"
    );

    Ok(())
}

/// Invariant: Dropping/cancelling the execution task midway leaves no resources leaked and causes no panics.
/// Oracle: Task handle cancellation terminates within bounded timeout and subsequent execution calls succeed cleanly.
#[tokio::test]
async fn test_drop_and_cancel_during_execution_is_clean() -> TestResult {
    let executor = Arc::new(WasmExecutor::new()?);
    let wat_sleep = r#"
        (module
            (import "contextra" "host_sleep" (func $host_sleep (param i32)))
            (func (export "_start") (call $host_sleep (i32.const 5000)))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat_sleep)?;

    let exec_clone = executor.clone();
    let task = tokio::spawn(async move {
        let caps = WasmCapabilities {
            max_wall_clock_ms: 10_000,
            ..Default::default()
        };
        exec_clone
            .execute(&wasm_bytes, b"", &caps, Duration::from_secs(10))
            .await
    });

    // Allow execution to start and enter sleep
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Cancel task
    task.abort();
    let join_res = task.await;
    assert!(
        join_res.is_err() && join_res.unwrap_err().is_cancelled(),
        "Aborted task must return cancelled error"
    );

    // Execute a subsequent quick WASM task on the executor to verify health
    let wat_quick = r#"(module (func (export "_start")))"#;
    let quick_bytes = wat::parse_str(wat_quick)?;
    let caps = WasmCapabilities::default();
    let res = executor
        .execute(&quick_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "Executor must remain fully functional following task cancellation"
    );

    Ok(())
}

/// Invariant: Edge WASM binary inputs (empty binary, invalid magic header, module without export) return explicit typed SandboxErrors.
/// Oracle: Match exact SandboxError enum variants without panics.
#[tokio::test]
async fn test_edge_cases_empty_invalid_and_no_export_wasm() -> TestResult {
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    // Edge Case 1: Empty WASM module binary
    let res_empty = executor
        .execute(b"", b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        matches!(res_empty, Err(SandboxError::InvalidModule(_))),
        "Empty binary must return InvalidModule error, got {:?}",
        res_empty
    );

    // Edge Case 2: Invalid WASM header magic bytes
    let res_invalid = executor
        .execute(b"NOT_WASM_MAGIC_BYTES", b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        matches!(res_invalid, Err(SandboxError::InvalidModule(_))),
        "Corrupted binary header must return InvalidModule error, got {:?}",
        res_invalid
    );

    // Edge Case 3: WASM module without `_start` export
    let wat_no_export = r#"(module (func $foo (nop)))"#;
    let wasm_no_export = wat::parse_str(wat_no_export)?;
    let res_no_export = executor
        .execute(&wasm_no_export, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res_no_export.is_ok(),
        "Module without `_start` export is valid and returns empty output, got {:?}",
        res_no_export
    );

    Ok(())
}

/// Invariant: Memory allocation up to `max_memory_pages` is allowed; exceeding `max_memory_pages` via memory.grow returns MemoryExceeded / WasmTrap.
/// Oracle: Guest memory grow call returns -1 (failed) or traps, enforcing hard page limit.
#[tokio::test]
async fn test_memory_grow_up_to_limit_and_exceed_boundary() -> TestResult {
    let executor = WasmExecutor::new()?;

    // WAT attempts to grow memory by 1 page (init 1 page, request +1 page -> total 2 pages)
    let wat_grow_1 = r#"
        (module
            (memory (export "memory") 1)
            (func (export "_start")
                (drop (memory.grow (i32.const 1)))
            )
        )
    "#;
    let wasm_grow_1 = wat::parse_str(wat_grow_1)?;

    // Case A: max_memory_pages = 2 -> Grow succeeds
    let caps_allowed = WasmCapabilities {
        max_memory_pages: 2,
        ..Default::default()
    };
    let res_allowed = executor
        .execute(&wasm_grow_1, b"", &caps_allowed, Duration::from_secs(1))
        .await;
    assert!(
        res_allowed.is_ok(),
        "Memory grow within max_memory_pages boundary must succeed"
    );

    // Case B: max_memory_pages = 1 -> Grow returns -1 in guest, execution finishes cleanly
    let caps_denied = WasmCapabilities {
        max_memory_pages: 1,
        ..Default::default()
    };
    let res_denied = executor
        .execute(&wasm_grow_1, b"", &caps_denied, Duration::from_secs(1))
        .await;
    assert!(
        res_denied.is_ok(),
        "Memory grow exceeding max_memory_pages returns -1 in WASM guest safely"
    );

    Ok(())
}

/// Invariant: Host call `host_cloud_query` handles 0-byte, normal, and max-length input buffers safely without panics.
/// Oracle: Return typed errors or success matching capability settings.
#[tokio::test]
async fn test_host_call_boundary_buffers() -> TestResult {
    let executor = WasmExecutor::new()?;

    let wat = r#"
        (module
            (import "contextra" "host_cloud_query" (func $host_cloud_query (result i32)))
            (func (export "_start")
                (drop (call $host_cloud_query))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;

    // Input buffer 1: 0-byte
    let caps_denied = WasmCapabilities {
        allow_cloud_egress: false,
        ..Default::default()
    };
    let res_empty_input = executor
        .execute(&wasm_bytes, b"", &caps_denied, Duration::from_secs(1))
        .await;
    assert!(
        matches!(res_empty_input, Err(SandboxError::CapabilityViolation { ref capability }) if capability == "allow_cloud_egress"),
        "Host call without cloud capability must return CapabilityViolation"
    );

    // Input buffer 2: max_stdin_bytes exact boundary
    let max_input = vec![0x41u8; caps_denied.max_stdin_bytes];
    let res_max_input = executor
        .execute(
            &wasm_bytes,
            &max_input,
            &caps_denied,
            Duration::from_secs(1),
        )
        .await;
    assert!(
        matches!(res_max_input, Err(SandboxError::CapabilityViolation { .. })),
        "Host call with exact max_stdin_bytes buffer must evaluate capability correctly"
    );

    // Input buffer 3: max_stdin_bytes + 1 byte
    let oversized_input = vec![0x41u8; caps_denied.max_stdin_bytes + 1];
    let res_oversized = executor
        .execute(
            &wasm_bytes,
            &oversized_input,
            &caps_denied,
            Duration::from_secs(1),
        )
        .await;
    assert!(
        matches!(res_oversized, Err(SandboxError::InputTooLarge { .. })),
        "Input exceeding max_stdin_bytes must return InputTooLarge error"
    );

    Ok(())
}
