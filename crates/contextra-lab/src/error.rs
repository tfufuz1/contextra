//! Fehlertypen für `contextra-lab`.

use std::fmt;

/// Fehler, die beim Parsen von Byte-Formaten oder bei der Verifikation auftreten.
#[derive(Debug, PartialEq, Eq)]
pub enum LabError {
    /// Die Eingabe ist kürzer als für das Format erforderlich.
    UnexpectedEof {
        /// Erwartete Mindestgröße.
        expected: usize,
        /// Tatsächlich vorhandene Größe.
        actual: usize,
    },
    /// Ungültige Magic-Bytes im Dateikopf oder Footer.
    InvalidMagic,
    /// Nicht unterstützte Formatversion.
    UnsupportedVersion(u16),
    /// Unbekannte oder unzulässige Dateikategorie (kind).
    UnsupportedKind(u8),
    /// Unbekannte Algorithmen-Identifikatoren.
    UnsupportedAlgIds([u8; 4]),
    /// Krypto-Verifikation (AEAD-Tag, MAC, Checksumme, Signatur) fehlgeschlagen.
    VerificationFailed(&'static str),
    /// Format-spezifische Korruption oder Inkonsistenz.
    Corrupted(&'static str),
    /// Positivkontrolle der Messung fehlgeschlagen.
    PositiveControlFailed,
    /// Unzulässige Zustandsübergänge des Scanners.
    InvalidState(&'static str),
}

impl fmt::Display for LabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof { expected, actual } => {
                write!(
                    f,
                    "Unerwartetes Dateiende: erwartet {expected}, erhalten {actual}"
                )
            }
            Self::InvalidMagic => write!(f, "Ungültige Magic-Bytes"),
            Self::UnsupportedVersion(v) => write!(f, "Nicht unterstützte Formatversion: {v}"),
            Self::UnsupportedKind(k) => write!(f, "Nicht unterstützte Dateikategorie: {k}"),
            Self::UnsupportedAlgIds(algs) => {
                write!(f, "Nicht unterstützte Algorithmen-IDs: {algs:?}")
            }
            Self::VerificationFailed(msg) => write!(f, "Verifikation fehlgeschlagen: {msg}"),
            Self::Corrupted(msg) => write!(f, "Formatkorruption: {msg}"),
            Self::PositiveControlFailed => write!(f, "Positivkontrolle fehlgeschlagen"),
            Self::InvalidState(msg) => write!(f, "Ungültiger Typzustand: {msg}"),
        }
    }
}

impl std::error::Error for LabError {}
