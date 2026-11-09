//! `error` — Zentrale Fehlertypen für das Cloud Egress Gateway und Privacy-Subsystem.
//!
//! # Zweck
//! Definiert typisierte Fehlerstrukturen für Fehlschläge bei der DLP-Prüfung (Data Loss Prevention),
//! Richtlinienverstößen, ungültigen Parametern sowie Surrogat-Token-Kollisionen.
//! Erfüllt B4 für Klasse-A-Bausteine (ein primärer `Error` / `EgressError`, `#[non_exhaustive]`).

use thiserror::Error;

// ANCHOR[contextra-privacy:ERR-001]
/// Fehlertypen des Egress-Gateways und der DLP-Prüfung (Klasse A).
///
/// Signalisiert spezifische Fehlerzustände bei der Analyse und Weiterleitung von Daten an externe Cloud-Dienste.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EgressError {
    /// Ungültige Eingabeparameter oder Konfiguration.
    #[error("Invalid parameter: {0}")]
    InvalidParams(String),

    /// Ungültige Eingabe (Klasse A Standardfehler).
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Kapazitäts- oder Größenlimit überschritten (Klasse A Standardfehler).
    #[error("Capacity exceeded: {0}")]
    CapacityExceeded(String),

    /// Richtlinienverstoß (Blockierung durch Richtlinie).
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Interner Verarbeitungsfehler im Egress-Subsystem.
    #[error("Internal egress error: {0}")]
    InternalError(String),

    /// I/O-Fehler (Klasse A Standardfehler).
    #[error("I/O error: {0}")]
    Io(String),

    /// Zustandskonflikt (Klasse A Standardfehler).
    #[error("State conflict: {0}")]
    StateConflict(String),

    /// Korrumpierte Daten (Klasse A Standardfehler).
    #[error("Corrupt data: {0}")]
    Corrupt(String),

    /// Kollision von Ersatz-Tokens (Surrogaten) im Vault.
    #[error("Surrogate token collision detected for token '{surrogate}': existing plaintext '{existing_plaintext}' != new plaintext '{new_plaintext}'")]
    SurrogateCollision {
        /// Ersatz-Token (Surrogate).
        surrogate: String,
        /// Bereits existierender Klartext-Wert.
        existing_plaintext: String,
        /// Neuer abweichender Klartext-Wert.
        new_plaintext: String,
    },
}

/// Primärer Crate-Fehlertyp-Alias für Klasse-A-Konformität (B4).
pub type Error = EgressError;

/// Result-Typ für contextra-privacy.
pub type Result<T> = std::result::Result<T, EgressError>;

// ANCHOR[contextra-privacy:ERR-002]
// Konvertierung von EgressVaultError in EgressError zur feingranularen Fehlerweitergabe.
// 1. InvalidPattern -> EgressError::InvalidParams
// 2. PayloadTooLarge -> EgressError::CapacityExceeded / InvalidParams
// 3. TenantScopeViolation -> EgressError::PolicyViolation
// 4. SurrogateCollision -> EgressError::SurrogateCollision
impl From<crate::egress_vault::EgressVaultError> for EgressError {
    fn from(err: crate::egress_vault::EgressVaultError) -> Self {
        match err {
            crate::egress_vault::EgressVaultError::InvalidPattern { pattern, reason } => {
                Self::InvalidParams(format!("Invalid pattern '{pattern}': {reason}"))
            }
            crate::egress_vault::EgressVaultError::PayloadTooLarge { size, limit } => {
                Self::CapacityExceeded(format!("Payload size exceeds limit: {size} > {limit}"))
            }
            crate::egress_vault::EgressVaultError::TenantScopeViolation(e) => {
                Self::PolicyViolation(e.to_string())
            }
            crate::egress_vault::EgressVaultError::SurrogateCollision {
                surrogate,
                existing_plaintext,
                new_plaintext,
            } => Self::SurrogateCollision {
                surrogate,
                existing_plaintext,
                new_plaintext,
            },
        }
    }
}

impl EgressError {
    // ANCHOR[contextra-privacy:ERR-003]
    /// Konstruiert einen `EgressError::InvalidParams`.
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::InvalidParams(msg.into())
    }

    /// Konstruiert einen `EgressError::InvalidInput`.
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Konstruiert einen `EgressError::CapacityExceeded`.
    pub fn capacity_exceeded(msg: impl Into<String>) -> Self {
        Self::CapacityExceeded(msg.into())
    }

    // ANCHOR[contextra-privacy:ERR-004]
    /// Konstruiert einen `EgressError::PolicyViolation`.
    pub fn policy_violation(msg: impl Into<String>) -> Self {
        Self::PolicyViolation(msg.into())
    }

    // ANCHOR[contextra-privacy:ERR-005]
    /// Konstruiert einen `EgressError::InternalError`.
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }

    /// Konstruiert einen `EgressError::Io`.
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }

    /// Konstruiert einen `EgressError::StateConflict`.
    pub fn state_conflict(msg: impl Into<String>) -> Self {
        Self::StateConflict(msg.into())
    }

    /// Konstruiert einen `EgressError::Corrupt`.
    pub fn corrupt(msg: impl Into<String>) -> Self {
        Self::Corrupt(msg.into())
    }
}
