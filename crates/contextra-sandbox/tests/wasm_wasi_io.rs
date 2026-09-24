//! Integration tests for WASI I/O, limits, clock, random, proc_exit, and capabilities (§4.18).

use contextra_sandbox::{SandboxError, WasmCapabilities, WasmExecutor};
use std::time::Duration;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// (a) Guest reads input via `fd_read` and writes it via `fd_write` back (Echo), Output == Input.
#[tokio::test]
async fn test_wasi_echo_stdin_to_stdout() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                ;; iovec for fd_read at offset 0: buf_ptr=100, buf_len=64
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (i32.const 64))
                ;; fd_read(0, iovs_ptr=0, iovs_len=1, nread_ptr=16)
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 16)))

                ;; iovec for fd_write at offset 32: buf_ptr=100, buf_len=*nread
                (i32.store (i32.const 32) (i32.const 100))
                (i32.store (i32.const 36) (i32.load (i32.const 16)))
                ;; fd_write(1, iovs_ptr=32, iovs_len=1, nwritten_ptr=48)
                (drop (call $fd_write (i32.const 1) (i32.const 32) (i32.const 1) (i32.const 48)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();
    let input_data = b"hello wasi echo test";

    let output = executor
        .execute(&wasm_bytes, input_data, &caps, Duration::from_secs(1))
        .await?;

    assert_eq!(&output.stdout[..], input_data);
    Ok(())
}

/// (b) Empty input -> EOF, guest terminates smoothly.
#[tokio::test]
async fn test_wasi_empty_input_eof() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                ;; iovec for fd_read: buf_ptr=100, buf_len=64
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (i32.const 64))
                ;; fd_read(0, iovs_ptr=0, iovs_len=1, nread_ptr=16)
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 16)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let output = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await?;

    assert!(output.stdout.is_empty());
    Ok(())
}

/// (c) Input > max_stdin_bytes -> InputTooLarge error.
#[tokio::test]
async fn test_wasi_input_too_large() -> TestResult {
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
    let input = b"0123456789_too_large";

    let result = executor
        .execute(&wasm_bytes, input, &caps, Duration::from_secs(1))
        .await;

    assert!(
        matches!(
            result,
            Err(SandboxError::InputTooLarge { len: 20, limit: 10 })
        ),
        "Expected InputTooLarge error, got: {:?}",
        result
    );
    Ok(())
}

/// (d) Guest writes in loop over max_output_bytes -> OutputLimitExceeded error, no unbounded growth.
#[tokio::test]
async fn test_wasi_output_limit_exceeded() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 100) "0123456789")
            (func (export "_start")
                ;; iovec at offset 0: buf_ptr=100, buf_len=10
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (i32.const 10))
                (loop $write_loop
                    (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 16)))
                    (br $write_loop)
                )
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        max_output_bytes: 25,
        ..Default::default()
    };

    let result = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(2))
        .await;

    assert!(
        matches!(
            result,
            Err(SandboxError::OutputLimitExceeded {
                stream: "stdout",
                limit: 25
            })
        ),
        "Expected OutputLimitExceeded error for stdout, got: {:?}",
        result
    );
    Ok(())
}

/// (e) proc_exit(0) with subsequent fd_write: write does NOT happen.
#[tokio::test]
async fn test_wasi_proc_exit_0_stops_execution() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "proc_exit"
                (func $proc_exit (param i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 100) "should not be written")
            (func (export "_start")
                ;; Exit immediately with code 0
                (call $proc_exit (i32.const 0))
                ;; Subsequent write should never execute
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (i32.const 21))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 16)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let output = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await?;

    assert!(
        output.stdout.is_empty(),
        "Expected empty stdout after proc_exit(0), got: {:?}",
        output.stdout
    );
    Ok(())
}

/// (f) proc_exit(3) -> ProcessExit { code: 3 }.
#[tokio::test]
async fn test_wasi_proc_exit_nonzero_returns_error() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "proc_exit"
                (func $proc_exit (param i32)))
            (func (export "_start")
                (call $proc_exit (i32.const 3))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let result = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;

    assert!(
        matches!(result, Err(SandboxError::ProcessExit { code: 3 })),
        "Expected ProcessExit {{ code: 3 }}, got: {:?}",
        result
    );
    Ok(())
}

