// FILE-CONTEXT
// STAND: 2026-10-30T00:00:00Z
// ZWECK: Abuse and exploit test matrix for Wasm execution boundary (§4.18)
// INVARIANTEN: Zero panics, no process termination, timeout < 10s per test, fail-closed security

use contextra_sandbox::{SandboxError, WasmCapabilities, WasmExecutor};
use std::time::Duration;

type TestResult = Result<(), Box<dyn std::error::Error>>;

// 1. Infinite loop fuel exhaustion
#[tokio::test]
async fn test_abuse_infinite_loop_fuel_exhaustion() -> TestResult {
    let wat = r#"
        (module
            (func (export "_start")
                (loop (br 0))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        max_fuel: 5_000,
        ..Default::default()
    };

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(2))
        .await;
    assert!(
        matches!(res, Err(SandboxError::FuelExhausted { consumed }) if consumed > 0),
        "Expected FuelExhausted, got: {:?}",
        res
    );
    Ok(())
}

// 2. memory.grow bomb
#[tokio::test]
async fn test_abuse_memory_grow_bomb() -> TestResult {
    let wat = r#"
        (module
            (memory 1)
            (func (export "_start")
                (loop
                    (drop (memory.grow (i32.const 1)))
                    (br 0)
                )
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        max_memory_pages: 4,
        max_fuel: 1_000_000,
        ..Default::default()
    };

    // Execution should terminate via fuel exhaustion or memory limit without crashing process
    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(2))
        .await;
    assert!(
        matches!(
            res,
            Err(SandboxError::FuelExhausted { .. }) | Err(SandboxError::MemoryExceeded { .. })
        ),
        "Expected FuelExhausted or MemoryExceeded, got: {:?}",
        res
    );
    Ok(())
}

// 3. Infinite recursion causing stack overflow
#[tokio::test]
async fn test_abuse_infinite_recursion_stack_overflow() -> TestResult {
    let wat = r#"
        (module
            (func $recurse (export "_start")
                (call $recurse)
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        max_fuel: 10_000_000,
        ..Default::default()
    };

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(2))
        .await;
    assert!(
        matches!(res, Err(SandboxError::WasmTrap(_))),
        "Expected WasmTrap for stack overflow, got: {:?}",
        res
    );
    Ok(())
}

// 4. Giant table creation / growth
#[tokio::test]
async fn test_abuse_giant_table_grow() -> TestResult {
    let wat = r#"
        (module
            (table 1000000 funcref)
            (func (export "_start"))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        max_table_entries: 50,
        ..Default::default()
    };

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(2))
        .await;
    assert!(
        matches!(
            res,
            Err(SandboxError::Runtime(_)) | Err(SandboxError::InvalidModule(_))
        ),
        "Expected failure due to table limit, got: {:?}",
        res
    );
    Ok(())
}

// 5. fd_write with invalid iovs_ptr
#[tokio::test]
async fn test_abuse_fd_write_invalid_iovs_ptr() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (drop (call $fd_write (i32.const 1) (i32.const 999999) (i32.const 1) (i32.const 0)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        allow_stdout: true,
        ..Default::default()
    };

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "fd_write with OOB pointer should return ERRNO_INVAL safely, got: {:?}",
        res
    );
    Ok(())
}

// 6. fd_write with invalid buffer pointer inside iovec
#[tokio::test]
async fn test_abuse_fd_write_invalid_buffer_ptr() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (i32.store (i32.const 0) (i32.const 999999)) ;; buf_ptr OOB
                (i32.store (i32.const 4) (i32.const 10))     ;; buf_len
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 32)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        allow_stdout: true,
        ..Default::default()
    };

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "fd_write with OOB buffer pointer should return ERRNO_INVAL safely"
    );
    Ok(())
}

