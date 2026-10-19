// FILE-CONTEXT
// ZWECK: Synchronous bridge from contextra-store MergeOperator to async WasmMergeFunction
// INVARIANTEN: Fail-Safe (always return Err on error, never panic); Thread/Runtime context independent; #![forbid(unsafe_code)]
// SIEHE AUCH: AGENTS.md §4.18, Spec §4.12, contextra-sandbox WasmMergeFunction

//! Synchronous bridge adapter for executing WASM merge functions within storage compaction.
//!
//! # Architecture & Guidance
//! The storage engine (`LsmStorage` / `CompactionEngine`) passes the [`WasmMergeOperator`]
//! to background SSTable compaction threads.
//!
//! To prevent OS thread and runtime exhaustion (W1-17), [`WasmMergeOperator`] delegates execution
//! to a shared Tokio runtime pool and leverages the LRU module cache in [`contextra_sandbox`].
//! Operatoren can be registered into a [`MergeOperatorRegistry`] after passing the fail-closed CRDT gate.

use contextra_sandbox::merge::MergeOperatorRegistry;
use contextra_sandbox::{
    MergeOperatorCapabilities, SandboxError, WasmMergeFunction,
};
use contextra_store::MergeOperator;
use contextra_types::{ContextraError, Result};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

static SHARED_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn get_shared_runtime() -> Result<&'static tokio::runtime::Runtime> {
    if let Some(rt) = SHARED_RUNTIME.get() {
        Ok(rt)
    } else {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .thread_name("wasm-merge-runtime")
            .enable_all()
            .build()
            .map_err(|e| {
                ContextraError::Internal(format!("Failed to build WASM merge runtime: {e}"))
            })?;
        let _ = SHARED_RUNTIME.set(rt);
        SHARED_RUNTIME
            .get()
            .ok_or_else(|| ContextraError::Internal("Runtime init failed".into()))
    }
}

/// Synchronous wrapper around [`WasmMergeFunction`] implementing [`MergeOperator`].
#[derive(Clone)]
pub struct WasmMergeOperator {
    func: Arc<WasmMergeFunction>,
    recv_timeout: Duration,
}

impl std::fmt::Debug for WasmMergeOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmMergeOperator")
            .field(
                "admitted_module_digest",
                &self.func.admitted_module().digest(),
            )
            .field("recv_timeout", &self.recv_timeout)
            .finish()
    }
}

impl WasmMergeOperator {
    /// Creates a new `WasmMergeOperator` with default capability limits (10,000,000 fuel, 5s timeout).
    pub fn new(wasm_bytes: impl Into<Arc<[u8]>>) -> Result<Self> {
        let caps = MergeOperatorCapabilities::pure_with_result_channel();
        Self::new_with_config(
            wasm_bytes,
            caps.max_fuel,
            Duration::from_millis(caps.max_wall_clock_ms),
        )
    }

    /// Creates a new `WasmMergeOperator` with custom `max_fuel` and `wall_clock_timeout` configuration.
    ///
    /// # Errors
    /// Returns `ContextraError::Sandbox` if WASM validation fails.
    pub fn new_with_config(
        wasm_bytes: impl Into<Arc<[u8]>>,
        max_fuel: u64,
        wall_clock_timeout: Duration,
    ) -> Result<Self> {
        let wasm_bytes = wasm_bytes.into();
        let mut caps = MergeOperatorCapabilities::pure_with_result_channel();
        caps.max_fuel = max_fuel;
        caps.max_wall_clock_ms = wall_clock_timeout.as_millis() as u64;

        // Wall-clock limit plus 2 second reserve for thread communication / task queueing
        let recv_timeout = Duration::from_millis(caps.max_wall_clock_ms.saturating_add(2_000));

        // Note (W1-17): WasmMergeFunction internally encapsulates WasmExecutor which maintains
        // an LRU ModuleCache for compiled WASM modules. Worker pool concurrency is bounded
        // when execution tasks run on the shared runtime pool.
        let func = Arc::new(
            WasmMergeFunction::new_with_capabilities(wasm_bytes, caps).map_err(|e| {
                ContextraError::Sandbox(format!("WASM validation failed: {}", map_sandbox_err(&e)))
            })?,
        );

        Ok(Self { func, recv_timeout })
    }

