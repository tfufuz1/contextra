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
    executor::{WasmExecutor, WasmWorkerPool},
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

    /// Returns a reference to the underlying admitted module.
    pub fn admitted_module(&self) -> &AdmittedModule {
        &self.admitted_module
    }

    /// Returns a reference to the executor.
    pub fn executor(&self) -> &WasmExecutor {
        &self.executor
    }
}

/// Registry and fail-closed admission gate for WASM CRDT merge operators (§4.18).
#[derive(Debug)]
pub struct MergeOperatorRegistry {
    admitted: std::sync::RwLock<std::collections::HashMap<[u8; 32], Arc<WasmMergeFunction>>>,
    worker_pool: Option<WasmWorkerPool>,
}

impl Default for MergeOperatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MergeOperatorRegistry {
    /// Creates a new empty `MergeOperatorRegistry`.
    pub fn new() -> Self {
        Self {
            admitted: std::sync::RwLock::new(std::collections::HashMap::new()),
            worker_pool: None,
        }
    }

    /// Creates a new `MergeOperatorRegistry` with a dedicated worker pool.
    pub fn new_with_pool(worker_pool: WasmWorkerPool) -> Self {
        Self {
            admitted: std::sync::RwLock::new(std::collections::HashMap::new()),
            worker_pool: Some(worker_pool),
        }
    }

    /// Admits a WASM merge module after running fail-closed CRDT property-based tests.
    ///
    /// Checks Commutativity (`merge(a,b) == merge(b,a)`), Associativity (`merge(merge(a,b),c) == merge(a,merge(b,c))`),
    /// and Idempotency (`merge(a,a) == a`) across generated test inputs.
    ///
    /// # Errors
    /// Returns `SandboxError` if WASM compilation fails or any CRDT property test fails.
    pub async fn admit(
        &self,
        wasm_bytes: impl Into<Arc<[u8]>>,
    ) -> Result<Arc<WasmMergeFunction>, SandboxError> {
        let wasm_bytes_arc: Arc<[u8]> = wasm_bytes.into();
        let hash = WasmCapabilities::compute_sha256(&wasm_bytes_arc);

        if let Some(existing) = self.get(&hash) {
            return Ok(existing);
        }

        let func = Arc::new(WasmMergeFunction::new_with_capabilities(
            wasm_bytes_arc,
            MergeOperatorCapabilities::pure_with_result_channel(),
        )?);

        self.verify_crdt_properties(&func).await?;

        let mut lock = self.admitted.write().unwrap_or_else(|e| e.into_inner());
        lock.insert(hash, func.clone());
        Ok(func)
    }

    /// Admits an existing `WasmMergeFunction` instance after CRDT property verification.
    pub async fn admit_function(
        &self,
        func: Arc<WasmMergeFunction>,
    ) -> Result<[u8; 32], SandboxError> {
        let hash = WasmCapabilities::compute_sha256(func.admitted_module().bytes());

        if self.is_admitted(&hash) {
            return Ok(hash);
        }

        self.verify_crdt_properties(&func).await?;

        let mut lock = self.admitted.write().unwrap_or_else(|e| e.into_inner());
        lock.insert(hash, func.clone());
        Ok(hash)
    }

    /// Returns `true` if the operator with the given hash has been admitted.
    pub fn is_admitted(&self, hash: &[u8; 32]) -> bool {
        let lock = self.admitted.read().unwrap_or_else(|e| e.into_inner());
        lock.contains_key(hash)
    }

    /// Retrieves an admitted operator by hash.
    pub fn get(&self, hash: &[u8; 32]) -> Option<Arc<WasmMergeFunction>> {
        let lock = self.admitted.read().unwrap_or_else(|e| e.into_inner());
        lock.get(hash).cloned()
    }

    /// Returns a reference to the worker pool if configured.
    pub fn worker_pool(&self) -> Option<&WasmWorkerPool> {
        self.worker_pool.as_ref()
    }

