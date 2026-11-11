//! `error` — Zentrale Fehlertypen für das Cloud Egress Gateway und Privacy-Subsystem.
//!
//! # Zweck
//! Definiert typisierte Fehlerstrukturen für Fehlschläge bei der DLP-Prüfung,
//! Richtlinienverstößen, ungültigen Parametern sowie Surrogat-Token-Kollisionen.
//! Erfüllt Invariante I3 (`PrivacyError`, `#[non_exhaustive]`, `thiserror`).

use thiserror::Error;

// ANCHOR[contextra-privacy:ERR-001]
/// Primärer Fehlertyp des Privacy- und Egress-Subsystems.
///
/// Signalisiert spezifische Fehlerzustände bei der Analyse, Filterung und Weiterleitung
/// von Daten an externe Dienste.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PrivacyError {
    /// Ungültige Eingabeparameter oder Konfiguration.
    #[error("Invalid parameter: {0}")]
    InvalidParams(String),

    /// Ungültige Eingabe.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Kapazitäts- oder Größenlimit überschritten.
    #[error("Capacity exceeded: {0}")]
    CapacityExceeded(String),

    /// Richtlinienverstoß (Blockierung durch Richtlinie).
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Interner Verarbeitungsfehler im Egress-Subsystem.
    #[error("Internal egress error: {0}")]
    InternalError(String),

    /// I/O-Fehler.
    #[error("I/O error: {0}")]
    Io(String),

    /// Zustandskonflikt.
    #[error("State conflict: {0}")]
    StateConflict(String),

    /// Korrumpierte Daten.
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

/// Abwärtskompatibler Alias für `PrivacyError`.
pub type EgressError = PrivacyError;

/// Primärer Crate-Fehlertyp-Alias.
pub type Error = PrivacyError;

/// Result-Typ für contextra-privacy.
pub type Result<T> = std::result::Result<T, PrivacyError>;

// ANCHOR[contextra-privacy:ERR-002]
// Konvertierung von EgressVaultError in PrivacyError zur feingranularen Fehlerweitergabe.
impl From<crate::egress_vault::EgressVaultError> for PrivacyError {
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

impl PrivacyError {
    // ANCHOR[contextra-privacy:ERR-003]
    /// Konstruiert einen `PrivacyError::InvalidParams`.
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::InvalidParams(msg.into())
    }

    /// Konstruiert einen `PrivacyError::InvalidInput`.
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Konstruiert einen `PrivacyError::CapacityExceeded`.
    pub fn capacity_exceeded(msg: impl Into<String>) -> Self {
        Self::CapacityExceeded(msg.into())
    }

    // ANCHOR[contextra-privacy:ERR-004]
    /// Konstruiert einen `PrivacyError::PolicyViolation`.
    pub fn policy_violation(msg: impl Into<String>) -> Self {
        Self::PolicyViolation(msg.into())
    }

    // ANCHOR[contextra-privacy:ERR-005]
    /// Konstruiert einen `PrivacyError::InternalError`.
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }

    /// Konstruiert einen `PrivacyError::Io`.
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }

    /// Konstruiert einen `PrivacyError::StateConflict`.
    pub fn state_conflict(msg: impl Into<String>) -> Self {
        Self::StateConflict(msg.into())
    }

    /// Konstruiert einen `PrivacyError::Corrupt`.
    pub fn corrupt(msg: impl Into<String>) -> Self {
        Self::Corrupt(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_error_display_and_conversion() {
        let cases: Vec<(PrivacyError, &str)> = vec![
            (
                PrivacyError::invalid_params("p1"),
                "Invalid parameter: p1",
            ),
            (
                PrivacyError::invalid_input("i1"),
                "Invalid input: i1",
            ),
            (
                PrivacyError::capacity_exceeded("c1"),
                "Capacity exceeded: c1",
            ),
            (
                PrivacyError::policy_violation("pv1"),
                "Policy violation: pv1",
            ),
            (
                PrivacyError::internal_error("ie1"),
                "Internal egress error: ie1",
            ),
            (PrivacyError::io("io1"), "I/O error: io1"),
            (
                PrivacyError::state_conflict("sc1"),
                "State conflict: sc1",
            ),
            (PrivacyError::corrupt("co1"), "Corrupt data: co1"),
            (
                PrivacyError::SurrogateCollision {
                    surrogate: "tok1".into(),
                    existing_plaintext: "old".into(),
                    new_plaintext: "new".into(),
                },
                "Surrogate token collision detected for token 'tok1': existing plaintext 'old' != new plaintext 'new'",
            ),
        ];

        for (err, expected_display) in cases {
            assert_eq!(err.to_string(), expected_display);
        }
    }
}
