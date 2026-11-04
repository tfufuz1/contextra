//! # Modul: Fehler- und Statustypen (`error`)
//!
//! Stellt typisierte, sichere Fehler- und Ergebnistypen bereit, die niemals
//! Pfad-, Speicher- oder Nutzerdaten nach außen tragen (SY-10).

use std::fmt;

/// Fester Fehler bei Synchronisationsoperationen (fsync, fdatasync, F_FULLFSYNC, flush_dir).
/// Kann nicht irrtuemlich in `Ok` umgedeutet werden (D-55).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncFailure {
    /// Das Dateisystem oder die Operation wird auf der Zielplattform nicht unterstuetzt.
    Unsupported,
    /// Systemaufruf scheiterte mit Zugriffsfehler oder Dateideskriptor ungueltig.
    PermissionDenied,
    /// I/O-Geraetefehler oder Speicherfehler waehrend der physischen Synchronisation.
    IoError,
    /// Der Pfad oder die referenzierte Datei existiert nicht.
    NotFound,
    /// Generischer Betriebssystemfehler.
    OsError(i32),
}

impl fmt::Display for SyncFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported => write!(f, "Operation is not supported on target platform"),
            Self::PermissionDenied => write!(f, "Permission denied during sync operation"),
            Self::IoError => write!(f, "Physical I/O failure during synchronization"),
            Self::NotFound => write!(f, "Path or descriptor not found"),
            Self::OsError(code) => write!(f, "Operating system sync failure code: {code}"),
        }
    }
}

impl std::error::Error for SyncFailure {}

impl From<std::io::Error> for SyncFailure {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound,
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            std::io::ErrorKind::Unsupported => Self::Unsupported,
            _ => {
                if let Some(raw) = err.raw_os_error() {
                    Self::OsError(raw)
                } else {
                    Self::IoError
                }
            }
        }
    }
}

impl From<SyncFailure> for std::io::Error {
    fn from(failure: SyncFailure) -> Self {
        match failure {
            SyncFailure::Unsupported => {
                Self::new(std::io::ErrorKind::Unsupported, failure.to_string())
            }
            SyncFailure::PermissionDenied => {
                Self::new(std::io::ErrorKind::PermissionDenied, failure.to_string())
            }
            SyncFailure::NotFound => Self::new(std::io::ErrorKind::NotFound, failure.to_string()),
            SyncFailure::IoError => Self::other(failure.to_string()),
            SyncFailure::OsError(code) => Self::from_raw_os_error(code),
        }
    }
}

/// Nicht unterstuetzte Dateisystemtypen fuer `FsKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnsupportedKind {
    /// Dateisystemtyp ist dem System unbekannt oder nicht auditierbar.
    Unknown,
    /// Netzlaufwerk (NFS, SMB/CIFS etc.).
    Network,
    /// User-Space-Dateisystem (FUSE).
    Fuse,
    /// Virtuelles In-Memory-Dateisystem (tmpfs, ramfs).
    RamFs,
    /// Nicht unterstuetzte Betriebssystemplattform.
    PlatformUnsupported,
}

impl fmt::Display for UnsupportedKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown filesystem"),
            Self::Network => write!(f, "Network filesystem unsupported"),
            Self::Fuse => write!(f, "FUSE filesystem unsupported"),
            Self::RamFs => write!(f, "RAM/temporary filesystem unsupported"),
            Self::PlatformUnsupported => write!(f, "Target platform unsupported"),
        }
    }
}

/// Allgemeine Systemfehler fuer Low-Level-Operationen in `contextra-sys`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysError {
    /// Unzureichende Berechtigungen.
    PermissionDenied,
    /// Ziel nicht gefunden.
    NotFound,
    /// Nicht unterstuetzte Operation oder Plattform.
    Unsupported,
    /// Prozesshaertung unvollstaendig bei Policy `Required`.
    HardeningIncomplete,
    /// Fehlerhafter oder ungueltiger Parameter.
    InvalidInput,
    /// Interner Betriebssystemfehler (z. B. errno).
    OsError(i32),
}

impl fmt::Display for SysError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PermissionDenied => write!(f, "Operation permission denied"),
            Self::NotFound => write!(f, "Target resource not found"),
            Self::Unsupported => write!(f, "Platform or filesystem unsupported"),
            Self::HardeningIncomplete => {
                write!(f, "Process hardening incomplete for required policy")
            }
            Self::InvalidInput => write!(f, "Invalid parameter provided"),
            Self::OsError(code) => write!(f, "Operating system error code: {code}"),
        }
    }
}

impl std::error::Error for SysError {}

impl From<std::io::Error> for SysError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound,
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            std::io::ErrorKind::Unsupported => Self::Unsupported,
            std::io::ErrorKind::InvalidInput => Self::InvalidInput,
            _ => {
                if let Some(code) = err.raw_os_error() {
                    Self::OsError(code)
                } else {
                    Self::Unsupported
                }
            }
        }
    }
}

impl From<SysError> for std::io::Error {
    fn from(err: SysError) -> Self {
        match err {
            SysError::PermissionDenied => {
                Self::new(std::io::ErrorKind::PermissionDenied, err.to_string())
            }
            SysError::NotFound => Self::new(std::io::ErrorKind::NotFound, err.to_string()),
            SysError::Unsupported => Self::new(std::io::ErrorKind::Unsupported, err.to_string()),
            SysError::HardeningIncomplete => Self::other(err.to_string()),
            SysError::InvalidInput => Self::new(std::io::ErrorKind::InvalidInput, err.to_string()),
            SysError::OsError(code) => Self::from_raw_os_error(code),
        }
    }
}
