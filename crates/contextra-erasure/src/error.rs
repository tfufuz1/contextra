//! Fehlerdefinitionen für den Löschkern.

use thiserror::Error;

/// Fehlerzustände des Löschkerns (SPEC §7.5).
#[derive(Debug, Error)]
pub enum ErasureError {
    /// E/A-Fehler bei Dateizugriffen über das Vfs.
    #[error("I/O-Fehler im Löschkern: {0}")]
    Io(String),

    /// Fehler bei Vault-Operationen (Sondieren, MAC, Kapazität).
    #[error("Vault-Fehler: {0}")]
    Vault(String),

    /// Fehler im Gate-Subsystem (Ungültige Bereiche, Restriktionen).
    #[error("Gate-Fehler: {0}")]
    Gate(String),

    /// Fehler bei der Messung (Positivkontrolle fehlgeschlagen, Residuen gefunden).
    #[error("Messfehler: {0}")]
    Measurement(String),

    /// Fehler im Ledger oder Restore-Guard.
    #[error("Ledger-Fehler: {0}")]
    Ledger(String),

    /// Unbekannter oder unerwarteter interner Fehler (fail-closed).
    #[error("Interner Fehler: {0}")]
    Internal(String),
}
