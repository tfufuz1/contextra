use super::*;
use std::time::Duration;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_invalid_wasm_module_returns_error() -> TestResult {
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();
    let result = executor
        .execute(b"not a wasm binary", b"", &caps, Duration::from_secs(1))
        .await;
    assert!(matches!(result, Err(SandboxError::InvalidModule(_))));
    Ok(())
}

#[tokio::test]
async fn test_cloud_egress_enforcement() -> TestResult {
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

    // Test case 1: allow_cloud_egress = false (default) -> CapabilityViolation
    let caps_denied = WasmCapabilities {
        allow_cloud_egress: false,
        ..Default::default()
    };
    let res_denied = executor
        .execute(&wasm_bytes, b"", &caps_denied, Duration::from_secs(1))
        .await;
    assert!(
        matches!(
            res_denied,
            Err(SandboxError::CapabilityViolation { ref capability }) if capability == "allow_cloud_egress"
        ),
        "Expected CapabilityViolation for allow_cloud_egress, got: {:?}",
        res_denied
    );

    // Test case 2: allow_cloud_egress = true -> Execution succeeds
    let caps_allowed = WasmCapabilities {
        allow_cloud_egress: true,
        ..Default::default()
    };
    let res_allowed = executor
        .execute(&wasm_bytes, b"", &caps_allowed, Duration::from_secs(1))
        .await;
    assert!(
        res_allowed.is_ok(),
        "Expected execution to succeed when allow_cloud_egress is true, got: {:?}",
        res_allowed
    );
    Ok(())
}

#[tokio::test]
async fn test_wasm_fuel_exhaustion_returns_error() -> TestResult {
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
        max_fuel: 1_000,
        ..Default::default()
    };

    let result = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(5))
        .await;

    assert!(
        matches!(result, Err(SandboxError::FuelExhausted { consumed }) if consumed > 0),
        "Expected FuelExhausted error with consumed > 0, got: {:?}",
        result
    );
    Ok(())
}

#[tokio::test]
async fn test_wasm_memory_isolation() -> TestResult {
    let wat = r#"
        (module
            (memory 32)
            (func (export "_start"))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;

    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        max_memory_pages: 2,
        ..Default::default()
    };

    let result = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(5))
        .await;

    assert!(
        matches!(
            result,
            Err(SandboxError::MemoryExceeded { .. }) | Err(SandboxError::Runtime(_))
        ),
        "Expected MemoryExceeded or Runtime error, got: {:?}",
        result
    );
    Ok(())
}

#[tokio::test]
async fn test_wasm_stdout_capture_via_fd_write() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 16) "hello")
            (func (export "_start")
                (i32.store (i32.const 0) (i32.const 16))
                (i32.store (i32.const 4) (i32.const 5))
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
    let output = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await?;
    assert_eq!(&output.stdout[..], b"hello");

    let caps_no_stdout = WasmCapabilities {
        allow_stdout: false,
        ..Default::default()
    };
    let output_no_stdout = executor
        .execute(&wasm_bytes, b"", &caps_no_stdout, Duration::from_secs(1))
        .await?;
    assert!(output_no_stdout.stdout.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_wasm_stderr_capture_via_fd_write() -> TestResult {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 16) "error msg")
            (func (export "_start")
                (i32.store (i32.const 0) (i32.const 16))
                (i32.store (i32.const 4) (i32.const 9))
                (drop (call $fd_write (i32.const 2) (i32.const 0) (i32.const 1) (i32.const 32)))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;

    let caps = WasmCapabilities {
        allow_stderr: true,
        ..Default::default()
    };
    let output = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await?;
    assert_eq!(&output.stderr[..], b"error msg");
    Ok(())
}

#[tokio::test]
async fn test_wasm_wall_clock_timeout_enforced() -> TestResult {
    let wat = r#"
        (module
            (import "contextra" "host_sleep" (func $host_sleep (param i32)))
            (func (export "_start")
                (call $host_sleep (i32.const 200))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;

    let caps = WasmCapabilities {
        max_wall_clock_ms: 50,
        ..Default::default()
    };

    let result_timeout = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(5))
        .await;

    assert!(
        matches!(
            result_timeout,
            Err(SandboxError::Timeout { timeout_ms: 50 })
        ),
        "Expected Timeout error with 50ms, got: {:?}",
        result_timeout
    );
    Ok(())
}

#[tokio::test]
async fn test_wasm_module_size_limit_exceeded() -> TestResult {
    let wat = r#"
        (module
            (func (export "_start"))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        max_module_size_bytes: 5, // Exceeded by any valid WASM binary
        ..Default::default()
    };

    let result = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;

    assert!(
        matches!(result, Err(SandboxError::InvalidModule(ref msg)) if msg.contains("exceeds maximum allowed size")),
        "Expected InvalidModule error due to binary size limit, got: {:?}",
        result
    );
    Ok(())
}

#[test]
fn test_memory_growing_div_ceil_boundary() -> TestResult {
    let mut state = SandboxState {
        max_pages: 2,
        max_table_entries: 10,
        allow_cloud_egress: false,
        allow_stdout: true,
        allow_stderr: false,
        allow_clock: true,
        max_output_bytes: 1024 * 1024,
        host_calls_count: 0,
        max_host_calls: 10_000,
        stdin_input: Vec::new(),
        stdin_pos: 0,
        stdout_buf: Arc::new(Mutex::new(Vec::new())),
        stderr_buf: Arc::new(Mutex::new(Vec::new())),
        start_instant: tokio::time::Instant::now(),
        rng_state: None,
    };
    use wasmtime::ResourceLimiter;
    // 65536 bytes = 1 page
    assert!(state.memory_growing(0, 65536, None)?);
    // 65537 bytes = 2 pages (div_ceil)
    assert!(state.memory_growing(0, 65537, None)?);
    // 131073 bytes = 3 pages (div_ceil) -> exceeds max_pages (2)
    assert!(!state.memory_growing(0, 131073, None)?);
    Ok(())
}

#[test]
fn test_table_growing_limit() -> TestResult {
    let mut state = SandboxState {
        max_pages: 2,
        max_table_entries: 10,
        allow_cloud_egress: false,
        allow_stdout: true,
        allow_stderr: false,
        allow_clock: true,
        max_output_bytes: 1024 * 1024,
        host_calls_count: 0,
        max_host_calls: 10_000,
        stdin_input: Vec::new(),
        stdin_pos: 0,
        stdout_buf: Arc::new(Mutex::new(Vec::new())),
        stderr_buf: Arc::new(Mutex::new(Vec::new())),
        start_instant: tokio::time::Instant::now(),
        rng_state: None,
    };
    use wasmtime::ResourceLimiter;
    assert!(state.table_growing(0, 10, None)?);
    assert!(!state.table_growing(0, 11, None)?);
    Ok(())
}

#[tokio::test]
async fn test_wasm_trap_unreachable_classified_as_wasm_trap() -> TestResult {
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

    let result = executor
        .execute(&wasm_bytes, b"", &caps, Duration::from_secs(1))
        .await;

    assert!(
        matches!(result, Err(SandboxError::WasmTrap(_))),
        "Expected WasmTrap error for unreachable instruction, got: {:?}",
        result
    );
    Ok(())
}
