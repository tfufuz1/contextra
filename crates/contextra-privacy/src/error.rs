//! `error` — Zentrale Fehlertypen für das Cloud Egress Gateway und Privacy-Subsystem.

use thiserror::Error;

/// Fehlertypen des Egress-Gateways und der DLP-Prüfung.
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
    /// Konstruiert einen `EgressError::InvalidParams`.
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::InvalidParams(msg.into())
    }

    /// Konstruiert einen `EgressError::PolicyViolation`.
    pub fn policy_violation(msg: impl Into<String>) -> Self {
        Self::PolicyViolation(msg.into())
    }

    /// Konstruiert einen `EgressError::InternalError`.
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }
}
