// FILE-CONTEXT
// ZWECK: Synchronous bridge from contextra-store MergeOperator to async WasmMergeFunction
// INVARIANTEN: Fail-Safe (always return Err on error, never panic); Thread/Runtime context independent; #![forbid(unsafe_code)]
// SIEHE AUCH: AGENTS.md §4.18, Spec §4.12, contextra-sandbox WasmMergeFunction

//! Synchronous bridge adapter for executing WASM merge functions within storage compaction.
//!
//! # Architecture & Guidance
//! The storage engine (`LsmStorage` / `CompactionEngine`) passes the [`WasmMergeOperator`]
//! to background SSTable compaction threads. Because compaction can run in arbitrary contexts
//! (standard OS threads, `spawn_blocking`, or Tokio async tasks), [`WasmMergeOperator`] bridges
//! synchronous [`contextra_store::MergeOperator`] invocations to the asynchronous
//! [`contextra_sandbox::WasmMergeFunction`] via a dedicated worker thread hosting an isolated
//! single-threaded Tokio runtime.
//!
//! # Safety & Determinism
//! - **Pure Modules Only**: Guest WASM modules MUST be pure and free of side effects.
//! - **Determinism**: Identical byte inputs MUST yield byte-identical outputs across runs.
//! - **Fail-Safe Contract**: Compaction engine expects errors on failures. On any error
//!   (timeout, trap, fuel exhaustion, or channel error), `merge` logs a `tracing::warn!`
//!   and returns a [`ContextraError`], preserving both versions in SSTable compaction without aborting.

use contextra_sandbox::{MergeOperatorCapabilities, SandboxError, WasmMergeFunction};
use contextra_store::MergeOperator;
use contextra_types::{ContextraError, Result};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Request payload sent to worker thread.
struct MergeJob {
    existing: Vec<u8>,
    new: Vec<u8>,
    respond_to: mpsc::Sender<std::result::Result<Vec<u8>, SandboxError>>,
}

/// Synchronous wrapper around [`WasmMergeFunction`] implementing [`MergeOperator`].
pub struct WasmMergeOperator {
    tx: Option<Sender<MergeJob>>,
    worker_handle: Option<JoinHandle<()>>,
    recv_timeout: Duration,
}

impl std::fmt::Debug for WasmMergeOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmMergeOperator")
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
    /// Returns `ContextraError::Sandbox` if WASM validation fails, or `ContextraError::Internal`
    /// if the worker thread or runtime initialization fails.
    pub fn new_with_config(
        wasm_bytes: impl Into<Arc<[u8]>>,
        max_fuel: u64,
        wall_clock_timeout: Duration,
    ) -> Result<Self> {
        let wasm_bytes = wasm_bytes.into();
        let mut caps = MergeOperatorCapabilities::pure_with_result_channel();
        caps.max_fuel = max_fuel;
        caps.max_wall_clock_ms = wall_clock_timeout.as_millis() as u64;

        let merge_fn = Arc::new(
            WasmMergeFunction::new_with_capabilities(wasm_bytes, caps.clone())
                .map_err(|e| ContextraError::Sandbox(format!("WASM validation failed: {}", map_sandbox_err(&e))))?,
        );

        // Wall-clock limit plus 1 second reserve for thread communication
        let recv_timeout = Duration::from_millis(caps.max_wall_clock_ms.saturating_add(1_000));

        let (tx, rx): (Sender<MergeJob>, Receiver<MergeJob>) = mpsc::channel();

        let merge_fn_clone = merge_fn.clone();
        let handle = thread::Builder::new()
            .name("wasm-merge-worker".into())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_time()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        tracing::error!("Failed to create tokio runtime in WasmMergeOperator worker: {e}");
                        return;
                    }
                };

                rt.block_on(async move {
                    while let Ok(job) = rx.recv() {
                        let res = merge_fn_clone.merge(&job.existing, &job.new).await;
                        let _ = job.respond_to.send(res);
                    }
                });
            })
            .map_err(|e| ContextraError::Internal(format!("Failed to spawn WASM merge worker thread: {e}")))?;

        Ok(Self {
            tx: Some(tx),
            worker_handle: Some(handle),
            recv_timeout,
        })
    }
}

impl MergeOperator for WasmMergeOperator {
    fn merge(&self, existing_val: &[u8], new_val: &[u8]) -> Result<Vec<u8>> {
        let tx = self.tx.as_ref().ok_or_else(|| {
            ContextraError::Sandbox("WASM merge operator worker disconnected".into())
        })?;

        let (respond_tx, respond_rx) = mpsc::channel();
        let job = MergeJob {
            existing: existing_val.to_vec(),
            new: new_val.to_vec(),
            respond_to: respond_tx,
        };

        if let Err(e) = tx.send(job) {
            tracing::warn!("WasmMergeOperator worker channel send error: {e}");
            return Err(ContextraError::Sandbox("Merge worker channel disconnected".into()));
        }

        match respond_rx.recv_timeout(self.recv_timeout) {
            Ok(Ok(val)) => Ok(val),
            Ok(Err(sandbox_err)) => {
                let err_msg = map_sandbox_err(&sandbox_err);
                tracing::warn!("WasmMergeOperator guest execution failed: {err_msg}");
                Err(ContextraError::Sandbox(format!("WASM merge function failed: {err_msg}")))
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                tracing::warn!("WasmMergeOperator execution timed out waiting for worker response");
                Err(ContextraError::Sandbox("WASM merge function execution timed out".into()))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                tracing::warn!("WasmMergeOperator worker thread disconnected or panicked");
                Err(ContextraError::Sandbox("WASM merge worker thread disconnected".into()))
            }
        }
    }
}

impl Drop for WasmMergeOperator {
    fn drop(&mut self) {
        // Drop sender first so rx.recv() in worker loop receives Disconnected and terminates worker thread.
        self.tx.take();
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
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
        SandboxError::Runtime(_) => "runtime initialization error",
    }
}
