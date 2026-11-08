// STAND: 2026-11-08T00:00:00Z
// ZWECK: Crate-level structured error types for contextra-sys (Ring 0 / SPEC §10.1 / B4)
// INVARIANTEN: Zero panic; structured error propagation; no string-parsed errnos; thiserror integration

//! # Modul: Fehlerbehandlung (`error`)
//!
//! Zentrales Fehlermodell für systemnahe POSIX-, Windows- und Speicherschutz-Operationen.
//! Alle Fehler werden strukturiert typisiert und transportieren rohe Betriebssystem-Fehlercodes
//! (`errno` als `i32`), um maschinenlesbare Diagnosen und Fail-Closed-Entscheidungen ohne
//! String-Parsing zu ermöglichen.

use thiserror::Error;

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
    /// Nach diesem Fehler ist das betroffene Handle unwiderruflich vergiftet (INV-SYS-FSYNC-1).
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

    /// Ein Datei-Handle befindet sich in einem vergifteten Zustand nach vorherigem Flush-Fehler (INV-SYS-FSYNC-1).
    #[error("file handle is permanently poisoned due to previous sync failure")]
    HandlePoisoned,
}

/// Spezifischer Result-Typ für `contextra-sys`.
pub type Result<T> = std::result::Result<T, Error>;

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
