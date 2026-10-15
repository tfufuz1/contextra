// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: WASM Sandbox Subsystem Wiring for contextra facade (Ring 4).
// INVARIANTEN: #![forbid(unsafe_code)]; SandboxWiring bundles verifier, capabilities, approval state machine.

#![forbid(unsafe_code)]

#[cfg(feature = "sandbox")]
use contextra_types::ContextraError;
#[cfg(feature = "sandbox")]
use contextra_sandbox::wiring::{SandboxWiring, SandboxWiringConfig};

/// Bundles configured WASM Sandbox capabilities.
#[cfg(feature = "sandbox")]
#[derive(Clone)]
pub struct SandboxWiringBundle {
    pub sandbox: SandboxWiring,
    pub enabled: bool,
}

#[cfg(feature = "sandbox")]
impl SandboxWiringBundle {
    /// Builds `SandboxWiringBundle` from configuration.
    pub fn build(enabled: bool) -> Result<Self, ContextraError> {
        let config = SandboxWiringConfig::default();
        let sandbox = SandboxWiring::build(config)
            .map_err(|e| ContextraError::PolicyViolation(e.to_string()))?;

        Ok(Self { sandbox, enabled })
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
