// =========================================================================
// ANCHOR[id=sys-error-main,epic=sys-error,seq=0] Crate-level structured error types
// =========================================================================
// LINK crates/contextra-sys/src/lib.rs
// NOTE: Crate-level structured error types for contextra-sys (Ring 0 / E0 / docs/SPEC.md v3 §3.1 / R3).
// Invariante: Zero panic; structured error propagation; no string-parsed errnos; thiserror integration.
// Keine unzulässigen Garantieversprechen (SPEC §6.4).

//! # Modul: Fehlerbehandlung (`error`)
//!
//! Zentrales Fehlermodell für systemnahe POSIX-, Windows- und Speicherschutz-Operationen.
//! Alle Fehler werden strukturiert typisiert und transportieren rohe Betriebssystem-Fehlercodes
//! (`errno` als `i32`), um maschinenlesbare Diagnosen und Fail-Closed-Entscheidungen ohne
//! String-Parsing zu ermöglichen.

use thiserror::Error;

// SECTION Error Enum
// ANCHOR[id=sys-error-enum,epic=sys-error,seq=1] Error Definition

/// Typisierter Systemfehler für Low-Level-OS-Operationen.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Ein Speicherpuffer konnte nicht im physischen RAM verankert werden (`mlock`/`VirtualLock`).
    #[error("memory locking failed (errno {errno}): {reason}")]
    MemoryLockFailed {
        /// Betriebssystemspezifischer Fehlercode (`errno` bzw. Win32 `GetLastError`).
        errno: i32,
        /// Technische Ursachenbeschreibung.
        reason: &'static str,
    },

    /// Eine physische Synchronisation (`fsync`/`fdatasync`/`FlushFileBuffers`) ist fehlgeschlagen.
    /// Nach diesem Fehler ist das betroffene Handle dauerhaft vergiftet (INV-SYS-FSYNC-1).
    #[error("durability sync failed (errno {0})")]
    SyncFailed(i32),

    /// Der Dateizugriff oder die Rechteprüfung wurde vom Betriebssystem verweigert (`EPERM`/`EACCES`).
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Eine ungültige Eingabe wurde übergeben (z. B. Nullbyte im Dateipfad oder Bereichsüberschreitung).
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Eine Standard-E/A-Operation ist fehlgeschlagen.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Die angeforderte Operation wird auf der Zielplattform nicht unterstützt.
    #[error("operation unsupported on this platform: {0}")]
    Unsupported(&'static str),

    /// Ein Datei-Handle befindet sich in einem dauerhaft vergifteten Zustand nach vorherigem Flush-Fehler (INV-SYS-FSYNC-1).
    #[error("file handle is permanently poisoned due to previous sync failure")]
    HandlePoisoned,
}
// !SECTION Error Enum

// SECTION Type Aliases & Standard Conversions
// ANCHOR[id=sys-error-aliases,epic=sys-error,seq=2] Typ-Aliase und Konvertierungen

/// Typ-Alias für das Crate-Fehlermodell gemäß Baukasten-Spezifikation (docs/SPEC.md §3.1 / R3).
pub type SysError = Error;

/// Spezifischer Result-Typ für `contextra-sys`.
pub type Result<T> = std::result::Result<T, Error>;

// ANCHOR[id=sys-error-from-io,epic=sys-error,seq=3] Konvertierung in std::io::Error
// NOTE: Ermöglicht nahtlose Interoperabilität mit der Rust-Standardbibliothek (std::io).
impl From<Error> for std::io::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Io(io_err) => io_err,
            Error::SyncFailed(code) => std::io::Error::from_raw_os_error(code),
            Error::MemoryLockFailed { errno, reason } => {
                if errno != 0 {
                    std::io::Error::from_raw_os_error(errno)
                } else {
                    std::io::Error::other(reason)
                }
            }
            Error::PermissionDenied(msg) => {
                std::io::Error::new(std::io::ErrorKind::PermissionDenied, msg)
            }
            Error::InvalidInput(msg) => std::io::Error::new(std::io::ErrorKind::InvalidInput, msg),
            Error::Unsupported(msg) => std::io::Error::new(std::io::ErrorKind::Unsupported, msg),
            Error::HandlePoisoned => std::io::Error::other(
                "file handle is permanently poisoned due to previous sync failure",
            ),
        }
    }
}
// !SECTION Type Aliases & Standard Conversions
