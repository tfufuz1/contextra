// FILE-CONTEXT
// STAND: 2026-09-15T16:00:00Z (SESSION: acf8fe72)
// ZWECK: WASM execution engine with WASI and fuel enforcement
// INVARIANTEN: #![forbid(unsafe_code)], fresh store/instance per call, strict fuel & timeout bounds
// NICHT-OFFENSICHTLICH: host_cloud_query host function checks allow_cloud_egress dynamically
// SIEHE AUCH: AGENTS.md §4.18, P9 Security Invariant

//! WasmExecutor — WASM Execution Engine (§4.18).
//!
//! Jeder execute_admitted()-Aufruf startet eine frische Store+Instance (kein Zustandsüberlauf).
//! Fuel (CPU-Limit) UND Wall-Clock-Timeout (tokio) sind beide aktiv.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::warn;
use wasmtime::{Config, Engine, Linker, Module, Store};

use crate::{
    admission::AdmittedModule,
    capabilities::WasmCapabilities,
    error::SandboxError,
    output::WasmOutput,
    wasi::{self, OutputLimitExceededError, ProcessExitError},
};

/// Bounded worker pool for WASM executions to prevent thread/concurrency exhaustion.
#[derive(Clone)]
pub struct WasmWorkerPool {
    semaphore: Arc<tokio::sync::Semaphore>,
    max_workers: usize,
}

impl std::fmt::Debug for WasmWorkerPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmWorkerPool")
            .field("max_workers", &self.max_workers)
            .field("available_permits", &self.available_workers())
            .finish()
    }
}

impl WasmWorkerPool {
    /// Erstellt einen neuen WasmWorkerPool mit einer festen Obergrenze an parallelen Worker-Slots.
    pub fn new(max_workers: usize) -> Self {
        let max_workers = max_workers.max(1);
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_workers)),
            max_workers,
        }
    }

    /// Gibt die maximale Anzahl paralleler Worker zurück.
    pub fn max_workers(&self) -> usize {
        self.max_workers
    }

    /// Gibt die Anzahl aktuell verbleibender freier Worker-Permits zurück.
    pub fn available_workers(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Führt ein Async-Job aus. Bei Rückstau (keine Permits frei) wird sofort `Err(SandboxError::Runtime("Busy: ..."))` geliefert.
    pub async fn run_job<F, Fut, T>(&self, job: F) -> Result<T, SandboxError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, SandboxError>>,
    {
        let permit = self.semaphore.try_acquire().map_err(|_| {
            SandboxError::Runtime("Busy: worker pool capacity reached".to_string())
        })?;
        let result = job().await;
        drop(permit);
        result
    }
}

/// In-memory LRU module cache for compiled `wasmtime::Module` instances.
struct ModuleCache {
    entries: HashMap<[u8; 32], (Module, u64)>,
    access_counter: u64,
    max_capacity: usize,
}

impl ModuleCache {
    fn new(max_capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            access_counter: 0,
            max_capacity: max_capacity.max(1),
        }
    }

    fn get(&mut self, hash: &[u8; 32]) -> Option<Module> {
        if let Some((module, access)) = self.entries.get_mut(hash) {
            self.access_counter = self.access_counter.saturating_add(1);
            *access = self.access_counter;
            Some(module.clone())
        } else {
            None
        }
    }

    fn insert(&mut self, hash: [u8; 32], module: Module) {
        if self.entries.contains_key(&hash) {
            self.access_counter = self.access_counter.saturating_add(1);
            self.entries.insert(hash, (module, self.access_counter));
            return;
        }

        if self.entries.len() >= self.max_capacity {
            // Evict least recently used entry
            if let Some((&lru_key, _)) = self
                .entries
                .iter()
                .min_by_key(|(_k, (_mod, access))| *access)
            {
                self.entries.remove(&lru_key);
            }
        }

        self.access_counter = self.access_counter.saturating_add(1);
        self.entries.insert(hash, (module, self.access_counter));
    }
}

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