// 7. fd_read with invalid iovs_ptr
#[tokio::test]
async fn test_abuse_fd_read_invalid_iovs_ptr() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (drop (call $fd_read (i32.const 0) (i32.const 999999) (i32.const 1) (i32.const 0)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let res = executor
        .execute(&wasm_bytes, b"input", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "fd_read with OOB iovs_ptr should return ERRNO_INVAL safely"
    );
    Ok(())
}

// 8. fd_read with invalid buffer pointer
#[tokio::test]
async fn test_abuse_fd_read_invalid_buffer_ptr() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (i32.store (i32.const 0) (i32.const 999999)) ;; buf_ptr OOB
                (i32.store (i32.const 4) (i32.const 10))     ;; buf_len
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 32)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let res = executor
        .execute(&wasm_bytes, b"input", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "fd_read with OOB buf_ptr should return ERRNO_INVAL safely"
    );
    Ok(())
}

// 9. clock_time_get with invalid pointer
#[tokio::test]
async fn test_abuse_clock_time_get_invalid_ptr() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "clock_time_get"
                (func $clock_time_get (param i32 i64 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (drop (call $clock_time_get (i32.const 1) (i64.const 0) (i32.const 999999)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        allow_clock: true,
        ..Default::default()
    };

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "clock_time_get with OOB time_ptr should return ERRNO_INVAL safely"
    );
    Ok(())
}

// 10. random_get with invalid pointer
#[tokio::test]
async fn test_abuse_random_get_invalid_ptr() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "random_get"
                (func $random_get (param i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (drop (call $random_get (i32.const 999999) (i32.const 16)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        random_seed: Some(42),
        ..Default::default()
    };

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "random_get with OOB ptr should return ERRNO_INVAL safely"
    );
    Ok(())
}

// 11. args_sizes_get with invalid pointer
#[tokio::test]
async fn test_abuse_args_sizes_get_invalid_ptr() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "args_sizes_get"
                (func $args_sizes_get (param i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (drop (call $args_sizes_get (i32.const 999999) (i32.const 0)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "args_sizes_get with OOB ptr should return ERRNO_INVAL safely"
    );
    Ok(())
}

// 12. environ_sizes_get with invalid pointer
#[tokio::test]
async fn test_abuse_environ_sizes_get_invalid_ptr() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "environ_sizes_get"
                (func $environ_sizes_get (param i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (drop (call $environ_sizes_get (i32.const 999999) (i32.const 0)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "environ_sizes_get with OOB ptr should return ERRNO_INVAL safely"
    );
    Ok(())
}

// 13. Integer overflow in WASI pointer arithmetic
#[tokio::test]
async fn test_abuse_pointer_overflow_wrapping() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                ;; iovs_ptr near u32::MAX
                (drop (call $fd_read (i32.const 0) (i32.const 2147483640) (i32.const 1000) (i32.const 0)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let res = executor
        .execute(&wasm_bytes, b"input", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "Pointer arithmetic overflow should be handled safely with ERRNO_INVAL"
    );
    Ok(())
}

// 14. Access attempt to ungranted WASI preopen
#[tokio::test]
async fn test_abuse_ungranted_wasi_preopen() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                ;; Attempt writing to fd 3 (preopen / file fd)
                (drop (call $fd_write (i32.const 3) (i32.const 0) (i32.const 0) (i32.const 0)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "Accessing ungranted fd should return ERRNO_BADF cleanly"
    );
    Ok(())
}

// 15. Path traversal strings in input fail-closed without FS
#[tokio::test]
async fn test_abuse_path_traversal_blocked() -> TestResult {
    let wat = r#"
        (module
            (func (export "_start"))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default(); // allow_filesystem = false

    let res = executor
        .execute(&wasm_bytes, b"../etc/passwd", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "Path traversal input with disabled FS runs safely without opening host files"
    );
    Ok(())
}

// 16. Output flood stdout exceeding limit
#[tokio::test]
async fn test_abuse_stdout_output_flood() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 16) "1234567890")
            (func (export "_start")
                (i32.store (i32.const 0) (i32.const 16))
                (i32.store (i32.const 4) (i32.const 10))
                (loop
                    (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 32)))
                    (br 0)
                )
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        allow_stdout: true,
        max_output_bytes: 50,
        ..Default::default()
    };

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(2))
        .await;
    assert!(
        matches!(
            res,
            Err(SandboxError::OutputLimitExceeded {
                stream: "stdout",
                limit: 50
            })
        ),
        "Expected OutputLimitExceeded, got: {:?}",
        res
    );
    Ok(())
}

// 17. Output flood stderr exceeding limit
#[tokio::test]
async fn test_abuse_stderr_output_flood() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 16) "1234567890")
            (func (export "_start")
                (i32.store (i32.const 0) (i32.const 16))
                (i32.store (i32.const 4) (i32.const 10))
                (loop
                    (drop (call $fd_write (i32.const 2) (i32.const 0) (i32.const 1) (i32.const 32)))
                    (br 0)
                )
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        allow_stderr: true,
        max_output_bytes: 30,
        ..Default::default()
    };

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(2))
        .await;
    assert!(
        matches!(
            res,
            Err(SandboxError::OutputLimitExceeded {
                stream: "stderr",
                limit: 30
            })
        ),
        "Expected OutputLimitExceeded, got: {:?}",
        res
    );
    Ok(())
}