    /// Returns a reference to the underlying [`WasmMergeFunction`].
    pub fn inner(&self) -> &Arc<WasmMergeFunction> {
        &self.func
    }

    /// Admits and registers the WASM merge operator into the given [`MergeOperatorRegistry`]
    /// after passing fail-closed CRDT property verification (Commutativity, Associativity, Idempotency).
    ///
    /// # Errors
    /// Returns `ContextraError::Sandbox` if CRDT property verification fails.
    pub fn admit_into_registry(&self, registry: Arc<MergeOperatorRegistry>) -> Result<[u8; 32]> {
        let func = self.func.clone();
        let rt = get_shared_runtime()?;
        let (tx, rx) = mpsc::channel();

        rt.spawn(async move {
            let res = registry.admit_function(func).await;
            let _ = tx.send(res);
        });

        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(hash)) => Ok(hash),
            Ok(Err(sandbox_err)) => Err(ContextraError::Sandbox(format!(
                "CRDT admission gate failed: {}",
                map_sandbox_err_dyn(&sandbox_err)
            ))),
            Err(mpsc::RecvTimeoutError::Timeout) => Err(ContextraError::Sandbox(
                "CRDT admission gate execution timed out".into(),
            )),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(ContextraError::Sandbox(
                "CRDT admission worker disconnected".into(),
            )),
        }
    }
}

impl MergeOperator for WasmMergeOperator {
    fn merge(&self, existing_val: &[u8], new_val: &[u8]) -> Result<Vec<u8>> {
        let func = self.func.clone();
        let existing = existing_val.to_vec();
        let new = new_val.to_vec();
        let rt = match get_shared_runtime() {
            Ok(rt) => rt,
            Err(e) => {
                tracing::warn!("WasmMergeOperator shared runtime initialization failed: {e}");
                return Err(ContextraError::Internal(format!(
                    "Shared runtime initialization error: {e}"
                )));
            }
        };

        let (tx, rx) = mpsc::channel();

        rt.spawn(async move {
            let res = func.merge(&existing, &new).await;
            let _ = tx.send(res);
        });

        match rx.recv_timeout(self.recv_timeout) {
            Ok(Ok(val)) => Ok(val),
            Ok(Err(sandbox_err)) => {
                let err_msg = map_sandbox_err(&sandbox_err);
                tracing::warn!("WasmMergeOperator guest execution failed: {err_msg}");
                Err(ContextraError::Sandbox(format!(
                    "WASM merge function failed: {err_msg}"
                )))
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                tracing::warn!("WasmMergeOperator execution timed out waiting for shared worker response");
                Err(ContextraError::Sandbox(
                    "WASM merge function execution timed out".into(),
                ))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                tracing::warn!("WasmMergeOperator worker task disconnected or panicked");
                Err(ContextraError::Sandbox(
                    "WASM merge worker task disconnected".into(),
                ))
            }
        }
    }
}

fn map_sandbox_err(err: &SandboxError) -> &'static str {
    match err {
        SandboxError::Timeout { .. } => "timeout exceeded",
        SandboxError::FuelExhausted { .. } => "fuel exhausted",
        SandboxError::MemoryExceeded { .. } => "memory limit exceeded",
        SandboxError::InvalidModule(_) => "invalid module",
        SandboxError::WasmTrap(_) => "WASM execution trap",
        SandboxError::ProcessExit { .. } => "process exit error",
        SandboxError::CapabilityViolation { .. } => "capability violation",
        SandboxError::OutputLimitExceeded { .. } => "output limit exceeded",
        SandboxError::InputTooLarge { .. } => "input too large",
        SandboxError::Admission(_) => "module admission error",
        SandboxError::Runtime(_) => "runtime initialization error",
    }
}

fn map_sandbox_err_dyn(err: &SandboxError) -> String {
    match err {
        SandboxError::Runtime(msg) => format!("runtime error: {msg}"),
        other => map_sandbox_err(other).to_string(),
    }
}