    /// Performs property-based testing over generated input test vectors.
    async fn verify_crdt_properties(&self, func: &WasmMergeFunction) -> Result<(), SandboxError> {
        let test_inputs = generate_crdt_test_vectors();

        for (a, b, c) in test_inputs {
            // 1. Commutativity: merge(a, b) == merge(b, a)
            let ab = func.merge(&a, &b).await?;
            let ba = func.merge(&b, &a).await?;
            if ab != ba {
                return Err(SandboxError::Runtime(format!(
                    "CRDT property violation (Commutativity): merge(a,b) != merge(b,a) for inputs a={:?}, b={:?}",
                    a, b
                )));
            }

            // 2. Associativity: merge(merge(a, b), c) == merge(a, merge(b, c))
            let ab_c = func.merge(&ab, &c).await?;
            let bc = func.merge(&b, &c).await?;
            let a_bc = func.merge(&a, &bc).await?;
            if ab_c != a_bc {
                return Err(SandboxError::Runtime(format!(
                    "CRDT property violation (Associativity): merge(merge(a,b),c) != merge(a,merge(b,c)) for inputs a={:?}, b={:?}, c={:?}",
                    a, b, c
                )));
            }

            // 3. Idempotency: merge(a, a) == a
            let aa = func.merge(&a, &a).await?;
            if aa != a {
                return Err(SandboxError::Runtime(format!(
                    "CRDT property violation (Idempotency): merge(a,a) != a for input a={:?}",
                    a
                )));
            }
        }

        Ok(())
    }
}

fn generate_crdt_test_vectors() -> Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let mut cases = Vec::new();

    // Integers / Counters (u64 LE)
    let n0 = 0u64.to_le_bytes().to_vec();
    let n1 = 1u64.to_le_bytes().to_vec();
    let n2 = 42u64.to_le_bytes().to_vec();
    let n3 = 100u64.to_le_bytes().to_vec();
    let max = u64::MAX.to_le_bytes().to_vec();

    cases.push((n0.clone(), n1.clone(), n2.clone()));
    cases.push((n1.clone(), n2.clone(), n3.clone()));
    cases.push((n2.clone(), n3.clone(), max.clone()));
    cases.push((n1.clone(), n1.clone(), n1.clone()));

    // Deterministic pseudo-random u64 test vectors (SplitMix64)
    let mut state = 0xdeadbeef12345678u64;
    let mut next_prng = || {
        state = state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    };

    for _ in 0..10 {
        let a = next_prng().to_le_bytes().to_vec();
        let b = next_prng().to_le_bytes().to_vec();
        let c = next_prng().to_le_bytes().to_vec();
        cases.push((a, b, c));
    }

    cases
}

/// Facade entrypoint module (`wiring`) for `contextra/src/builder.rs` (§4.18).
pub mod wiring {
    use super::*;

    /// Wiring dependency configuration for Sandbox/WASM subsystem.
    #[derive(Debug, Clone)]
    pub struct SandboxWiringDeps {
        pub max_workers: usize,
    }

    impl Default for SandboxWiringDeps {
        fn default() -> Self {
            Self { max_workers: 16 }
        }
    }

    /// Facade wiring entrypoint for contextra-sandbox.
    #[derive(Debug)]
    pub struct SandboxWiring {
        pub registry: Arc<MergeOperatorRegistry>,
        pub worker_pool: WasmWorkerPool,
    }

    impl SandboxWiring {
        /// Builds the SandboxWiring instance.
        pub fn build(deps: SandboxWiringDeps) -> Result<Self, SandboxError> {
            let worker_pool = WasmWorkerPool::new(deps.max_workers);
            let registry = Arc::new(MergeOperatorRegistry::new_with_pool(worker_pool.clone()));
            Ok(Self {
                registry,
                worker_pool,
            })
        }
    }
}
