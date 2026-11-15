//! Fehlertyp von acta-history (HR-4: ein Fehlertyp je Crate, kein `Other(String)`).

use acta_base::{ClassifiedError, ErrorClass};

/// Fehler von acta-history. Platzhalter bis zur Umsetzung der Archiv-Extraktion.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HistoryError {
    /// Funktion ist noch nicht umgesetzt (nie als Erfolg zu melden, Z11).
    #[error("not implemented")]
    Unimplemented,
}

impl ClassifiedError for HistoryError {
    fn class(&self) -> ErrorClass {
        ErrorClass::Invalid
    }
    fn code(&self) -> &'static str {
        "history.unimplemented"
    }
}
