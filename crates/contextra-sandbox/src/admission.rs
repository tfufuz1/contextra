//! WASM Module Admission Control (§4.18).
//!
//! Provides strict pre-execution verification for WASM binaries. Persisted or raw WASM modules
//! must be verified and wrapped in an [`AdmittedModule`] before execution via
//! [`WasmExecutor::execute_admitted`].

use crate::capabilities::WasmCapabilities;
use crate::error::AdmissionError;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

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

/// Ed25519-based module verifier enforcing strict signature checks and policy limits (§4.18).
#[derive(Debug, Clone, Default)]
pub struct Ed25519ModuleVerifier {
    trusted_keys: Vec<[u8; 32]>,
    expires_at_unix_ms: Option<u64>,
    now_unix_ms: Option<u64>,
}

impl Ed25519ModuleVerifier {
    /// Constructs a new [`Ed25519ModuleVerifier`] with a list of trusted Ed25519 32-byte public keys.
    pub fn new(trusted_keys: Vec<[u8; 32]>) -> Self {
        Self {
            trusted_keys,
            expires_at_unix_ms: None,
            now_unix_ms: None,
        }
    }

    /// Sets an optional policy expiration timestamp in Unix milliseconds.
    pub fn with_expiration(mut self, expires_at_unix_ms: u64) -> Self {
        self.expires_at_unix_ms = Some(expires_at_unix_ms);
        self
    }

    /// Sets an optional evaluation timestamp in Unix milliseconds for verification.
    pub fn with_now(mut self, now_unix_ms: u64) -> Self {
        self.now_unix_ms = Some(now_unix_ms);
        self
    }

    /// Prepares a signed binary payload by prepending an Ed25519 signature over `wasm_bytes`.
    pub fn sign_payload(signing_key: &SigningKey, wasm_bytes: &[u8]) -> Vec<u8> {
        let signature = signing_key.sign(wasm_bytes);
        let mut payload = Vec::with_capacity(64 + wasm_bytes.len());
        payload.extend_from_slice(&signature.to_bytes());
        payload.extend_from_slice(wasm_bytes);
        payload
    }

    /// Verifies the signed WASM payload at the given optional Unix timestamp in milliseconds.
    pub fn verify_at(
        &self,
        bytes: &[u8],
        now_unix_ms: Option<u64>,
    ) -> Result<AdmittedModule, AdmissionError> {
        if let Some(expires_at) = self.expires_at_unix_ms {
            match now_unix_ms {
                Some(now) if now >= expires_at => {
                    return Err(AdmissionError::VerificationFailed(format!(
                        "Module policy expired at {expires_at}ms (evaluated at {now}ms)"
                    )));
                }
                None => {
                    return Err(AdmissionError::VerificationFailed(
                        "Module policy has expiration configured but evaluation timestamp was not provided".into(),
                    ));
                }
                _ => {}
            }
        }

        if bytes.len() < 64 {
            return Err(AdmissionError::UnsignedModule);
        }

        let sig_bytes: [u8; 64] = match bytes[..64].try_into() {
            Ok(arr) => arr,
            Err(_) => {
                return Err(AdmissionError::InvalidSignature(
                    "Signature header payload invalid".into(),
                ))
            }
        };

        let signature = Signature::from_bytes(&sig_bytes);
        let wasm_payload = &bytes[64..];

        if self.trusted_keys.is_empty() {
            return Err(AdmissionError::InvalidSignature(
                "Verification failed: no trusted Ed25519 public keys configured".into(),
            ));
        }

        let mut matched = false;
        for pk_bytes in &self.trusted_keys {
            if let Ok(verifying_key) = VerifyingKey::from_bytes(pk_bytes) {
                if verifying_key.verify(wasm_payload, &signature).is_ok() {
                    matched = true;
                    break;
                }
            }
        }

        if !matched {
            return Err(AdmissionError::InvalidSignature(
                "Ed25519 signature verification failed or signing key not in trusted list".into(),
            ));
        }

        let digest = WasmCapabilities::compute_sha256(wasm_payload);
        Ok(AdmittedModule::new(wasm_payload.to_vec(), digest))
    }
}

impl ModuleVerifier for Ed25519ModuleVerifier {
    fn verify(&self, bytes: &[u8]) -> Result<AdmittedModule, AdmissionError> {
        self.verify_at(bytes, self.now_unix_ms)
    }
}
