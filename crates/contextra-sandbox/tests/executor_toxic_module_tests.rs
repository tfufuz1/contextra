// STAND: 2026-10-13
// ZWECK: Toxic module isolation tests (infinite loop fuel, memory limit, guest trap, fuel determinism)

use contextra_sandbox::{
    admission::AdmittedModule,
    capabilities::WasmCapabilities,
    executor::WasmExecutor,
    SandboxError,
};
use std::time::Duration;

#[tokio::test]
async fn test_toxic_module_infinite_loop_fuel_exhausted_host_remains_intact() -> Result<(), Box<dyn std::error::Error>> {
    let wat = r#"
        (module
            (func (export "_start")
                (loop (br 0))
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let admitted = AdmittedModule::new(wasm_bytes, [0u8; 32]);
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        max_fuel: 5_000,
        ..Default::default()
    };

    let result = executor
        .execute_admitted(&admitted, b"", &caps, Duration::from_secs(2))
        .await;

    assert!(
        matches!(result, Err(SandboxError::FuelExhausted { consumed }) if consumed > 0),
        "Expected FuelExhausted, got: {:?}",
        result
    );

    // Host executor remains functional for subsequent executions!
    let echo_wat = r#"(module (func (export "_start")))"#;
    let echo_bytes = wat::parse_str(echo_wat)?;
    let echo_admitted = AdmittedModule::new(echo_bytes, [0u8; 32]);
    let ok_res = executor
        .execute_admitted(&echo_admitted, b"", &caps, Duration::from_secs(2))
        .await;
    assert!(ok_res.is_ok(), "Host must remain intact after toxic module failure");

    Ok(())
}

#[tokio::test]
async fn test_toxic_module_memory_limit_exceeded_host_remains_intact() -> Result<(), Box<dyn std::error::Error>> {
    let wat = r#"
        (module
            (memory 100)
            (func (export "_start"))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let admitted = AdmittedModule::new(wasm_bytes, [0u8; 32]);
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities {
        max_memory_pages: 4, // Allow max 4 pages
        ..Default::default()
    };

    let result = executor
        .execute_admitted(&admitted, b"", &caps, Duration::from_secs(2))
        .await;

    assert!(
        matches!(
            result,
            Err(SandboxError::MemoryExceeded { .. }) | Err(SandboxError::Runtime(_))
        ),
        "Expected MemoryExceeded/Runtime error, got: {:?}",
        result
    );

    Ok(())
}

#[tokio::test]
async fn test_toxic_module_trap_unreachable_host_remains_intact() -> Result<(), Box<dyn std::error::Error>> {
    let wat = r#"
        (module
            (func (export "_start")
                unreachable
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let admitted = AdmittedModule::new(wasm_bytes, [0u8; 32]);
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let result = executor
        .execute_admitted(&admitted, b"", &caps, Duration::from_secs(2))
        .await;

    assert!(
        matches!(result, Err(SandboxError::WasmTrap(_))),
        "Expected WasmTrap, got: {:?}",
        result
    );

    Ok(())
}

#[tokio::test]
async fn test_deterministic_fuel_consumption() -> Result<(), Box<dyn std::error::Error>> {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 16) "deterministic")
            (func (export "_start")
                (local $i i32)
                (local.set $i (i32.const 0))
                (block
                    (loop
                        (local.set $i (i32.add (local.get $i) (i32.const 1)))
                        (br_if 1 (i32.ge_s (local.get $i) (i32.const 100)))
                        (br 0)
                    )
                )
                (i32.store (i32.const 0) (i32.const 16))
                (i32.store (i32.const 4) (i32.const 13))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 32)))
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat)?;
    let admitted = AdmittedModule::new(wasm_bytes, [0u8; 32]);
    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let out1 = executor
        .execute_admitted(&admitted, b"input1", &caps, Duration::from_secs(2))
        .await?;

    let out2 = executor
        .execute_admitted(&admitted, b"input1", &caps, Duration::from_secs(2))
        .await?;

    assert_eq!(&out1.stdout[..], b"deterministic");
    assert_eq!(&out2.stdout[..], b"deterministic");

    assert_eq!(
        out1.fuel_consumed, out2.fuel_consumed,
        "Fuel consumption must be 100% deterministic across identical runs: run1={}, run2={}",
        out1.fuel_consumed, out2.fuel_consumed
    );
    assert!(out1.fuel_consumed > 0);

    Ok(())
}
