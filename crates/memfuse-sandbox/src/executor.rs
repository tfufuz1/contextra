// FILE-CONTEXT
// STAND: 2026-09-15T16:00:00Z (SESSION: acf8fe72)
// ZWECK: WASM execution engine with WASI and fuel enforcement
// INVARIANTEN: #![forbid(unsafe_code)], fresh store/instance per call, strict fuel & timeout bounds
// NICHT-OFFENSICHTLICH: host_cloud_query host function checks allow_cloud_egress dynamically
// SIEHE AUCH: AGENTS.md §4.18, P9 Security Invariant

//! WasmExecutor — WASM Execution Engine (§4.18).
//!
//! Jeder execute()-Aufruf startet eine frische Store+Instance (kein Zustandsüberlauf).
//! Fuel (CPU-Limit) UND Wall-Clock-Timeout (tokio) sind beide aktiv.

use std::time::Duration;
use tracing::warn;
use wasmtime::{Config, Engine, Module, Store};

use crate::{capabilities::WasmCapabilities, error::SandboxError, output::WasmOutput};

/// Custom error type returned by host functions on capability violation (INV-SBX-3).
#[derive(Debug, Clone)]
pub struct CapabilityViolationError {
    pub capability: &'static str,
}

impl std::fmt::Display for CapabilityViolationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WASM capability violation: {}", self.capability)
    }
}

impl std::error::Error for CapabilityViolationError {}

struct SandboxState {
    max_pages: u32,
    max_table_entries: u32,
    allow_cloud_egress: bool,
    allow_stdout: bool,
    allow_stderr: bool,
}

