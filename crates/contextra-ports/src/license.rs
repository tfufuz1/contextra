//! License and activation gate port trait definitions (§14.6, §15).

/// Vector deletion mode for vector index maintenance (§15).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VectorDeleteMode {
    /// Synchronous graph repair during vector deletion.
    SynchronousRepair,
    /// Asynchronous background graph repair during vector deletion.
    BackgroundRepair,
}

impl Default for VectorDeleteMode {
    fn default() -> Self {
        Self::BackgroundRepair
    }
}

/// Feature-Ring gemäß Spec §15.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FeatureRing {
    /// Fast ring (open source core).
    Fast,
    /// Sovereign ring (enterprise data sovereignty).
    Sovereign,
    /// Compliance ring (strict auditability and regulatory rules).
    Compliance,
}

/// Token representing an authorized feature ring granting access to engine features (§15).
///
/// `AuthorizedRing` cannot be directly instantiated outside of the `contextra-ports` crate or
/// without passing through a [`LicenseGate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorizedRing {
    ring: FeatureRing,
}

impl AuthorizedRing {
    /// Creates a new [`AuthorizedRing`] instance for crate-internal authorization.
    pub(crate) fn new(ring: FeatureRing) -> Self {
        Self { ring }
    }

    /// Returns the authorized [`FeatureRing`].
    pub fn ring(&self) -> FeatureRing {
        self.ring
    }
}

/// Fehlerklasse für Lizenz-/Aktivierungsprüfung — crate-lokal, an der Grenze in
/// `ContextraError` konvertierbar (P6-konform, siehe Spec §17).
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum LicenseError {
    /// No valid activation found for the requested feature ring.
    #[error("no valid activation found for feature ring {0:?}")]
    NotActivated(FeatureRing),
    /// Activation signature verification failed.
    #[error("activation signature verification failed")]
    InvalidSignature,
    /// Activation expired at timestamp.
    #[error("activation expired at {0}")]
    Expired(i64),
}

/// Dyn-kompatibler Port (P27) für Lizenzprüfung — Implementierung (Lizenzserver-Anbindung,
/// Ed25519-Signaturprüfung analog zum Löschbeweis-Pfad §14.1) ist NICHT Teil dieses Tickets
/// und folgt in einem eigenen, separat zu beauftragenden Schritt.
pub trait LicenseGate: Send + Sync {
    /// Checks if the requested feature ring is activated and accessible.
    fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError>;

    /// Authorizes the requested feature ring and yields an [`AuthorizedRing`] token on success.
    fn authorize(&self, requested: FeatureRing) -> Result<AuthorizedRing, LicenseError> {
        self.check_ring(requested)?;
        Ok(AuthorizedRing::new(requested))
    }
}

/// Default open activation gate allowing `Fast` ring features without restriction.
#[derive(Debug, Default, Clone, Copy)]
pub struct OpenFastGate;

impl LicenseGate for OpenFastGate {
    fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError> {
        match ring {
            FeatureRing::Fast => Ok(()),
            other => Err(LicenseError::NotActivated(other)),
        }
    }
}

#[cfg(test)]
mod dyn_safety {
    use super::*;

    fn _assert_dyn(_: &dyn LicenseGate) {}

    #[test]
    fn test_license_gate_dyn_safety() {
        let _ = _assert_dyn;
    }

    /// Doc test / test verifying that AuthorizedRing cannot be constructed externally.
    /// ```compile_fail
    /// use contextra_ports::license::{AuthorizedRing, FeatureRing};
    /// let _ = AuthorizedRing { ring: FeatureRing::Fast };
    /// ```
    fn _doctest_authorized_ring_private_fields() {}
}
