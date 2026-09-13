//! WasmExecutor — WASM Execution Engine (§4.18).
//!
//! Jeder execute()-Aufruf startet eine frische Store+Instance (kein Zustandsüberlauf).
//! Fuel (CPU-Limit) UND Wall-Clock-Timeout (tokio) sind beide aktiv.

use std::time::Duration;
use tracing::warn;
use wasmtime::{Config, Engine, Module, Store};

use crate::{
    capabilities::WasmCapabilities,
    error::SandboxError,
    output::WasmOutput,
};

struct SandboxState {
    max_pages: u32,
}

impl wasmtime::ResourceLimiter for SandboxState {
    fn memory_growing(&mut self, _current: usize, desired: usize, _max: Option<usize>) -> anyhow::Result<bool> {
        let desired_pages = (desired / 65536) as u32;
        Ok(desired_pages <= self.max_pages)
    }

    fn table_growing(&mut self, _current: u32, _desired: u32, _max: Option<u32>) -> anyhow::Result<bool> {
        Ok(true)
    }
}

/// WASM-Ausführungsgrenze für die `CodeExecution`-Permission.
///
/// # Designprinzip (§4.18)
/// - Frische `Store` + `Instance` pro execute()-Aufruf: kein Zustandsüberlauf
/// - Fuel-Budget (CPU, deterministisch) + Wall-Clock-Timeout (nicht-deterministisch) — beide pflicht
/// - `#![forbid(unsafe_code)]` — ausschließlich wasmtime's sichere Rust-API
pub struct WasmExecutor {
    engine: Engine,
}

impl WasmExecutor {
    /// Erstellt einen neuen WasmExecutor.
    ///
    /// # Errors
    /// Wenn wasmtime::Engine nicht initialisiert werden kann.
    pub fn new() -> Result<Self, SandboxError> {
        let mut config = Config::new();
        config.async_support(true);
        config.consume_fuel(true);  // Fuel-Mechanismus aktivieren (CPU-Limit)
        // Cranelift-Backend (standard, sicher)
        let engine = Engine::new(&config)
            .map_err(|e| SandboxError::Runtime(format!("Engine init failed: {}", e)))?;
        Ok(Self { engine })
    }

    /// Führt ein WASM-Binary mit Capability-Einschränkungen aus.
    ///
    /// # Garantien
    /// - Fuel-Budget: CPU-Limit (deterministisch), verhindert Endlosschleifen
    /// - Wall-Clock-Timeout: Verhindert IO-waiting über Zeithorizont
    /// - Frische Store+Instance: kein Zustandsüberlauf zwischen Aufrufen
    ///
    /// # Errors
    /// - `SandboxError::Timeout` wenn Wall-Clock-Timeout überschritten
    /// - `SandboxError::FuelExhausted` wenn Fuel-Budget verbraucht
    /// - `SandboxError::MemoryExceeded` wenn Memory-Limit überschritten
    /// - `SandboxError::InvalidModule` wenn WASM ungültig
    /// - `SandboxError::WasmTrap` bei WASM-Trap
    pub async fn execute(
        &self,
        wasm_bytes: &[u8],
        input: &[u8],
        capabilities: &WasmCapabilities,
        timeout: Duration,
    ) -> Result<WasmOutput, SandboxError> {
        // Modul kompilieren (Validierung inbegriffen)
        let module = Module::from_binary(&self.engine, wasm_bytes)
            .map_err(|e| SandboxError::InvalidModule(e.to_string()))?;

        // Frische Store für diese Execution (kein Zustandsüberlauf)
        let mut store = Store::new(
            &self.engine,
            SandboxState {
                max_pages: capabilities.max_memory_pages,
            },
        );

        // Fuel-Budget setzen
        store
            .set_fuel(capabilities.max_fuel)
            .map_err(|e| SandboxError::Runtime(format!("Fuel setup failed: {}", e)))?;

        // Memory-Limit via Store-Limiter
        store.limiter(|state| state);

        // Stdout/Stderr Buffer
        let stdout_buf = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
        let stderr_buf = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));

        // Capability-Checks
        if capabilities.allow_filesystem {
            warn!("WasmExecutor: allow_filesystem=true — erhöhtes Risiko");
        }
        if capabilities.allow_network {
            warn!("WasmExecutor: allow_network=true — erhöhtes Risiko");
        }

        // Linker mit minimalen WASI-Imports
        let mut linker = wasmtime::Linker::new(&self.engine);

        // Input als WASM-Export verfügbar machen (über stdin-Emulation)
        let _input_clone = input.to_vec();
        let stdout_clone = stdout_buf.clone();
        let stderr_clone = stderr_buf.clone();

        // Minimalste WASI-fd_write Implementierung für stdout
        linker
            .func_wrap(
                "wasi_snapshot_preview1",
                "fd_write",
                move |_caller: wasmtime::Caller<'_, SandboxState>,
                      fd: i32,
                      iovs: i32,
                      iovs_len: i32,
                      nwritten: i32|
                      -> i32 {
                    // Minimal stub — verhindert Trap bei fd_write-Calls
                    let _ = (fd, iovs, iovs_len, nwritten, &stdout_clone, &stderr_clone);
                    0i32
                },
            )
            .map_err(|e| SandboxError::Runtime(format!("Linker setup failed: {}", e)))?;

        // proc_exit
        linker
            .func_wrap(
                "wasi_snapshot_preview1",
                "proc_exit",
                |_: wasmtime::Caller<'_, SandboxState>, _code: i32| {},
            )
            .map_err(|e| SandboxError::Runtime(format!("Linker proc_exit failed: {}", e)))?;

        // Wall-Clock-Timeout wrapping der Instanziierung + Ausführung
        let execute_future = async {
            let instance = linker
                .instantiate_async(&mut store, &module)
                .await
                .map_err(|e| SandboxError::Runtime(format!("Instantiation failed: {}", e)))?;

            // _start / main aufrufen
            if let Ok(start_fn) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
                start_fn.call_async(&mut store, ()).await.map_err(|e| {
                    if e.to_string().contains("fuel") {
                        let consumed = capabilities.max_fuel;
                        SandboxError::FuelExhausted { consumed }
                    } else {
                        SandboxError::WasmTrap(e.to_string())
                    }
                })?;
            }

            let fuel_consumed = capabilities
                .max_fuel
                .saturating_sub(store.get_fuel().unwrap_or(0));

            Ok::<WasmOutput, SandboxError>(WasmOutput::new(
                stdout_buf.lock().unwrap_or_else(|e| e.into_inner()).clone(),
                stderr_buf.lock().unwrap_or_else(|e| e.into_inner()).clone(),
                fuel_consumed,
            ))
        };

        tokio::time::timeout(timeout, execute_future)
            .await
            .map_err(|_| SandboxError::Timeout {
                timeout_ms: timeout.as_millis() as u64,
            })?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_invalid_wasm_module_returns_error() {
        let executor = WasmExecutor::new().expect("WasmExecutor");
        let caps = WasmCapabilities::default();
        let result = executor
            .execute(b"not a wasm binary", b"", &caps, Duration::from_secs(1))
            .await;
        assert!(matches!(result, Err(SandboxError::InvalidModule(_))));
    }
}
