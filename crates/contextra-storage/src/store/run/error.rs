//! # Fehlerarten für das Run-Modul
//!
//! Alle Fehler im Lese- und Schreibpfad von SSTable-Runs.
//! Keine Integer-Casts (M-02 = 0).

use std::fmt;
use std::io;

/// Fehler beim Verarbeiten eines Runs.
#[derive(Debug)]
pub enum RunError {
    /// E/A-Fehler.
    Io(io::Error),
    /// Formatbeschädigung oder ungültige Prüfsumme/Signatur.
    Corrupt(String),
    /// Überschreitung einer Systemgrenze (INV-FMT-2, SPEC §6.7).
    Limit(String),
    /// Unbekanntes Pflicht-Feature oder nicht unterstützte Version.
    Unsupported(String),
    /// Körper wurde vernichtet (blob_no = 0).
    Shredded {
        /// Domänenkennung des vernichteten Körpers.
        domain: u32,
    },
    /// Unerwartetes Dateiende.
    UnexpectedEof,
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "E/A-Fehler im Run: {e}"),
            Self::Corrupt(msg) => write!(f, "Korrupter Run: {msg}"),
            Self::Limit(msg) => write!(f, "Limit überschritten im Run: {msg}"),
            Self::Unsupported(msg) => write!(f, "Nicht unterstütztes Format im Run: {msg}"),
            Self::Shredded { domain } => write!(f, "Körper vernichtet für Domäne {domain}"),
            Self::UnexpectedEof => write!(f, "Unerwartetes Dateiende im Run"),
        }
    }
}

impl std::error::Error for RunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for RunError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
