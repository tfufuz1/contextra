// FILE-CONTEXT
// STAND: 2026-10-01T00:00:00Z
// ZWECK: Pure, deterministic WASM merge function executor (§4.18)
// INVARIANTEN: No contextra-store dependencies (Ring DAG rule); fresh store/instance per call
// SIEHE AUCH: AGENTS.md §4.18, MergeOperatorCapabilities

//! Pure WASM Merge Executor (§4.18).
//!
//! Provides `WasmMergeFunction` for executing pure WASM merge operations on key-value pairs
//! without side effects or store dependencies.

use std::sync::Arc;
use std::time::Duration;

use crate::{
    admission::AdmittedModule,
    capabilities::{MergeOperatorCapabilities, ModulePolicy, WasmCapabilities},
    error::SandboxError,
    executor::WasmExecutor,
};

/// Standalone, pure, deterministic WASM merge operator executor.
///
/// Encapsulates a pre-validated WASM module and executes pure merge logic
/// over binary inputs via stdin/stdout.
#[derive(Debug)]
pub struct WasmMergeFunction {
    executor: WasmExecutor,
    admitted_module: AdmittedModule,
    caps: WasmCapabilities,
}

impl WasmMergeFunction {
    /// Creates a new `WasmMergeFunction` and validates the WASM binary once upon creation.
    /// Uses default pure merge capabilities (`MergeOperatorCapabilities::pure_with_result_channel()`).
    ///
    /// # Errors
    /// Returns `SandboxError::InvalidModule` if the WASM binary fails validation or exceeds
    /// module size limits, or `SandboxError::Runtime` if engine initialization fails.
    pub fn new(wasm_bytes: impl Into<Arc<[u8]>>) -> Result<Self, SandboxError> {
        Self::new_with_capabilities(
            wasm_bytes,
            MergeOperatorCapabilities::pure_with_result_channel(),
        )
    }

    /// Creates a new `WasmMergeFunction` configured with a strict SHA-256 module allowlist (fail-closed provenance).
    ///
    /// # Errors
    /// Returns `SandboxError::InvalidModule` if the WASM binary is not in the allowlist or fails WASM validation.
    pub fn new_with_allowlist(
        wasm_bytes: impl Into<Arc<[u8]>>,
        allowed_hashes: Vec<[u8; 32]>,
    ) -> Result<Self, SandboxError> {
        Self::new_with_policy(wasm_bytes, ModulePolicy::HashAllowlist(allowed_hashes))
    }

    /// Creates a new `WasmMergeFunction` with a specific module provenance [`ModulePolicy`].
    pub fn new_with_policy(
        wasm_bytes: impl Into<Arc<[u8]>>,
        policy: ModulePolicy,
    ) -> Result<Self, SandboxError> {
        let mut caps = MergeOperatorCapabilities::pure_with_result_channel();
        caps.module_policy = policy;
        Self::new_with_capabilities(wasm_bytes, caps)
    }

    /// Creates a new `WasmMergeFunction` with custom capability limits (e.g. `max_fuel` or `max_wall_clock_ms`).
    pub fn new_with_capabilities(
        wasm_bytes: impl Into<Arc<[u8]>>,
        caps: WasmCapabilities,
    ) -> Result<Self, SandboxError> {
        let executor = WasmExecutor::new()?;
        let wasm_bytes_arc: Arc<[u8]> = wasm_bytes.into();

        executor.validate_module(&wasm_bytes_arc, caps.max_module_size_bytes)?;
        caps.verify_module_policy(&wasm_bytes_arc)?;

        let admitted_module = AdmittedModule::new(wasm_bytes_arc.to_vec(), [0u8; 32]);

        Ok(Self {
            executor,
            admitted_module,
            caps,
        })
    }

    /// Merges two binary values (`existing` and `new`) deterministically using the WASM module.
    ///
    /// # Input Encoding
    /// Binary stream passed to guest stdin:
    /// `u32 LE len(existing) | existing bytes | u32 LE len(new) | new bytes`
    ///
    /// # Output Encoding
    /// Result byte stream written to stdout by the guest module, bounded by `max_output_bytes`.
    ///
    /// # Errors
    /// Returns `SandboxError` on fuel exhaustion, timeout, memory exceeded, WASM trap,
    /// process exit error, or capability violation without panicking.
    pub async fn merge(&self, existing: &[u8], new: &[u8]) -> Result<Vec<u8>, SandboxError> {
        let caps = &self.caps;

        let len_existing =
            u32::try_from(existing.len()).map_err(|_| SandboxError::InputTooLarge {
                len: existing.len(),
                limit: caps.max_stdin_bytes,
            })?;
        let len_new = u32::try_from(new.len()).map_err(|_| SandboxError::InputTooLarge {
            len: new.len(),
            limit: caps.max_stdin_bytes,
        })?;

        let total_stdin_len = 8usize
            .checked_add(existing.len())
            .and_then(|l| l.checked_add(new.len()))
            .ok_or_else(|| SandboxError::InputTooLarge {
                len: existing.len().saturating_add(new.len()).saturating_add(8),
                limit: caps.max_stdin_bytes,
            })?;

        if total_stdin_len > caps.max_stdin_bytes {
            return Err(SandboxError::InputTooLarge {
                len: total_stdin_len,
                limit: caps.max_stdin_bytes,
            });
        }

        let mut input = Vec::with_capacity(total_stdin_len);
        input.extend_from_slice(&len_existing.to_le_bytes());
        input.extend_from_slice(existing);
        input.extend_from_slice(&len_new.to_le_bytes());
        input.extend_from_slice(new);

        let timeout = Duration::from_millis(caps.max_wall_clock_ms);
        let output = self
            .executor
            .execute_admitted(&self.admitted_module, &input, caps, timeout)
            .await?;

        Ok(output.stdout.to_vec())
    }
}