pub(crate) struct SandboxState {
    pub(crate) max_pages: u32,
    pub(crate) max_table_entries: u32,
    pub(crate) allow_cloud_egress: bool,
    pub(crate) allow_stdout: bool,
    pub(crate) allow_stderr: bool,
    pub(crate) allow_clock: bool,
    pub(crate) max_output_bytes: usize,
    pub(crate) stdin_input: Vec<u8>,
    pub(crate) stdin_pos: usize,
    pub(crate) stdout_buf: Arc<Mutex<Vec<u8>>>,
    pub(crate) stderr_buf: Arc<Mutex<Vec<u8>>>,
    pub(crate) start_instant: std::time::Instant,
    pub(crate) rng_state: Option<u64>,
}

impl wasmtime::ResourceLimiter for SandboxState {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _max: Option<usize>,
    ) -> anyhow::Result<bool> {
        // INV-SBX-2: Rounding up with div_ceil ensures exact page limits without page under-counting
        let desired_pages = match u32::try_from(desired.div_ceil(65536)) {
            Ok(p) => p,
            Err(_) => return Ok(false), // page count exceeds u32::MAX — deny
        };
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
/// - Frische `Store` + `Instance` pro execute_admitted()-Aufruf: kein Zustandsüberlauf
/// - Fuel-Budget (CPU, deterministisch) + Wall-Clock-Timeout (nicht-deterministisch) — beide pflicht
/// - `#![forbid(unsafe_code)]` — ausschließlich wasmtime's sichere Rust-API
pub struct WasmExecutor {
    engine: Engine,
    linker: Linker<SandboxState>,
    module_cache: Mutex<ModuleCache>,
    worker_pool: Option<WasmWorkerPool>,
    compilation_count: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl std::fmt::Debug for WasmExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmExecutor")
            .field("compilation_count", &self.compilation_count.load(Ordering::Relaxed))
            .field("cache_hits", &self.cache_hits.load(Ordering::Relaxed))
            .field("cache_misses", &self.cache_misses.load(Ordering::Relaxed))
            .field("worker_pool", &self.worker_pool)
            .finish()
    }
}

impl WasmExecutor {
    /// Erstellt einen neuen WasmExecutor.
    ///
    /// # Errors
    /// Wenn wasmtime::Engine oder Linker nicht initialisiert werden kann.
    pub fn new() -> Result<Self, SandboxError> {
        Self::new_with_pool(None)
    }

    /// Erstellt einen neuen WasmExecutor mit optionalem Worker-Pool.
    pub fn new_with_pool(worker_pool: Option<WasmWorkerPool>) -> Result<Self, SandboxError> {
        let mut config = Config::new();
        config.async_support(true);
        config.consume_fuel(true); // Fuel-Mechanismus aktivieren (CPU-Limit)

        let engine = Engine::new(&config)
            .map_err(|e| SandboxError::Runtime(format!("Engine init failed: {}", e)))?;

        let mut linker = Linker::new(&engine);

        wasi::register(&mut linker)
            .map_err(|e| SandboxError::Runtime(format!("Linker WASI setup failed: {}", e)))?;

        linker
            .func_wrap(
                "contextra",
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

        linker
            .func_wrap_async(
                "contextra",
                "host_sleep",
                move |_caller: wasmtime::Caller<'_, SandboxState>, (millis,): (u32,)| {
                    Box::new(async move {
                        tokio::time::sleep(Duration::from_millis(millis as u64)).await;
                        Ok(())
                    })
                },
            )
            .map_err(|e| SandboxError::Runtime(format!("Linker host_sleep failed: {}", e)))?;

        Ok(Self {
            engine,
            linker,
            module_cache: Mutex::new(ModuleCache::new(64)),
            worker_pool,
            compilation_count: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        })
    }

    /// Setzt einen Worker-Pool für diesen Executor.
    pub fn with_worker_pool(mut self, pool: WasmWorkerPool) -> Self {
        self.worker_pool = Some(pool);
        self
    }

