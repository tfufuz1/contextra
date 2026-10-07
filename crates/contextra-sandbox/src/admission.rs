//! WASM Module Admission Control (§4.18).
//!
//! Provides strict pre-execution verification for WASM binaries. Persisted or raw WASM modules
//! must be verified and wrapped in an [`AdmittedModule`] before execution via
//! [`WasmExecutor::execute_admitted`].

use crate::error::AdmissionError;

/// Pre-verified WASM module wrapper required for execution.
///
/// Fields are kept strictly private so external callers cannot bypass admission controls
/// or instantiate invalid module states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedModule {
    bytes: Vec<u8>,
    digest: [u8; 32],
}

impl AdmittedModule {
    /// Constructs a new [`AdmittedModule`] from verified WASM binary bytes and its 32-byte digest.
    ///
    /// Private fields prevent direct struct literal instantiation.
    pub fn new(bytes: Vec<u8>, digest: [u8; 32]) -> Self {
        Self { bytes, digest }
    }

    /// Returns a reference to the verified WASM binary bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns a reference to the module's 32-byte digest.
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// Consumes the [`AdmittedModule`] and returns the inner WASM binary bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// Trait for verifying WASM module bytes prior to admission.
///
/// Explicitly constructed verifiers perform cryptographic or policy verification over
/// WASM binary payloads and construct an [`AdmittedModule`] upon success.
pub trait ModuleVerifier {
    /// Verifies the provided WASM binary bytes.
    ///
    /// # Errors
    /// Returns [`AdmissionError`] if verification fails (e.g., unsigned module, invalid signature, or digest mismatch).
    fn verify(&self, bytes: &[u8]) -> Result<AdmittedModule, AdmissionError>;
}
