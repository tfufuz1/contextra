//! License and activation gate port trait definitions (§14.6, §15).

/// Feature-Ring gemäß Spec §15.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureRing {
    /// Fast ring (open source core).
    Fast,
    /// Sovereign ring (enterprise data sovereignty).
    Sovereign,
    /// Compliance ring (strict auditability and regulatory rules).
    Compliance,
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
}

#[cfg(test)]
mod dyn_safety {
    use super::*;

    fn _assert_dyn(_: &dyn LicenseGate) {}

    #[test]
    fn test_license_gate_dyn_safety() {
        let _ = _assert_dyn;
    }
}
