//! Fehlertyp von acta-reports (HR-4: ein Fehlertyp je Crate, kein `Other(String)`).

use acta_base::{ClassifiedError, ErrorClass};

/// Fehler von acta-reports. Platzhalter bis zur Umsetzung der Archiv-Extraktion.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReportsError {
    /// Funktion ist noch nicht umgesetzt (nie als Erfolg zu melden, Z11).
    #[error("not implemented")]
    Unimplemented,
}

impl ClassifiedError for ReportsError {
    fn class(&self) -> ErrorClass {
        ErrorClass::Invalid
    }
    fn code(&self) -> &'static str {
        "reports.unimplemented"
    }
}
