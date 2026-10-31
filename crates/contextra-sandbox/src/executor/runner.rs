// FILE-CONTEXT
// STAND: 2026-10-30T00:00:00Z
// ZWECK: WASM module execution and linker setup
// INVARIANTEN: #![forbid(unsafe_code)], zero panics, strict error classification

use std::sync::{Arc, Mutex};
use std::time::Duration;
use wasmtime::{Engine, Linker, Module, Store};

use crate::{
    capabilities::WasmCapabilities,
    error::SandboxError,
    executor::state::{CapabilityViolationError, HostCallLimitExceededError, SandboxState},
    output::WasmOutput,
    wasi::{self, OutputLimitExceededError, ProcessExitError},
};

pub(crate) async fn compile_module(
    engine: &Engine,
    wasm_bytes: &[u8],
    capabilities: &WasmCapabilities,
    deadline: tokio::time::Instant,
    timeout_ms: u64,
) -> Result<Module, SandboxError> {
    if wasm_bytes.len() > capabilities.max_module_size_bytes {
        return Err(SandboxError::InvalidModule(format!(
            "WASM module binary size ({} bytes) exceeds maximum allowed size ({} bytes)",
            wasm_bytes.len(),
            capabilities.max_module_size_bytes
        )));
    }

    let engine_clone = engine.clone();
    let wasm_bytes_vec = wasm_bytes.to_vec();
    let compile_task =
        tokio::task::spawn_blocking(move || Module::from_binary(&engine_clone, &wasm_bytes_vec));

    match tokio::time::timeout_at(deadline, compile_task).await {
        Ok(Ok(Ok(m))) => Ok(m),
        Ok(Ok(Err(e))) => Err(SandboxError::InvalidModule(e.to_string())),
        Ok(Err(join_err)) => Err(SandboxError::Runtime(format!(
            "Compilation task failed: {}",
            join_err
        ))),
        Err(_) => Err(SandboxError::Timeout { timeout_ms }),
    }
}

pub(crate) fn create_linker(engine: &Engine) -> Result<Linker<SandboxState>, SandboxError> {
    let mut linker = Linker::new(engine);

    wasi::register(&mut linker)
        .map_err(|e| SandboxError::Runtime(format!("Linker WASI setup failed: {}", e)))?;

    linker
        .func_wrap(
            "contextra",
            "host_cloud_query",
            move |mut caller: wasmtime::Caller<'_, SandboxState>| -> Result<i32, wasmtime::Error> {
                let state = caller.data_mut();
                if let Err(e) = state.check_and_inc_host_call() {
                    return Err(wasmtime::Error::from(e));
                }
                if !state.allow_cloud_egress {
                    Err(wasmtime::Error::from(CapabilityViolationError {
                        capability: "allow_cloud_egress",
                    }))
                } else {
                    Ok(0)
                }
            },
        )
        .map_err(|e| SandboxError::Runtime(format!("Linker host_cloud_query failed: {}", e)))?;

    linker
        .func_wrap_async(
            "contextra",
            "host_sleep",
            move |mut caller: wasmtime::Caller<'_, SandboxState>, (millis,): (u32,)| {
                Box::new(async move {
                    let state = caller.data_mut();
                    if let Err(e) = state.check_and_inc_host_call() {
                        return Err(wasmtime::Error::from(e));
                    }
                    tokio::time::sleep(Duration::from_millis(millis as u64)).await;
                    Ok(())
                })
            },
        )
        .map_err(|e| SandboxError::Runtime(format!("Linker host_sleep failed: {}", e)))?;

    Ok(linker)
}

pub(crate) async fn execute_instance(
    linker: Linker<SandboxState>,
    mut store: Store<SandboxState>,
    module: &Module,
    capabilities: &WasmCapabilities,
    stdout_buf: Arc<Mutex<Vec<u8>>>,
    stderr_buf: Arc<Mutex<Vec<u8>>>,
) -> Result<WasmOutput, SandboxError> {
    let instance = linker
        .instantiate_async(&mut store, module)
        .await
        .map_err(|e| SandboxError::Runtime(format!("Instantiation failed: {}", e)))?;

    if let Ok(start_fn) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
        if let Err(e) = start_fn.call_async(&mut store, ()).await {
            classify_execution_error(e, capabilities.max_fuel)?;
        }
    }

    let fuel_consumed = capabilities
        .max_fuel
        .saturating_sub(store.get_fuel().unwrap_or(0));

    let stdout = stdout_buf
        .lock()
        .map_err(|e| SandboxError::Runtime(e.to_string()))?
        .clone();
    let stderr = stderr_buf
        .lock()
        .map_err(|e| SandboxError::Runtime(e.to_string()))?
        .clone();

    Ok(WasmOutput::new(stdout, stderr, fuel_consumed))
}

fn classify_execution_error(e: anyhow::Error, max_fuel: u64) -> Result<(), SandboxError> {
    if let Some(exit_err) = e.downcast_ref::<ProcessExitError>() {
        if exit_err.code != 0 {
            return Err(SandboxError::ProcessExit {
                code: exit_err.code,
            });
        }
        Ok(())
    } else if let Some(out_err) = e.downcast_ref::<OutputLimitExceededError>() {
        Err(SandboxError::OutputLimitExceeded {
            stream: out_err.stream,
            limit: out_err.limit,
        })
    } else if let Some(cap_err) = e.downcast_ref::<CapabilityViolationError>() {
        Err(SandboxError::CapabilityViolation {
            capability: cap_err.capability.to_string(),
        })
    } else if let Some(host_err) = e.downcast_ref::<HostCallLimitExceededError>() {
        Err(SandboxError::CapabilityViolation {
            capability: host_err.to_string(),
        })
    } else if let Some(wasmtime::Trap::OutOfFuel) = e.downcast_ref::<wasmtime::Trap>() {
        Err(SandboxError::FuelExhausted { consumed: max_fuel })
    } else {
        Err(SandboxError::WasmTrap(format!("{:#}", e)))
    }
}
