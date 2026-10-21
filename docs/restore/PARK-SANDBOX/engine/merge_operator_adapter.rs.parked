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
//! to an explicitly provided Tokio runtime handle (`tokio::runtime::Handle`) and leverages the LRU
//! module cache in [`contextra_sandbox`].
//! Operators can be registered into a [`MergeOperatorHost`] implementation after passing the fail-closed CRDT gate.

use contextra_ports::MergeOperatorHost;
use contextra_store::MergeOperator;
use contextra_types::{ContextraError, Result};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

/// Synchronous wrapper around [`contextra_sandbox::WasmMergeFunction`] implementing [`MergeOperator`].
#[derive(Clone)]
pub struct WasmMergeOperator {
    func: Arc<contextra_sandbox::WasmMergeFunction>,
    recv_timeout: Duration,
    handle: tokio::runtime::Handle,
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
    /// Creates a new `WasmMergeOperator` using active Tokio context handle with default capability limits.
    pub fn new(wasm_bytes: impl Into<Arc<[u8]>>) -> Result<Self> {
        let handle = tokio::runtime::Handle::try_current().map_err(|e| {
            ContextraError::Internal(format!("No Tokio runtime context available: {e}"))
        })?;
        Self::new_with_handle(wasm_bytes, handle)
    }

    /// Creates a new `WasmMergeOperator` with an explicit [`tokio::runtime::Handle`] and default capability limits.
    pub fn new_with_handle(
        wasm_bytes: impl Into<Arc<[u8]>>,
        handle: tokio::runtime::Handle,
    ) -> Result<Self> {
        let caps = contextra_sandbox::MergeOperatorCapabilities::pure_with_result_channel();
        Self::new_with_config_and_handle(
            wasm_bytes,
            caps.max_fuel,
            Duration::from_millis(caps.max_wall_clock_ms),
            handle,
        )
    }

    /// Creates a new `WasmMergeOperator` using active Tokio context handle with custom limits.
    pub fn new_with_config(
        wasm_bytes: impl Into<Arc<[u8]>>,
        max_fuel: u64,
        wall_clock_timeout: Duration,
    ) -> Result<Self> {
        let handle = tokio::runtime::Handle::try_current().map_err(|e| {
            ContextraError::Internal(format!("No Tokio runtime context available: {e}"))
        })?;
        Self::new_with_config_and_handle(wasm_bytes, max_fuel, wall_clock_timeout, handle)
    }

    /// Creates a new `WasmMergeOperator` with custom `max_fuel`, `wall_clock_timeout`, and an explicit [`tokio::runtime::Handle`].
    ///
    /// # Errors
    /// Returns `ContextraError::Sandbox` if WASM validation fails.
    pub fn new_with_config_and_handle(
        wasm_bytes: impl Into<Arc<[u8]>>,
        max_fuel: u64,
        wall_clock_timeout: Duration,
        handle: tokio::runtime::Handle,
    ) -> Result<Self> {
        let wasm_bytes = wasm_bytes.into();
        let mut caps = contextra_sandbox::MergeOperatorCapabilities::pure_with_result_channel();
        caps.max_fuel = max_fuel;
        caps.max_wall_clock_ms = wall_clock_timeout.as_millis() as u64;

        // Wall-clock limit plus 2 second reserve for thread communication / task queueing
        let recv_timeout = Duration::from_millis(caps.max_wall_clock_ms.saturating_add(2_000));

        // Note (W1-17): WasmMergeFunction internally encapsulates WasmExecutor which maintains
        // an LRU ModuleCache for compiled WASM modules.
        let func = Arc::new(
            contextra_sandbox::WasmMergeFunction::new_with_capabilities(wasm_bytes, caps)
                .map_err(|e| {
                    ContextraError::Sandbox(format!(
                        "WASM validation failed: {}",
                        map_sandbox_err(&e)
                    ))
                })?,
        );

        Ok(Self {
            func,
            recv_timeout,
            handle,
        })
    }

    /// Returns a reference to the underlying [`contextra_sandbox::WasmMergeFunction`].
    pub fn inner(&self) -> &Arc<contextra_sandbox::WasmMergeFunction> {
        &self.func
    }

