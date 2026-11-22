// FILE-CONTEXT
// STAND: 2026-10-30T00:00:00Z
// ZWECK: WasmExecutor API and facade (§4.18)
// INVARIANTEN: #![forbid(unsafe_code)], fresh store/instance per call, strict fuel & timeout bounds

pub(crate) mod runner;
pub(crate) mod state;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::warn;
use wasmtime::{Config, Engine, Module, Store};

pub use state::CapabilityViolationError;
pub(crate) use state::SandboxState;

use crate::{capabilities::WasmCapabilities, error::SandboxError, output::WasmOutput};

/// WASM-Ausführungsgrenze für die `CodeExecution`-Permission.
///
/// # Designprinzip (§4.18)
/// - Frische `Store` + `Instance` pro execute()-Aufruf: kein Zustandsüberlauf
/// - Fuel-Budget (CPU, deterministisch) + Wall-Clock-Timeout (nicht-deterministisch) — beide pflicht
/// - `#![forbid(unsafe_code)]` — ausschließlich wasmtime's sichere Rust-API
pub struct WasmExecutor {
    engine: Engine,
}

impl std::fmt::Debug for WasmExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmExecutor").finish()
    }
}

impl WasmExecutor {
    /// Erstellt einen neuen WasmExecutor.
    ///
    /// # Errors
    /// Wenn wasmtime::Engine nicht initialisiert werden kann.
    pub fn new() -> Result<Self, SandboxError> {
        let mut config = Config::new();
        config.async_support(true);
        config.consume_fuel(true);
        config.max_wasm_stack(512 * 1024);

        let engine = Engine::new(&config)
            .map_err(|e| SandboxError::Runtime(format!("Engine init failed: {}", e)))?;
        Ok(Self { engine })
    }

    /// Validates a WASM binary without executing it.
    ///
    /// Checks binary size limits and attempts module compilation.
    pub fn validate_module(
        &self,
        wasm_bytes: &[u8],
        max_module_size_bytes: usize,
    ) -> Result<(), SandboxError> {
        if wasm_bytes.len() > max_module_size_bytes {
            return Err(SandboxError::InvalidModule(format!(
                "WASM module binary size ({} bytes) exceeds maximum allowed size ({} bytes)",
                wasm_bytes.len(),
                max_module_size_bytes
            )));
        }

        Module::validate(&self.engine, wasm_bytes)
            .map_err(|e| SandboxError::InvalidModule(e.to_string()))
    }

    /// Führt ein WASM-Binary mit Capability-Einschränkungen aus.
    ///
    /// # Garantien
    /// - Fuel-Budget: CPU-Limit (deterministisch), verhindert Endlosschleifen
    /// - Wall-Clock-Timeout: Verhindert IO-waiting über Zeithorizont
    /// - Frische Store+Instance: kein Zustandsüberlauf zwischen Aufrufen
    #[allow(clippy::disallowed_methods, clippy::cast_possible_truncation)]
    pub async fn execute(
        &self,
        wasm_bytes: &[u8],
        input: &[u8],
        capabilities: &WasmCapabilities,
        timeout: Duration,
    ) -> Result<WasmOutput, SandboxError> {
        let effective_timeout = if capabilities.max_wall_clock_ms > 0 {
            std::cmp::min(
                timeout,
                Duration::from_millis(capabilities.max_wall_clock_ms),
            )
        } else {
            timeout
        };
        let timeout_ms = u64::try_from(effective_timeout.as_millis()).unwrap_or(u64::MAX);
        let deadline = tokio::time::Instant::now() + effective_timeout;

        if input.len() > capabilities.max_stdin_bytes {
            return Err(SandboxError::InputTooLarge {
                len: input.len(),
                limit: capabilities.max_stdin_bytes,
            });
        }

        let module =
            runner::compile_module(&self.engine, wasm_bytes, capabilities, deadline, timeout_ms)
                .await?;

        let stdout_buf = Arc::new(Mutex::new(Vec::<u8>::new()));
        let stderr_buf = Arc::new(Mutex::new(Vec::<u8>::new()));

        let mut store = Store::new(
            &self.engine,
            SandboxState {
                max_pages: capabilities.max_memory_pages,
                max_table_entries: capabilities.max_table_entries,
                allow_cloud_egress: capabilities.allow_cloud_egress,
                allow_stdout: capabilities.allow_stdout,
                allow_stderr: capabilities.allow_stderr,
                allow_clock: capabilities.allow_clock,
                max_output_bytes: capabilities.max_output_bytes,
                host_calls_count: 0,
                max_host_calls: capabilities.max_host_calls,
                stdin_input: input.to_vec(),
                stdin_pos: 0,
                stdout_buf: stdout_buf.clone(),
                stderr_buf: stderr_buf.clone(),
                start_instant: tokio::time::Instant::now(),
                rng_state: capabilities.random_seed,
            },
        );

        store
            .set_fuel(capabilities.max_fuel)
            .map_err(|e| SandboxError::Runtime(format!("Fuel setup failed: {}", e)))?;

        store.limiter(|state| state);

        log_capability_warnings(capabilities);

        let linker = runner::create_linker(&self.engine)?;

        let execute_future =
            runner::execute_instance(linker, store, &module, capabilities, stdout_buf, stderr_buf);

        tokio::time::timeout_at(deadline, execute_future)
            .await
            .map_err(|_| SandboxError::Timeout { timeout_ms })?
    }
}

fn log_capability_warnings(capabilities: &WasmCapabilities) {
    if capabilities.allow_filesystem {
        warn!(
            "WasmExecutor: allow_filesystem=true — erhöhtes Risiko (keine FS-Imports registriert)"
        );
    }
    if capabilities.allow_network {
        warn!("WasmExecutor: allow_network=true — erhöhtes Risiko (keine Network-Imports registriert)");
    }
    if capabilities.allow_cloud_egress {
        warn!("WasmExecutor: allow_cloud_egress=true — erhöhtes Risiko");
    }
}

#[cfg(test)]
mod tests;
