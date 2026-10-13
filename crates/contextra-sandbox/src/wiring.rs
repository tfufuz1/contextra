//! Sandbox Wiring (§4.18, Rule 7).
//!
//! Bundles module admission verifier, capability policies, and the human approval workflow state machine.

use std::sync::Arc;

use crate::admission::{AdmittedModule, Ed25519ModuleVerifier, ModuleVerifier};
use crate::approval::{classify_risk, ApprovalRequest, ApprovalRisk, ApprovalTransitionError};
use crate::capabilities::{ModulePolicy, WasmCapabilities};
use crate::error::{AdmissionError, SandboxError};

/// Configuration dependencies for building [`SandboxWiring`].
#[derive(Clone, Default)]
pub struct SandboxWiringConfig {
    /// Configured execution capability policy. Default: `WasmCapabilities::default()`.
    pub capabilities: WasmCapabilities,
    /// Explicit module verifier implementation.
    ///
    /// If `None` and `capabilities.module_policy` is [`ModulePolicy::SignedBy`],
    /// an [`Ed25519ModuleVerifier`] is instantiated automatically.
    pub verifier: Option<Arc<dyn ModuleVerifier + Send + Sync>>,
}

/// Composition root and facade bundle for the WASM sandbox subsystem.
#[derive(Clone)]
pub struct SandboxWiring {
    capabilities: WasmCapabilities,
    verifier: Arc<dyn ModuleVerifier + Send + Sync>,
}

impl SandboxWiring {
    /// Constructs a new [`SandboxWiring`] instance from configuration dependencies.
    ///
    /// # Errors
    /// Returns [`SandboxError`] if the configuration is invalid.
    pub fn build(config: SandboxWiringConfig) -> Result<Self, SandboxError> {
        let verifier: Arc<dyn ModuleVerifier + Send + Sync> = match config.verifier {
            Some(v) => v,
            None => match &config.capabilities.module_policy {
                ModulePolicy::SignedBy(keys) => {
                    Arc::new(Ed25519ModuleVerifier::new(keys.clone()))
                }
                _ => Arc::new(Ed25519ModuleVerifier::default()),
            },
        };

        Ok(Self {
            capabilities: config.capabilities,
            verifier,
        })
    }

    /// Returns a reference to the active [`WasmCapabilities`].
    pub fn capabilities(&self) -> &WasmCapabilities {
        &self.capabilities
    }

    /// Returns a reference to the active [`ModuleVerifier`].
    pub fn verifier(&self) -> &(dyn ModuleVerifier + Send + Sync) {
        &*self.verifier
    }

    /// Verifies raw or signed WASM binary bytes against the active [`ModuleVerifier`] and capability policy.
    ///
    /// # Errors
    /// Returns [`AdmissionError`] if module admission or signature verification fails.
    pub fn verify_module(&self, wasm_bytes: &[u8]) -> Result<AdmittedModule, AdmissionError> {
        self.verifier.verify(wasm_bytes)
    }

    /// Classifies the execution risk level of the active capability policy.
    pub fn classify_risk(&self) -> ApprovalRisk {
        classify_risk(&self.capabilities)
    }

    /// Creates a new [`ApprovalRequest`] for the active capability policy.
    pub fn create_approval_request(
        &self,
        request_id: String,
        created_at_unix_ms: u64,
        ttl_ms: u64,
    ) -> ApprovalRequest {
        ApprovalRequest::new(request_id, &self.capabilities, created_at_unix_ms, ttl_ms)
    }

    /// Evaluates an execution request against human approval policy rules.
    ///
    /// Low-risk requests are auto-approved via `auto_approve_if_low_risk`.
    /// Elevated or High-risk requests remain in `Pending` state awaiting explicit human approval.
    ///
    /// # Errors
    /// Returns [`ApprovalTransitionError`] if an auto-approval state transition fails (e.g. expired request).
    pub fn evaluate_execution(
        &self,
        request_id: String,
        system_identity: String,
        created_at_unix_ms: u64,
        ttl_ms: u64,
        now_unix_ms: u64,
    ) -> Result<ApprovalRequest, ApprovalTransitionError> {
        let request = self.create_approval_request(request_id, created_at_unix_ms, ttl_ms);
        request.auto_approve_if_low_risk(system_identity, now_unix_ms)
    }
}
