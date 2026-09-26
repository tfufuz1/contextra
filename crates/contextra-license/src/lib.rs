#![forbid(unsafe_code)]

//! `contextra-license`
//!
//! License and activation gate module for Contextra feature rings (§14.6, §15).

/// Feature-Ring gemäß Spec §15.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureRing {
    Fast,
    Sovereign,
    Compliance,
}

/// Fehlerklasse für Lizenz-/Aktivierungsprüfung — crate-lokal, an der Grenze in
/// `ContextraError` konvertierbar (P6-konform, siehe Spec §17).
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum LicenseError {
    #[error("no valid activation found for feature ring {0:?}")]
    NotActivated(FeatureRing),
    #[error("activation signature verification failed")]
    InvalidSignature,
    #[error("activation expired at {0}")]
    Expired(i64),
}

/// Dyn-kompatibler Port (P27) für Lizenzprüfung — Implementierung (Lizenzserver-Anbindung,
/// Ed25519-Signaturprüfung analog zum Löschbeweis-Pfad §14.1) ist NICHT Teil dieses Tickets
/// und folgt in einem eigenen, separat zu beauftragenden Schritt.
pub trait LicenseGate: Send + Sync {
    fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError>;
}

/// Offener Default: alles unterhalb `Fast` ist immer erlaubt (Open-Source-Ring bleibt frei).
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
