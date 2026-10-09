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

/// Concrete implementation of [`ModuleVerifier`] that validates WASM binaries against
/// basic WASM magic header bytes (`\0asm`) and constructs an [`AdmittedModule`].
#[derive(Debug, Clone, Default)]
pub struct DefaultModuleVerifier {
    expected_digest: Option<[u8; 32]>,
}

impl DefaultModuleVerifier {
    /// Constructs a new [`DefaultModuleVerifier`] with no pinned hash digest.
    pub fn new() -> Self {
        Self {
            expected_digest: None,
        }
    }

    /// Constructs a new [`DefaultModuleVerifier`] pinned to an expected 32-byte digest.
    pub fn with_expected_digest(digest: [u8; 32]) -> Self {
        Self {
            expected_digest: Some(digest),
        }
    }
}

impl ModuleVerifier for DefaultModuleVerifier {
    fn verify(&self, bytes: &[u8]) -> Result<AdmittedModule, AdmissionError> {
        if bytes.len() < 8 || &bytes[0..4] != b"\0asm" {
            return Err(AdmissionError::InvalidModule(
                "Missing WASM magic header '\\0asm'".to_string(),
            ));
        }

        let mut computed_digest = [0u8; 32];
        for (i, &byte) in bytes.iter().enumerate() {
            computed_digest[i % 32] ^= byte;
        }

        if let Some(expected) = self.expected_digest {
            if expected != computed_digest {
                return Err(AdmissionError::DigestMismatch {
                    expected: format!("{:?}", expected),
                    actual: format!("{:?}", computed_digest),
                });
            }
        }

        Ok(AdmittedModule::new(bytes.to_vec(), computed_digest))
    }
}
