//! Fehlerdefinitionen für das Erasure Service Provider Interface (SPI).
//!
//! Alle Fehlertypen sind strikt typisiert, neutral formuliert und enthalten
//! keinerlei Bezug zu Rechtsnormen oder Gesetzesbegrifflichkeiten (SPEC 2.6).

#![forbid(unsafe_code)]

use thiserror::Error;

/// Zentraler Fehlertyp für SPI-Operationen (Bereinigung und Messung).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SpiError {
    /// Die angeforderte Storage-Schicht ist nicht verfügbar oder deaktiviert.
    #[error("Schicht nicht verfügbar: {0}")]
    LayerUnavailable(String),

    /// Positivkontrolle (Canary / Control Match) vor oder während des Scans fehlgeschlagen.
    #[error("Positivkontrolle fehlgeschlagen: {0}")]
    ControlMatchFailed(String),

    /// Invariantenverletzung bei der Erzeugung des Messnachweises (INV-WITNESS-MINT-1).
    #[error("Invariantenverletzung bei der Nachweiserzeugung: {0}")]
    WitnessMintViolation(String),

    /// Ungültiges oder unvollständiges Scan-Ziel (ScanTarget).
    #[error("Ungültiges Scan-Ziel: {0}")]
    InvalidTarget(String),

    /// Physischer Bereinigungsfehler während des Purge-Vorgangs.
    #[error("Bereinigungsfehler in Schicht {layer}: {reason}")]
    PurgeFailed {
        /// Betroffene Schicht.
        layer: String,
        /// Technische Fehlerursache.
        reason: String,
    },

    /// Interner oder I/O-bedingter Ausführungsfehler.
    #[error("Interner SPI-Fehler: {0}")]
    Internal(String),
}

/// Spezieller Fehlertyp für Purger-Operationen.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PurgeError {
    /// Ziel existiert nicht oder ist bereits bereinigt.
    #[error("Bereinigungsziel nicht gefunden: {0}")]
    NotFound(String),

    /// Fehler beim physischen Überschreiben oder Entfernen.
    #[error("Physisches Bereinigen fehlgeschlagen: {0}")]
    ExecutionFailed(String),

    /// Die Schicht verweigert den Zugriff im aktuellen Zustand.
    #[error("Schicht verweigert Bereinigung: {0}")]
    Refused(String),

    /// Interner Fehler.
    #[error("Interner Bereinigungsfehler: {0}")]
    Internal(String),
}

/// Spezieller Fehlertyp für Scanner-Operationen.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ScanError {
    /// Positivkontrolle vor Messbeginn nicht bestanden.
    #[error("Positivkontrolle verfehlt: {0}")]
    PositiveControlMissing(String),

    /// Fehler beim Durchmustern der Einheiten.
    #[error("Scan-Ausführung fehlgeschlagen: {0}")]
    InspectionFailed(String),

    /// Schicht deaktiviert oder nicht unterstützt.
    #[error("Schicht-Scan nicht unterstützt: {0}")]
    Unsupported(String),

    /// Invariante verletzt (z. B. scanned == 0 für aktive Techniken).
    #[error("Scan-Invariante verletzt: {0}")]
    InvariantViolation(String),

    /// Interner Fehler.
    #[error("Interner Scan-Fehler: {0}")]
    Internal(String),
}

/// Typalias für Resultate von SPI-Aufrufen.
pub type SpiResult<T> = std::result::Result<T, SpiError>;