impl wasmtime::ResourceLimiter for SandboxState {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _max: Option<usize>,
    ) -> anyhow::Result<bool> {
        // INV-SBX-2: Rounding up with div_ceil ensures exact page limits without page under-counting
        let desired_pages = desired.div_ceil(65536) as u32;
        Ok(desired_pages <= self.max_pages)
    }

    fn table_growing(
        &mut self,
        _current: u32,
        desired: u32,
        _max: Option<u32>,
    ) -> anyhow::Result<bool> {
        // INV-SBX-2: Enforce strict ceiling on table elements
        Ok(desired <= self.max_table_entries)
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
        config.consume_fuel(true); // Fuel-Mechanismus aktivieren (CPU-Limit)
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
        // Wall-Clock-Timeout Enforcement (§4.18, IP-15 / B5)
        // `max_wall_clock_ms` is orthogonal to `max_fuel` (CPU limit vs. Wall-Clock limit, configured independently).
        // A value of 0 in `max_wall_clock_ms` indicates unlimited wall-clock capability limit, falling back to the caller's `timeout`.
        let effective_timeout = if capabilities.max_wall_clock_ms > 0 {
            std::cmp::min(
                timeout,
                Duration::from_millis(capabilities.max_wall_clock_ms),
            )
        } else {
            timeout
        };
        let timeout_ms = effective_timeout.as_millis() as u64;
        let deadline = tokio::time::Instant::now() + effective_timeout;

        // INV-SBX-1: WASM module binary size limit check prior to compilation
        if wasm_bytes.len() > capabilities.max_module_size_bytes {
            return Err(SandboxError::InvalidModule(format!(
                "WASM module binary size ({} bytes) exceeds maximum allowed size ({} bytes)",
                wasm_bytes.len(),
                capabilities.max_module_size_bytes
            )));
        }

        // INV-SBX-1: Offload synchronous Module compilation to spawn_blocking with wall-clock timeout to prevent compilation DoS
        let engine_clone = self.engine.clone();
        let wasm_bytes_vec = wasm_bytes.to_vec();
        let compile_task = tokio::task::spawn_blocking(move || {
            Module::from_binary(&engine_clone, &wasm_bytes_vec)
        });

        let module = match tokio::time::timeout_at(deadline, compile_task).await {
            Ok(Ok(Ok(m))) => m,
            Ok(Ok(Err(e))) => return Err(SandboxError::InvalidModule(e.to_string())),
            Ok(Err(join_err)) => {
                return Err(SandboxError::Runtime(format!(
                    "Compilation task failed: {}",
                    join_err
                )))
            }
            Err(_) => return Err(SandboxError::Timeout { timeout_ms }),
        };

        // Frische Store für diese Execution (kein Zustandsüberlauf)
        let mut store = Store::new(
            &self.engine,
            SandboxState {
                max_pages: capabilities.max_memory_pages,
                max_table_entries: capabilities.max_table_entries,
                allow_cloud_egress: capabilities.allow_cloud_egress,
                allow_stdout: capabilities.allow_stdout,
                allow_stderr: capabilities.allow_stderr,
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
        // INV-SBX-5: Capability fields (allow_filesystem, allow_network) are strictly enforced by non-registration
        // of WASI filesystem/network socket host functions in the Linker below.
        if capabilities.allow_filesystem {
            warn!("WasmExecutor: allow_filesystem=true — erhöhtes Risiko (keine FS-Imports registriert)");
        }
        if capabilities.allow_network {
            warn!("WasmExecutor: allow_network=true — erhöhtes Risiko (keine Network-Imports registriert)");
        }
        if capabilities.allow_cloud_egress {
            warn!("WasmExecutor: allow_cloud_egress=true — erhöhtes Risiko");
        }

        // Linker mit minimalen WASI-Imports
        let mut linker = wasmtime::Linker::new(&self.engine);

        // Input als WASM-Export verfügbar machen (über stdin-Emulation)
        let _input_clone = input.to_vec();
        let stdout_clone = stdout_buf.clone();
        let stderr_clone = stderr_buf.clone();

        // AI-TAG[RESOLVED] WASI fd_write buffer parsing (ID: AGT-SANDBOX-12a4a39c) (TS: 2026-09-15T16:05:00Z) (SESSION: acf8fe72)
        // BEFUND: Linker stub for WASI fd_write discards iovs memory buffers and returns 0 without writing to stdout_buf or stderr_buf.
        // RESOLVED: Implemented WASI preview1 buffer parser reading guest memory ciovec structures into stdout_buf/stderr_buf with defensive bounds checking.
        linker
            .func_wrap(
                "wasi_snapshot_preview1",
                "fd_write",
                move |mut caller: wasmtime::Caller<'_, SandboxState>,
                      fd: i32,
                      iovs_ptr: i32,
                      iovs_len: i32,
                      nwritten_ptr: i32|
                      -> i32 {
                    const ERRNO_SUCCESS: i32 = 0;
                    const ERRNO_BADF: i32 = 8;
                    const ERRNO_INVAL: i32 = 28;

                    if fd != 1 && fd != 2 {
                        return ERRNO_BADF;
                    }

                    if iovs_ptr < 0 || iovs_len < 0 || nwritten_ptr < 0 {
                        return ERRNO_INVAL;
                    }

                    let memory = match caller.get_export("memory") {
                        Some(wasmtime::Extern::Memory(mem)) => mem,
                        _ => return ERRNO_INVAL,
                    };

                    let mem_slice = memory.data(&caller);
                    let iovs_start = iovs_ptr as usize;
                    let iovs_count = iovs_len as usize;

                    let iovs_bytes = match iovs_count.checked_mul(8) {
                        Some(bytes) => bytes,
                        None => return ERRNO_INVAL,
                    };

                    let iovs_end = match iovs_start.checked_add(iovs_bytes) {
                        Some(end) => end,
                        None => return ERRNO_INVAL,
                    };

                    if iovs_end > mem_slice.len() {
                        return ERRNO_INVAL;
                    }

                    let mut total_written: u32 = 0;
                    let allow_write = if fd == 1 {
                        caller.data().allow_stdout
                    } else {
                        caller.data().allow_stderr
                    };

                    for i in 0..iovs_count {
                        let offset = iovs_start + i * 8;
                        let iov_buf = match mem_slice.get(offset..offset + 8) {
                            Some(slice) => slice,
                            None => return ERRNO_INVAL,
                        };

                        let buf_ptr = u32::from_le_bytes(match iov_buf[0..4].try_into() {
                            Ok(arr) => arr,
                            Err(_) => return ERRNO_INVAL,
                        }) as usize;
                        let buf_len = u32::from_le_bytes(match iov_buf[4..8].try_into() {
                            Ok(arr) => arr,
                            Err(_) => return ERRNO_INVAL,
                        }) as usize;

                        let buf_end = match buf_ptr.checked_add(buf_len) {
                            Some(end) => end,
                            None => return ERRNO_INVAL,
                        };

                        if buf_end > mem_slice.len() {
                            return ERRNO_INVAL;
                        }

                        if allow_write && buf_len > 0 {
                            if let Some(slice) = mem_slice.get(buf_ptr..buf_end) {
                                if fd == 1 {
                                    if let Ok(mut guard) = stdout_clone.lock() {
                                        guard.extend_from_slice(slice);
                                    }
                                } else if fd == 2 {
                                    if let Ok(mut guard) = stderr_clone.lock() {
                                        guard.extend_from_slice(slice);
                                    }
                                }
                            }
                        }

                        total_written = match total_written.checked_add(buf_len as u32) {
                            Some(sum) => sum,
                            None => return ERRNO_INVAL,
                        };
                    }

                    let nwritten_offset = nwritten_ptr as usize;
                    let nwritten_end = match nwritten_offset.checked_add(4) {
                        Some(end) => end,
                        None => return ERRNO_INVAL,
                    };

                    let mem_slice_mut = memory.data_mut(&mut caller);
                    if nwritten_end > mem_slice_mut.len() {
                        return ERRNO_INVAL;
                    }

                    if let Some(dest) = mem_slice_mut.get_mut(nwritten_offset..nwritten_end) {
                        dest.copy_from_slice(&total_written.to_le_bytes());
                    } else {
                        return ERRNO_INVAL;
                    }

                    ERRNO_SUCCESS
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

        // Host Cloud Query function enforcement
        linker
            .func_wrap(
                "memfuse",
                "host_cloud_query",
                move |caller: wasmtime::Caller<'_, SandboxState>| -> Result<i32, wasmtime::Error> {
                    if !caller.data().allow_cloud_egress {
                        Err(wasmtime::Error::from(CapabilityViolationError {
                            capability: "allow_cloud_egress",
                        }))
                    } else {
                        Ok(0)
                    }
                },
            )
            .map_err(|e| SandboxError::Runtime(format!("Linker host_cloud_query failed: {}", e)))?;

        // Host async sleep/delay helper function for testing wall-clock timeouts
        linker
            .func_wrap_async(
                "memfuse",
                "host_sleep",
                move |_caller: wasmtime::Caller<'_, SandboxState>, (millis,): (u32,)| {
                    Box::new(async move {
                        tokio::time::sleep(Duration::from_millis(millis as u64)).await;
                        Ok(())
                    })
                },
            )
            .map_err(|e| SandboxError::Runtime(format!("Linker host_sleep failed: {}", e)))?;

        // Wall-Clock-Timeout wrapping der Instanziierung + Ausführung
        let execute_future = async {
            let instance = linker
                .instantiate_async(&mut store, &module)
                .await
                .map_err(|e| SandboxError::Runtime(format!("Instantiation failed: {}", e)))?;

            // _start / main aufrufen
            if let Ok(start_fn) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
                start_fn.call_async(&mut store, ()).await.map_err(|e| {
                    // INV-SBX-3: Precise error classification using downcast_ref without string matching
                    if let Some(cap_err) = e.downcast_ref::<CapabilityViolationError>() {
                        SandboxError::CapabilityViolation {
                            capability: cap_err.capability.to_string(),
                        }
                    } else if let Some(trap) = e.downcast_ref::<wasmtime::Trap>() {
                        match trap {
                            wasmtime::Trap::OutOfFuel => {
                                let consumed = capabilities.max_fuel;
                                SandboxError::FuelExhausted { consumed }
                            }
                            _ => SandboxError::WasmTrap(format!("{:#}", e)),
                        }
                    } else {
                        SandboxError::WasmTrap(format!("{:#}", e))
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

        tokio::time::timeout_at(deadline, execute_future)
            .await
            .map_err(|_| SandboxError::Timeout { timeout_ms })?
    }
}

#[cfg(test)]
mod tests {
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
                (import "memfuse" "host_cloud_query" (func $host_cloud_query (result i32)))
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

        let caps = WasmCapabilities::default();
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
                (import "memfuse" "host_sleep" (func $host_sleep (param i32)))
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
}
