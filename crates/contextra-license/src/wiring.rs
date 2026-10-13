//! License gate wiring module for contextra facade composition (§14.6, §15).

use std::path::PathBuf;
use std::sync::Arc;

use contextra_ports::clock::{Clock, SystemClock};
use contextra_ports::license::LicenseGate;
use contextra_types::ContextraError;

use crate::signed_gate::{LicenseKeyring, LicensePayload, SignedActivation, SignedLicenseGate};
use crate::LicenseGateMode;

/// Dependencies input for building [`LicenseWiring`].
#[derive(Clone)]
pub struct LicenseWiringDeps {
    /// License gate enforcement mode (defaults to `LicenseGateMode::Enforced`).
    pub mode: LicenseGateMode,
    /// Verified license payload (optional).
    pub payload: Option<LicensePayload>,
    /// Signed activation record (optional).
    pub activation: Option<SignedActivation>,
    /// Managed key ring for Ed25519 key rotation/revocation (optional).
    pub keyring: Option<LicenseKeyring>,
    /// Clock port instance (defaults to `SystemClock` if `None`).
    pub clock: Option<Arc<dyn Clock>>,
    /// Persistent storage path for auto-deriving installation ID (optional).
    pub storage_path: Option<PathBuf>,
}

impl std::fmt::Debug for LicenseWiringDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LicenseWiringDeps")
            .field("mode", &self.mode)
            .field("payload", &self.payload)
            .field("activation", &self.activation)
            .field("keyring", &self.keyring)
            .field("storage_path", &self.storage_path)
            .finish()
    }
}

impl Default for LicenseWiringDeps {
    fn default() -> Self {
        Self {
            mode: LicenseGateMode::Enforced,
            payload: None,
            activation: None,
            keyring: None,
            clock: None,
            storage_path: None,
        }
    }
}

/// Wiring entry point for constructing license enforcement gates.
#[derive(Clone)]
pub struct LicenseWiring {
    gate: Arc<dyn LicenseGate>,
}

impl LicenseWiring {
    /// Builds a [`LicenseWiring`] instance from the provided dependencies contract (Rule 7).
    ///
    /// # Errors
    /// Returns [`ContextraError`] if dependency configuration is invalid or unconstructible.
    pub fn build(deps: LicenseWiringDeps) -> Result<Self, ContextraError> {
        let clock = deps.clock.unwrap_or_else(|| Arc::new(SystemClock::new()));

        let mut gate = SignedLicenseGate::no_activation_with_clock(clock)
            .with_mode(deps.mode);

        if let Some(keyring) = deps.keyring {
            gate = gate.with_keyring(keyring);
        }

        if let Some(activation) = deps.activation {
            gate = gate.with_activation(activation);
        }

        if let Some(payload) = deps.payload {
            gate = gate.with_payload(payload);
        }

        if let Some(storage_path) = deps.storage_path.as_deref() {
            gate = gate.with_auto_installation_id(Some(storage_path));
        }

        Ok(Self {
            gate: Arc::new(gate),
        })
    }

    /// Returns the constructed [`LicenseGate`] instance wrapped in an `Arc`.
    pub fn gate(&self) -> Arc<dyn LicenseGate> {
        self.gate.clone()
    }

    /// Consumes this wiring wrapper and yields the inner `Arc<dyn LicenseGate>`.
    pub fn into_gate(self) -> Arc<dyn LicenseGate> {
        self.gate
    }
}
