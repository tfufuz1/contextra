#![forbid(unsafe_code)]

//! `contextra-license`
//!
//! License and activation gate module for Contextra feature rings (§14.6, §15).

pub use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};

#[deprecated(
    note = "OpenFastGate is deprecated; use LicenseWiring or SignedLicenseGate with appropriate LicenseGateMode instead"
)]
pub use contextra_ports::license::OpenFastGate;

/// Enforcement mode for license gate verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LicenseGateMode {
    /// Enforce strict cryptographic license checking across all rings.
    #[default]
    Enforced,
    /// Permissive development mode allowing Fast ring bypass without activation.
    /// Only constructible in test/debug builds or when the `dev-open-gate` Cargo feature is enabled.
    #[cfg(any(test, debug_assertions, feature = "dev-open-gate"))]
    DevelopmentOpen,
}

impl LicenseGateMode {
    /// Constructs `LicenseGateMode::Enforced`.
    pub fn enforced() -> Self {
        Self::Enforced
    }

    /// Constructs `LicenseGateMode::DevelopmentOpen` when permitted by build configuration.
    #[cfg(any(test, debug_assertions, feature = "dev-open-gate"))]
    pub fn development_open() -> Self {
        Self::DevelopmentOpen
    }
}

pub mod signed_gate;
pub use signed_gate::{
    derive_local_installation_id_hash, LicenseKeyEntry, LicenseKeyring, LicensePayload,
    SignedActivation, SignedLicenseGate,
};

pub mod wiring;
pub use wiring::{LicenseWiring, LicenseWiringDeps};
