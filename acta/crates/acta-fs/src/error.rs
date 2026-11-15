//! Fehlertyp von acta-fs (HR-4: ein Fehlertyp je Crate, `#[non_exhaustive]`).
//!
//! Andere Fehler melden die Ports als `std::io::Error` (Spec 3.7). Nur das
//! Fsync-Versagen braucht eine eigene Klasse, weil es die Datei endgültig entwertet.

use acta_base::{ClassifiedError, ErrorClass};

/// `fsync` der Datei ist fehlgeschlagen.
///
/// Nach diesem Fehler wird dieselbe Datei nie wieder als stabil angenommen (Spec 3.7,
/// WP-7). Der Aufrufer schreibt den Inhalt aus Journal oder Speicher in eine neue
/// Datei. Scheitert auch das, wird der betroffene Bereich vergiftet (Spec 15.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("fsync failed; the file is no longer treated as stable")]
#[non_exhaustive]
pub struct FsyncFailed;

impl ClassifiedError for FsyncFailed {
    fn class(&self) -> ErrorClass {
        ErrorClass::Integrity
    }

    fn code(&self) -> &'static str {
        "fs.fsync_failed"
    }
}