    /// Gibt die Gesamtzahl der Modul-Kompilierungen zurück.
    pub fn compilation_count(&self) -> u64 {
        self.compilation_count.load(Ordering::Relaxed)
    }

    /// Gibt die Anzahl der Modul-Cache-Treffer zurück.
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }

    /// Gibt die Anzahl der Modul-Cache-Misses zurück.
    pub fn cache_misses(&self) -> u64 {
        self.cache_misses.load(Ordering::Relaxed)
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

    /// Standard-Ausführungspfad für ein vorab verifiziertes [`AdmittedModule`].
    ///
    /// # Garantien
    /// - Fuel-Budget: CPU-Limit (deterministisch), verhindert Endlosschleifen
    /// - Wall-Clock-Timeout: Verhindert IO-waiting über Zeithorizont
    /// - Frische Store+Instance: kein Zustandsüberlauf zwischen Aufrufen
    ///
    /// ```compile_fail
    /// use contextra_sandbox::{WasmExecutor, WasmCapabilities};
    /// use std::time::Duration;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let executor = WasmExecutor::new()?;
    ///     let caps = WasmCapabilities::default();
    ///     let bytes: &[u8] = b"wasm bytes";
    ///     // Calling execute_admitted with raw byte slice fails compilation:
    ///     let _ = executor.execute_admitted(bytes, b"", &caps, Duration::from_secs(1)).await;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    /// - `SandboxError::Timeout` wenn Wall-Clock-Timeout überschritten
    /// - `SandboxError::FuelExhausted` wenn Fuel-Budget verbraucht
    /// - `SandboxError::MemoryExceeded` wenn Memory-Limit überschritten
    /// - `SandboxError::InvalidModule` wenn WASM ungültig
    /// - `SandboxError::WasmTrap` bei WASM-Trap
    pub async fn execute_admitted(
        &self,
        module: &AdmittedModule,
        input: &[u8],
        capabilities: &WasmCapabilities,
        timeout: Duration,
    ) -> Result<WasmOutput, SandboxError> {
        self.execute_internal(module.bytes(), input, capabilities, timeout)
            .await
    }

    /// Führt ein unüberprüftes WASM-Binary aus.
    ///
    /// # Warnung
    /// **Unsicher für persistierte Module!**
    /// Das Ausführen von Rohbytes ohne Verifikation ist anfällig für Manipulationen
    /// und schädliche Modul-Payloads. Nutze stattdessen [`Self::execute_admitted`].
    #[cfg(feature = "unverified-execute")]
    pub async fn execute(
        &self,
        wasm_bytes: &[u8],
        input: &[u8],
        capabilities: &WasmCapabilities,
        timeout: Duration,
    ) -> Result<WasmOutput, SandboxError> {
        self.execute_internal(wasm_bytes, input, capabilities, timeout)
            .await
    }

    async fn execute_internal(
        &self,
        wasm_bytes: &[u8],
        input: &[u8],
        capabilities: &WasmCapabilities,
        timeout: Duration,
    ) -> Result<WasmOutput, SandboxError> {
        // Enforce worker pool backpressure if configured
        let _permit = if let Some(ref pool) = self.worker_pool {
            Some(pool.semaphore.try_acquire().map_err(|_| {
                SandboxError::Runtime("Busy: worker pool capacity reached".to_string())
            })?)
        } else {
            None
        };

        // Wall-Clock-Timeout Enforcement (§4.18, IP-15 / B5)
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

        // Check input size limit prior to execution
        if input.len() > capabilities.max_stdin_bytes {
            return Err(SandboxError::InputTooLarge {
                len: input.len(),
                limit: capabilities.max_stdin_bytes,
            });
        }

        // INV-SBX-1: WASM module binary size limit check prior to compilation
        if wasm_bytes.len() > capabilities.max_module_size_bytes {
            return Err(SandboxError::InvalidModule(format!(
                "WASM module binary size ({} bytes) exceeds maximum allowed size ({} bytes)",
                wasm_bytes.len(),
                capabilities.max_module_size_bytes
            )));
        }

        // P05 / F-02 / HIGH: Module Provenance & Authorization Policy check prior to compilation and resource allocation
        capabilities.verify_module_policy(wasm_bytes)?;

        // Module Cache Lookup / Compilation
        let hash = WasmCapabilities::compute_sha256(wasm_bytes);
        let cached_module = {
            let mut cache = self
                .module_cache
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            cache.get(&hash)
        };

        let module = match cached_module {
            Some(m) => {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                m
            }
            None => {
                self.cache_misses.fetch_add(1, Ordering::Relaxed);
                let engine_clone = self.engine.clone();
                let wasm_bytes_vec = wasm_bytes.to_vec();
                let compile_task = tokio::task::spawn_blocking(move || {
                    Module::from_binary(&engine_clone, &wasm_bytes_vec)
                });

                let compiled = match tokio::time::timeout_at(deadline, compile_task).await {
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

                self.compilation_count.fetch_add(1, Ordering::Relaxed);
                let mut cache = self
                    .module_cache
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                cache.insert(hash, compiled.clone());
                compiled
            }
        };

        // Stdout/Stderr Buffer
        let stdout_buf = Arc::new(Mutex::new(Vec::<u8>::new()));
        let stderr_buf = Arc::new(Mutex::new(Vec::<u8>::new()));

        // Frische Store für diese Execution (kein Zustandsüberlauf)
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
                stdin_input: input.to_vec(),
                stdin_pos: 0,
                stdout_buf: stdout_buf.clone(),
                stderr_buf: stderr_buf.clone(),
                start_instant: std::time::Instant::now(),
                rng_state: capabilities.random_seed,
            },
        );

        // Fuel-Budget setzen
        store
            .set_fuel(capabilities.max_fuel)
            .map_err(|e| SandboxError::Runtime(format!("Fuel setup failed: {}", e)))?;

        // Memory-Limit via Store-Limiter
        store.limiter(|state| state);

        // Capability-Checks
        if capabilities.allow_filesystem {
            warn!("WasmExecutor: allow_filesystem=true — erhöhtes Risiko (keine FS-Imports registriert)");
        }
        if capabilities.allow_network {
            warn!("WasmExecutor: allow_network=true — erhöhtes Risiko (keine Network-Imports registriert)");
        }
        if capabilities.allow_cloud_egress {
            warn!("WasmExecutor: allow_cloud_egress=true — erhöhtes Risiko");
        }

        // Wall-Clock-Timeout wrapping der Instanziierung + Ausführung
        let execute_future = async {
            let instance = self
                .linker
                .instantiate_async(&mut store, &module)
                .await
                .map_err(|e| SandboxError::Runtime(format!("Instantiation failed: {}", e)))?;

            // _start / main aufrufen
            if let Ok(start_fn) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
                if let Err(e) = start_fn.call_async(&mut store, ()).await {
                    if let Some(exit_err) = e.downcast_ref::<ProcessExitError>() {
                        if exit_err.code != 0 {
                            return Err(SandboxError::ProcessExit {
                                code: exit_err.code,
                            });
                        }
                    } else if let Some(out_err) = e.downcast_ref::<OutputLimitExceededError>() {
                        return Err(SandboxError::OutputLimitExceeded {
                            stream: out_err.stream,
                            limit: out_err.limit,
                        });
                    } else if let Some(cap_err) = e.downcast_ref::<CapabilityViolationError>() {
                        return Err(SandboxError::CapabilityViolation {
                            capability: cap_err.capability.to_string(),
                        });
                    } else if let Some(wasmtime::Trap::OutOfFuel) =
                        e.downcast_ref::<wasmtime::Trap>()
                    {
                        let consumed = capabilities.max_fuel;
                        return Err(SandboxError::FuelExhausted { consumed });
                    } else {
                        return Err(SandboxError::WasmTrap(format!("{:#}", e)));
                    }
                }
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

    fn admit(bytes: Vec<u8>) -> AdmittedModule {
        AdmittedModule::new(bytes, [0u8; 32])
    }

    #[tokio::test]
    async fn test_invalid_wasm_module_returns_error() -> TestResult {
        let executor = WasmExecutor::new()?;
        let caps = WasmCapabilities::default();
        let admitted = admit(b"not a wasm binary".to_vec());
        let result = executor
            .execute_admitted(&admitted, b"", &caps, Duration::from_secs(1))
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
        let admitted = admit(wasm_bytes);

        let executor = WasmExecutor::new()?;

        let caps_denied = WasmCapabilities {
            allow_cloud_egress: false,
            ..Default::default()
        };
        let res_denied = executor
            .execute_admitted(&admitted, b"", &caps_denied, Duration::from_secs(1))
            .await;
        assert!(
            matches!(
                res_denied,
                Err(SandboxError::CapabilityViolation { ref capability }) if capability == "allow_cloud_egress"
            ),
            "Expected CapabilityViolation for allow_cloud_egress, got: {:?}",
            res_denied
        );

        let caps_allowed = WasmCapabilities {
            allow_cloud_egress: true,
            ..Default::default()
        };
        let res_allowed = executor
            .execute_admitted(&admitted, b"", &caps_allowed, Duration::from_secs(1))
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
        let admitted = admit(wasm_bytes);

        let executor = WasmExecutor::new()?;
        let caps = WasmCapabilities {
            max_fuel: 1_000,
            ..Default::default()
        };

        let result = executor
            .execute_admitted(&admitted, b"", &caps, Duration::from_secs(5))
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
        let admitted = admit(wasm_bytes);

        let executor = WasmExecutor::new()?;
        let caps = WasmCapabilities {
            max_memory_pages: 2,
            ..Default::default()
        };

        let result = executor
            .execute_admitted(&admitted, b"", &caps, Duration::from_secs(5))
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
        let admitted = admit(wasm_bytes);
        let executor = WasmExecutor::new()?;

        let caps = WasmCapabilities::default();
        let output = executor
            .execute_admitted(&admitted, b"", &caps, Duration::from_secs(1))
            .await?;
        assert_eq!(&output.stdout[..], b"hello");

        let caps_no_stdout = WasmCapabilities {
            allow_stdout: false,
            ..Default::default()
        };
        let output_no_stdout = executor
            .execute_admitted(&admitted, b"", &caps_no_stdout, Duration::from_secs(1))
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
        let admitted = admit(wasm_bytes);
        let executor = WasmExecutor::new()?;

        let caps = WasmCapabilities {
            allow_stderr: true,
            ..Default::default()
        };
        let output = executor
            .execute_admitted(&admitted, b"", &caps, Duration::from_secs(1))
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
        let admitted = admit(wasm_bytes);
        let executor = WasmExecutor::new()?;

        let caps = WasmCapabilities {
            max_wall_clock_ms: 50,
            ..Default::default()
        };

        let result_timeout = executor
            .execute_admitted(&admitted, b"", &caps, Duration::from_secs(5))
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
        let admitted = admit(wasm_bytes);
        let executor = WasmExecutor::new()?;
        let caps = WasmCapabilities {
            max_module_size_bytes: 5,
            ..Default::default()
        };

        let result = executor
            .execute_admitted(&admitted, b"", &caps, Duration::from_secs(1))
            .await;

        assert!(
            matches!(result, Err(SandboxError::InvalidModule(ref msg)) if msg.contains("exceeds maximum allowed size")),
            "Expected InvalidModule error due to binary size limit, got: {:?}",
            result
        );
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
        let admitted = admit(wasm_bytes);
        let executor = WasmExecutor::new()?;
        let caps = WasmCapabilities::default();

        let result = executor
            .execute_admitted(&admitted, b"", &caps, Duration::from_secs(1))
            .await;

        assert!(
            matches!(result, Err(SandboxError::WasmTrap(_))),
            "Expected WasmTrap error for unreachable instruction, got: {:?}",
            result
        );
        Ok(())
    }
}
