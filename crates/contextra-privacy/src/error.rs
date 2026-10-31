//! `error` — Zentrale Fehlertypen für das Cloud Egress Gateway und Privacy-Subsystem.
//!
//! # Zweck
//! Definiert typisierte Fehlerstrukturen für Fehlschläge bei der DLP-Prüfung (Data Loss Prevention),
//! Richtlinienverstößen, ungültigen Parametern sowie Surrogat-Token-Kollisionen.
//!
//! # Ring-Zugehörigkeit
//! Ring 3 (Domain Services). Wird modulweit und in übergeordneten Schichten zur Fehlerbehandlung genutzt.

use thiserror::Error;

// ANCHOR[contextra-privacy:ERR-001]
/// Fehlertypen des Egress-Gateways und der DLP-Prüfung.
///
/// Signalisiert spezifische Fehlerzustände bei der Analyse und Weiterleitung von Daten an externe Cloud-Dienste.
#[derive(Debug, Error)]
pub enum EgressError {
    /// Ungültige Eingabeparameter oder Konfiguration.
    #[error("Invalid parameter: {0}")]
    InvalidParams(String),
    /// Richtlinienverstoß (Blockierung durch Richtlinie).
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
    /// Interner Verarbeitungsfehler im Egress-Subsystem.
    #[error("Internal egress error: {0}")]
    InternalError(String),
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

// ANCHOR[contextra-privacy:ERR-002]
// Konvertierung von EgressVaultError in EgressError zur feingranularen Fehlerweitergabe.
// 1. InvalidPattern -> EgressError::InvalidParams
// 2. PayloadTooLarge -> EgressError::InvalidParams
// 3. TenantScopeViolation -> EgressError::PolicyViolation
// 4. SurrogateCollision -> EgressError::SurrogateCollision
impl From<crate::egress_vault::EgressVaultError> for EgressError {
    fn from(err: crate::egress_vault::EgressVaultError) -> Self {
        match err {
            crate::egress_vault::EgressVaultError::InvalidPattern { pattern, reason } => {
                Self::InvalidParams(format!("Invalid pattern '{pattern}': {reason}"))
            }
            crate::egress_vault::EgressVaultError::PayloadTooLarge { size, limit } => {
                Self::InvalidParams(format!("Payload size exceeds limit: {size} > {limit}"))
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
    ///
    /// # Parameter
    /// * `msg`: Fehlermeldung oder Beschreibung der ungültigen Parameter.
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::InvalidParams(msg.into())
    }

    // ANCHOR[contextra-privacy:ERR-004]
    /// Konstruiert einen `EgressError::PolicyViolation`.
    ///
    /// # Parameter
    /// * `msg`: Beschreibung des verletzten Richtlinien-Kriteriums.
    pub fn policy_violation(msg: impl Into<String>) -> Self {
        Self::PolicyViolation(msg.into())
    }

    // ANCHOR[contextra-privacy:ERR-005]
    /// Konstruiert einen `EgressError::InternalError`.
    ///
    /// # Parameter
    /// * `msg`: Interne System-Fehlermeldung.
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }
}