/// (g) clock_time_get with allow_clock=true gives monotonic values; with false returns ACCES (2).
#[tokio::test]
async fn test_wasi_clock_time_get() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "clock_time_get"
                (func $clock_time_get (param i32 i64 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                ;; Call clock_time_get(clock_id=1, precision=0, time_ptr=100)
                (local $res i32)
                (local.set $res (call $clock_time_get (i32.const 1) (i64.const 0) (i32.const 100)))
                ;; Store errno at offset 200 (4 bytes)
                (i32.store (i32.const 200) (local.get $res))
                ;; iovec at offset 0: buf_ptr=200, buf_len=4
                (i32.store (i32.const 0) (i32.const 200))
                (i32.store (i32.const 4) (i32.const 4))
                ;; Write errno to stdout
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 300)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;

    // Case 1: allow_clock = true -> errno 0
    let caps_allowed = WasmCapabilities {
        allow_clock: true,
        ..Default::default()
    };
    let output_allowed = executor
        .execute(&wasm_bytes, b"", &caps_allowed, Duration::from_secs(1))
        .await?;
    let errno_allowed = i32::from_le_bytes(output_allowed.stdout[0..4].try_into()?);
    assert_eq!(errno_allowed, 0, "Expected clock_time_get to succeed (0)");

    // Case 2: allow_clock = false -> errno 2 (ERRNO_ACCES)
    let caps_denied = WasmCapabilities {
        allow_clock: false,
        ..Default::default()
    };
    let output_denied = executor
        .execute(&wasm_bytes, b"", &caps_denied, Duration::from_secs(1))
        .await?;
    let errno_denied = i32::from_le_bytes(output_denied.stdout[0..4].try_into()?);
    assert_eq!(
        errno_denied, 2,
        "Expected clock_time_get to return ERRNO_ACCES (2) when disabled"
    );

    Ok(())
}

/// (h) random_get with same seed twice -> identical bytes; without seed -> NOSYS (52).
#[tokio::test]
async fn test_wasi_random_get() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "random_get"
                (func $random_get (param i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (local $res i32)
                ;; random_get(buf_ptr=100, buf_len=16)
                (local.set $res (call $random_get (i32.const 100) (i32.const 16)))
                ;; Store errno at offset 200
                (i32.store (i32.const 200) (local.get $res))
                ;; iovec 0: ptr=200, len=4 (errno)
                (i32.store (i32.const 0) (i32.const 200))
                (i32.store (i32.const 4) (i32.const 4))
                ;; iovec 1: ptr=100, len=16 (random bytes)
                (i32.store (i32.const 8) (i32.const 100))
                (i32.store (i32.const 12) (i32.const 16))
                ;; fd_write(1, iovs_ptr=0, iovs_len=2, nwritten_ptr=300)
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 2) (i32.const 300)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;

    // Case 1: Seed set -> deterministic identical bytes across calls
    let caps_seeded = WasmCapabilities {
        random_seed: Some(42),
        ..Default::default()
    };
    let out1 = executor
        .execute(&wasm_bytes, b"", &caps_seeded, Duration::from_secs(1))
        .await?;
    let out2 = executor
        .execute(&wasm_bytes, b"", &caps_seeded, Duration::from_secs(1))
        .await?;

    let errno1 = i32::from_le_bytes(out1.stdout[0..4].try_into()?);
    let errno2 = i32::from_le_bytes(out2.stdout[0..4].try_into()?);
    assert_eq!(errno1, 0);
    assert_eq!(errno2, 0);
    assert_eq!(
        &out1.stdout[4..20],
        &out2.stdout[4..20],
        "Seeded random_get outputs must match"
    );

    // Case 2: No seed -> NOSYS (52)
    let caps_unseeded = WasmCapabilities {
        random_seed: None,
        ..Default::default()
    };
    let out_unseeded = executor
        .execute(&wasm_bytes, b"", &caps_unseeded, Duration::from_secs(1))
        .await?;
    let errno_nosys = i32::from_le_bytes(out_unseeded.stdout[0..4].try_into()?);
    assert_eq!(
        errno_nosys, 52,
        "Expected random_get without seed to return ERRNO_NOSYS (52)"
    );

    Ok(())
}

/// (i) args_sizes_get returns argc=0, argv_buf_size=0 and errno 0.
#[tokio::test]
async fn test_wasi_args_sizes_get() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "args_sizes_get"
                (func $args_sizes_get (param i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (local $res i32)
                ;; args_sizes_get(argc_ptr=100, argv_buf_size_ptr=104)
                (local.set $res (call $args_sizes_get (i32.const 100) (i32.const 104)))
                ;; iovec 0: ptr=100, len=8 (argc + buf_size)
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (i32.const 8))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 200)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let output = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await?;

    let argc = u32::from_le_bytes(output.stdout[0..4].try_into()?);
    let buf_size = u32::from_le_bytes(output.stdout[4..8].try_into()?);

    assert_eq!(argc, 0, "Expected argc = 0");
    assert_eq!(buf_size, 0, "Expected argv_buf_size = 0");
    Ok(())
}

/// (j) allow_stdout=false discards output silently without error.
#[tokio::test]
async fn test_wasi_allow_stdout_false_discards_output() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 100) "secret stdout message")
            (func (export "_start")
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (i32.const 21))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 200)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        allow_stdout: false,
        ..Default::default()
    };

    let output = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await?;

    assert!(
        output.stdout.is_empty(),
        "Expected stdout to be empty when allow_stdout=false"
    );
    Ok(())
}