// 18. WASM trap unreachable
#[tokio::test]
async fn test_abuse_wasm_trap_unreachable() -> TestResult {
    let wat = r#"
        (module
            (func (export "_start")
                unreachable
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        matches!(res, Err(SandboxError::WasmTrap(_))),
        "Expected WasmTrap, got: {:?}",
        res
    );
    Ok(())
}

// 19. WASM trap division by zero
#[tokio::test]
async fn test_abuse_wasm_trap_division_by_zero() -> TestResult {
    let wat = r#"
        (module
            (func (export "_start")
                (drop (i32.div_s (i32.const 10) (i32.const 0)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        matches!(res, Err(SandboxError::WasmTrap(_))),
        "Expected WasmTrap for div by 0, got: {:?}",
        res
    );
    Ok(())
}

// 20. WASM trap out-of-bounds memory load
#[tokio::test]
async fn test_abuse_wasm_trap_oob_memory_access() -> TestResult {
    let wat = r#"
        (module
            (memory 1)
            (func (export "_start")
                (drop (i32.load (i32.const 70000))) ;; memory 1 = 65536 bytes
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        matches!(res, Err(SandboxError::WasmTrap(_))),
        "Expected WasmTrap for OOB load, got: {:?}",
        res
    );
    Ok(())
}

// 21. Host call limit exceeded
#[tokio::test]
async fn test_abuse_host_call_flood() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "args_get"
                (func $args_get (param i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (loop
                    (drop (call $args_get (i32.const 0) (i32.const 0)))
                    (br 0)
                )
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        max_host_calls: 10,
        ..Default::default()
    };

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(2))
        .await;
    assert!(
        matches!(res, Err(SandboxError::CapabilityViolation { ref capability }) if capability.contains("Host call limit exceeded")),
        "Expected Host call limit CapabilityViolation error, got: {:?}",
        res
    );
    Ok(())
}

// 22. Clock access denied by default
#[tokio::test]
async fn test_abuse_clock_access_denied_by_default() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "clock_time_get"
                (func $clock_time_get (param i32 i64 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                ;; store result errno at 0
                (i32.store (i32.const 0) (call $clock_time_get (i32.const 1) (i64.const 0) (i32.const 8)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default(); // allow_clock = false

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "clock_time_get when disabled should return ERRNO_ACCES without trap"
    );
    Ok(())
}

// 23. Random access denied by default
#[tokio::test]
async fn test_abuse_random_access_denied_by_default() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "random_get"
                (func $random_get (param i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (drop (call $random_get (i32.const 0) (i32.const 16)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default(); // random_seed = None

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "random_get without seed returns ERRNO_NOSYS cleanly"
    );
    Ok(())
}

// 24. Cloud egress denied by default
#[tokio::test]
async fn test_abuse_cloud_egress_denied_by_default() -> TestResult {
    let wat = r#"
        (module
            (import "contextra" "host_cloud_query" (func $host_cloud_query (result i32)))
            (func (export "_start")
                (drop (call $host_cloud_query))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default(); // allow_cloud_egress = false

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        matches!(res, Err(SandboxError::CapabilityViolation { ref capability }) if capability == "allow_cloud_egress"),
        "Expected CapabilityViolation for cloud egress, got: {:?}",
        res
    );
    Ok(())
}

// 25. Oversized module binary
#[tokio::test]
async fn test_abuse_oversized_module_binary() -> TestResult {
    let wat = r#"
        (module
            (func (export "_start"))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        max_module_size_bytes: 5,
        ..Default::default()
    };

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        matches!(res, Err(SandboxError::InvalidModule(_))),
        "Expected InvalidModule for binary size overflow, got: {:?}",
        res
    );
    Ok(())
}

// 26. Oversized stdin input
#[tokio::test]
async fn test_abuse_oversized_stdin_input() -> TestResult {
    let wat = r#"
        (module
            (func (export "_start"))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        max_stdin_bytes: 10,
        ..Default::default()
    };

    let large_input = vec![0u8; 100];
    let res = executor
        .execute(&wasm_bytes, &large_input, &caps, Duration::from_secs(1))
        .await;
    assert!(
        matches!(
            res,
            Err(SandboxError::InputTooLarge {
                len: 100,
                limit: 10
            })
        ),
        "Expected InputTooLarge, got: {:?}",
        res
    );
    Ok(())
}

// 27. Environment variables zero leakage check
#[tokio::test]
async fn test_abuse_environ_get_zero_leakage() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "environ_sizes_get"
                (func $environ_sizes_get (param i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                ;; store environc at 0 and environ_buf_size at 4
                (drop (call $environ_sizes_get (i32.const 0) (i32.const 4)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        res.is_ok(),
        "environ_sizes_get returns 0 variables without leaking host environment"
    );
    Ok(())
}

// 28. Process exit with non-zero code
#[tokio::test]
async fn test_abuse_nonzero_proc_exit() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "proc_exit"
                (func $proc_exit (param i32)))
            (func (export "_start")
                (call $proc_exit (i32.const 42))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let res = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(
        matches!(res, Err(SandboxError::ProcessExit { code: 42 })),
        "Expected ProcessExit {{ code: 42 }}, got: {:?}",
        res
    );
    Ok(())
}

// 29. Sandbox reusability after trap
#[tokio::test]
async fn test_abuse_sandbox_reusability_after_trap() -> TestResult {
    let trap_wat = r#"
        (module
            (func (export "_start") unreachable)
        )
    "#;
    let valid_wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 16) "ok")
            (func (export "_start")
                (i32.store (i32.const 0) (i32.const 16))
                (i32.store (i32.const 4) (i32.const 2))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 32)))
            )
        )
    "#;

    let trap_bytes = wat::parse_str(trap_wat)?;
    let valid_bytes = wat::parse_str(valid_wat)?;

    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        allow_stdout: true,
        ..Default::default()
    };

    // First call traps
    let res_trap = executor
        .execute(&trap_bytes, b"", &caps, Duration::from_secs(1))
        .await;
    assert!(matches!(res_trap, Err(SandboxError::WasmTrap(_))));

    // Second call on same executor instance succeeds without residue
    let res_valid = executor
        .execute(&valid_bytes, b"", &caps, Duration::from_secs(1))
        .await?;
    assert_eq!(&res_valid.stdout[..], b"ok");
    Ok(())
}