    /// Admits and registers the WASM merge operator into the given [`MergeOperatorHost`]
    /// after passing fail-closed CRDT property verification (Commutativity, Associativity, Idempotency).
    ///
    /// # Errors
    /// Returns `ContextraError::Sandbox` if CRDT property verification fails.
    pub fn admit_into_registry(&self, host: Arc<dyn MergeOperatorHost>) -> Result<[u8; 32]> {
        let func = self.func.clone();
        let handle = self.handle.clone();
        let (tx, rx) = mpsc::channel();

        tokio::task::block_in_place(move || {
            handle.spawn(async move {
                let res = host.admit_wasm_function(func).await;
                let _ = tx.send(res);
            });
        });

        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(hash)) => Ok(hash),
            Ok(Err(err)) => Err(err),
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
        let handle = self.handle.clone();

        let (tx, rx) = mpsc::channel();

        tokio::task::block_in_place(move || {
            handle.spawn(async move {
                let res = func.merge(&existing, &new).await;
                let _ = tx.send(res);
            });
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

fn map_sandbox_err(err: &contextra_sandbox::SandboxError) -> &'static str {
    match err {
        contextra_sandbox::SandboxError::Timeout { .. } => "timeout exceeded",
        contextra_sandbox::SandboxError::FuelExhausted { .. } => "fuel exhausted",
        contextra_sandbox::SandboxError::MemoryExceeded { .. } => "memory limit exceeded",
        contextra_sandbox::SandboxError::InvalidModule(_) => "invalid module",
        contextra_sandbox::SandboxError::WasmTrap(_) => "WASM execution trap",
        contextra_sandbox::SandboxError::ProcessExit { .. } => "process exit error",
        contextra_sandbox::SandboxError::CapabilityViolation { .. } => "capability violation",
        contextra_sandbox::SandboxError::OutputLimitExceeded { .. } => "output limit exceeded",
        contextra_sandbox::SandboxError::InputTooLarge { .. } => "input too large",
        contextra_sandbox::SandboxError::Admission(_) => "module admission error",
        contextra_sandbox::SandboxError::Runtime(_) => "runtime initialization error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeMergeOperatorHost {
        expected_digest: [u8; 32],
        should_fail: bool,
    }

    impl MergeOperatorHost for FakeMergeOperatorHost {
        fn admit_wasm_function<'a>(
            &'a self,
            func: Arc<dyn std::any::Any + Send + Sync>,
        ) -> contextra_ports::BoxFuture<'a, Result<[u8; 32]>> {
            Box::pin(async move {
                if self.should_fail {
                    Err(ContextraError::Sandbox(
                        "CRDT admission gate failed: non-commutative".into(),
                    ))
                } else {
                    assert!(func.is::<contextra_sandbox::WasmMergeFunction>());
                    Ok(self.expected_digest)
                }
            })
        }
    }

    #[tokio::test]
    async fn test_admit_into_registry_fake_host_success() {
        let wasm_bytes = &[
            0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 7, 10, 1, 6, 95, 115,
            116, 97, 114, 116, 0, 0, 10, 9, 1, 7, 0, 3, 64, 12, 0, 11, 11,
        ];
        let op = WasmMergeOperator::new(&wasm_bytes[..]).expect("valid operator");
        let expected_digest = [0xAB; 32];
        let host = Arc::new(FakeMergeOperatorHost {
            expected_digest,
            should_fail: false,
        });

        let res = op.admit_into_registry(host).expect("admission succeeds");
        assert_eq!(res, expected_digest);
    }

    #[tokio::test]
    async fn test_admit_into_registry_fake_host_failure() {
        let wasm_bytes = &[
            0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 7, 10, 1, 6, 95, 115,
            116, 97, 114, 116, 0, 0, 10, 9, 1, 7, 0, 3, 64, 12, 0, 11, 11,
        ];
        let op = WasmMergeOperator::new(&wasm_bytes[..]).expect("valid operator");
        let host = Arc::new(FakeMergeOperatorHost {
            expected_digest: [0; 32],
            should_fail: true,
        });

        let res = op.admit_into_registry(host);
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("CRDT admission gate failed"));
    }
}
