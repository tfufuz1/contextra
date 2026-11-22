//! # Redo-Fehler und HWM-Prüfung
//!
//! Fehlerarten für das Redo-Log und HWM-Verankerung nach SPEC §9.4, §10.5, §13.1 und §18.

use std::fmt;
use std::io;

/// Fehlerarten im Redo-Subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedoError {
    /// Segmentdaten beschädigt (fail-closed, INV-LOG-6).
    Corrupt(String),
    /// Redo-Schwanz unterhalb der Hochwassermarke verloren (INV-LOG-8).
    CorruptRedoTruncated {
        /// Letzter gefundener Stamp.
        last_stamp: u64,
        /// Dauerhafte Hochwassermarke aus dem Katalog.
        redo_hwm: u64,
    },
    /// Subsystem ist vergiftet nach E/A- oder Sync-Fehler (INV-LOG-5).
    Poisoned {
        /// Ursache der Vergiftung.
        cause: String,
    },
    /// Speicherplatz erschöpft (wiederholbarer Fehler nach Abschneiden).
    BusyStorageFull,
    /// Allgemeiner E/A-Fehler.
    Io(String),
    /// Ungültiger oder nicht unterstützter Integritätsmodus.
    InvalidIntegrityMode(u8),
    /// Nicht unterstütztes Feature oder ungültiges Format.
    Unsupported(String),
    /// Fehler bei kryptografischer Operation.
    CryptoError(String),
    /// Domänenschlüssel für versiegelten Körper nicht gefunden.
    DomainKeyNotFound(u32),
}

impl fmt::Display for RedoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corrupt(msg) => write!(f, "INV-LOG-6: Redo beschädigt: {msg}"),
            Self::CorruptRedoTruncated { last_stamp, redo_hwm } => {
                write!(
                    f,
                    "INV-LOG-8: Redo-Schwanz abgeschnitten: letzter Stamp {last_stamp} < redo_hwm {redo_hwm}"
                )
            }
            Self::Poisoned { cause } => write!(f, "INV-LOG-5: Redo vergiftet: {cause}"),
            Self::BusyStorageFull => write!(f, "Redo-Speicher voll (StorageFull)"),
            Self::Io(msg) => write!(f, "Redo E/A-Fehler: {msg}"),
            Self::InvalidIntegrityMode(m) => write!(f, "Ungültiger Integritätsmodus: {m}"),
            Self::Unsupported(msg) => write!(f, "Nicht unterstützt: {msg}"),
            Self::CryptoError(msg) => write!(f, "Kryptografiefehler: {msg}"),
            Self::DomainKeyNotFound(d) => write!(f, "Domänenschlüssel für Domäne {d} fehlt"),
        }
    }
}

impl std::error::Error for RedoError {}

impl From<io::Error> for RedoError {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::StorageFull {
            Self::BusyStorageFull
        } else {
            Self::Io(e.to_string())
        }
    }
}

/// Prüft die Redo-HWM-Bedingung nach SPEC §9.4, §13.1 Schritt 6 und INV-LOG-8.
///
/// Findet die Recovery danach Redo-Segmente, deren letzter gültiger Stamp unter `redo_hwm`
/// liegt (und die Lücke nicht durch `redo_floor_stamp` erklärt ist), ist das Ergebnis
/// `Err(RedoError::CorruptRedoTruncated)`.
pub fn check_redo_hwm(
    last_valid_stamp: Option<u64>,
    redo_floor_stamp: u64,
    redo_hwm: u64,
) -> Result<(), RedoError> {
    // // INV-LOG-8: Letzter gültiger Redo-Stamp < redo_hwm (ohne redo_floor_stamp-Erklärung) => Corrupt{RedoTruncated}
    let effective_stamp = last_valid_stamp.unwrap_or(redo_floor_stamp);
    if effective_stamp < redo_hwm && redo_floor_stamp < redo_hwm {
        return Err(RedoError::CorruptRedoTruncated {
            last_stamp: effective_stamp,
            redo_hwm,
        });
    }
    Ok(())
}
