//! Lokales Fehlermodell für contextra-cognition (SPEC §6.3, R3).
//
// FILE-CONTEXT
// ZWECK: Fehlertypen und Fehlerkonvertierung für alle Kognitions- und Konsolidierungsoperationen.
// INVARIANTEN: Zero-Panic; bitgleiche Konvertierung nach ContextraError; kein Informationsverlust.
// STAND: 2026-11-11

use thiserror::Error;

// =========================================================================
// SECTION: Fehler-Definitionen
// =========================================================================

// ANCHOR[contextra-cognition:ERR-001]
/// Lokaler Fehlertyp für Kognitions-, Konsolidierungs- und Kompaktierungsoperationen.
///
/// Garantiert strukturierte Fehlerbehandlung innerhalb des Bausteins ohne
/// direkte Abhängigkeit von Workspace-Fehlern im Kern.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CognitionError {
    /// Ungültige Eingabeparameter oder Validierungsfehler.
    #[error("Ungültige Eingabe für Kognitions-Pipeline: {0}")]
    InvalidInput(String),

    /// Verletzung von Kognitions- oder Konsolidierungsrichtlinien.
    #[error("Richtlinienverletzung bei Kognition: {0}")]
    PolicyViolation(String),

    /// Interner Verarbeitungsfehler.
    #[error("Interner Kognitionsfehler: {0}")]
    Internal(String),

    /// Speicherbudget überschritten.
    #[error("Speicherbudget für Konsolidierung überschritten: {peak_bytes} > {max_bytes} Bytes")]
    BudgetExceeded {
        /// Erwartetes Spitzenbudget in Bytes.
        peak_bytes: usize,
        /// Maximal konfiguriertes Budget in Bytes.
        max_bytes: usize,
    },

    /// Veralteter Zustand oder OCC-Konflikt.
    #[error("Veralteter Zustand oder Konflikt bei Konsolidierung: {0}")]
    StaleRead(String),

    /// Nicht gefunden.
    #[error("Ressource nicht gefunden: {0}")]
    NotFound(String),

    /// Serialisierungsfehler.
    #[error("Serialisierungsfehler bei Kognition: {0}")]
    Serialization(String),

    /// Transaktionsfehler.
    #[error("Transaktionsfehler bei Konsolidierung: {0}")]
    Transaction(String),
}
// !SECTION: Fehler-Definitionen

// =========================================================================
// SECTION: Workspace-Fehlerkonvertierung
// =========================================================================

// ANCHOR[contextra-cognition:ERR-002]
// NOTE: Verlustfreie Konvertierung von CognitionError in den Workspace-Typ ContextraError.
impl From<CognitionError> for contextra_types::ContextraError {
    fn from(err: CognitionError) -> Self {
        match err {
            CognitionError::InvalidInput(msg) => contextra_types::ContextraError::InvalidInput(msg),
            CognitionError::PolicyViolation(msg) => {
                contextra_types::ContextraError::PolicyViolation(msg)
            }
            CognitionError::Internal(msg) => contextra_types::ContextraError::Internal(msg),
            CognitionError::BudgetExceeded {
                peak_bytes,
                max_bytes,
            } => contextra_types::ContextraError::MemoryLimitExceeded(format!(
                "Speicherbudget für Konsolidierung überschritten: {peak_bytes} > {max_bytes} Bytes"
            )),
            CognitionError::StaleRead(msg) => contextra_types::ContextraError::StaleRead(msg),
            CognitionError::NotFound(msg) => contextra_types::ContextraError::NotFound(msg),
            CognitionError::Serialization(msg) => {
                contextra_types::ContextraError::Serialization(msg)
            }
            CognitionError::Transaction(msg) => contextra_types::ContextraError::Transaction(msg),
        }
    }
}
// !SECTION: Workspace-Fehlerkonvertierung
