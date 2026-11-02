// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: Fehlertypen für Systemaufrufe, Haltbarkeit und Dateisystemprobe (Ring 0 / SPEC §8.3, SY-05, SY-10, D-55)
// INVARIANTEN: Zero panic; keine Pfade oder Nutzerdaten in Display/Debug (T-LOG-1); SyncFailure nie in Ok umdeutbar

//! # Modul: Fehlerbehandlung (`error`)
//!
//! Dieses Modul definiert die kanonischen Fehlertypen für Low-Level-Systemaufrufe:
//! - [`SyncFailure`]: Spezifischer, nicht ignorierbarer Fehler bei fsync/fdatasync/flush_dir Operationen (D-55).
//! - [`UnsupportedKind`]: Spezifikation des nicht unterstützten Dateisystemtyps (D-56, T-PLAT-1).
//! - [`SysError`]: Allgemeiner Fehler für Systemprüfungen und Speicherplatzproben.

use std::fmt;

// ANCHOR[contextra-sys:ERR-SYNC-FAILURE-001]
/// Repräsentiert einen fatalen Fehler bei der physischen Dateisynchronisation (`fsync`, `fdatasync`, `flush_dir`).
///
/// Gemäß Architektur-Entscheidung D-55 darf ein [`SyncFailure`] niemals in `Ok(())` umgedeutet oder ignoriert
/// werden. Wiederholungsversuche auf demselben Dateihandle sind untersagt, da der Linux-Kernel nach einem
/// fehlgeschlagenen `fsync` Pufferseiten als sauber markieren kann („fsyncgate“).
#[derive(Debug)]
pub struct SyncFailure {
    inner: std::io::Error,
}

impl SyncFailure {
    // ANCHOR[contextra-sys:ERR-SYNC-NEW-001]
    /// Erstellt eine neue Instanz von [`SyncFailure`] aus einem [`std::io::Error`].
    #[must_use]
    pub fn new(err: std::io::Error) -> Self {
        Self { inner: err }
    }

    // ANCHOR[contextra-sys:ERR-SYNC-RAW-OS-001]
    /// Gibt den rohen Betriebssystem-Fehlercode (`errno`) zurück, falls vorhanden.
    #[must_use]
    pub fn raw_os_error(&self) -> Option<i32> {
        self.inner.raw_os_error()
    }

    // ANCHOR[contextra-sys:ERR-SYNC-KIND-001]
    /// Gibt die [`std::io::ErrorKind`]-Kategorie des Fehlers zurück.
    #[must_use]
    pub fn kind(&self) -> std::io::ErrorKind {
        self.inner.kind()
    }

    // ANCHOR[contextra-sys:ERR-SYNC-INTO-IO-001]
    /// Konvertiert den Fehler in den zugrundeliegenden [`std::io::Error`].
    #[must_use]
    pub fn into_inner(self) -> std::io::Error {
        self.inner
    }
}

impl fmt::Display for SyncFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // T-LOG-1 / SY-10: Fester Text ohne Pfade oder variable Dateisysteminhalte
        if let Some(code) = self.raw_os_error() {
            write!(
                f,
                "low-level persistence synchronization failed (os error {code})"
            )
        } else {
            write!(f, "low-level persistence synchronization failed")
        }
    }
}

impl std::error::Error for SyncFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

impl From<std::io::Error> for SyncFailure {
    fn from(err: std::io::Error) -> Self {
        Self::new(err)
    }
}

impl From<SyncFailure> for std::io::Error {
    fn from(failure: SyncFailure) -> Self {
        failure.into_inner()
    }
}

// ANCHOR[contextra-sys:ERR-UNSUPPORTED-KIND-001]
/// Klassifikation von nicht unterstützten Dateisystemtypen (D-56, FM-11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum UnsupportedKind {
    /// OverlayFS (z. B. Docker/Container-Schichten ohne direkte POSIX-Flush-Garantie).
    Overlayfs,
    /// Temporäres RAM-Dateisystem (tmpfs / shm).
    Tmpfs,
    /// Network File System (NFS).
    Nfs,
    /// Server Message Block / CIFS (SMB).
    Smb,
    /// Filesystem in Userspace (FUSE).
    Fuse,
    /// Unbekannter oder nicht verifizierter Dateisystem-Magic-Code.
    Unknown(u64),
}

impl fmt::Display for UnsupportedKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overlayfs => write!(f, "overlayfs filesystem is unsupported"),
            Self::Tmpfs => write!(f, "tmpfs filesystem is unsupported for durable storage"),
            Self::Nfs => write!(f, "nfs network filesystem is unsupported"),
            Self::Smb => write!(f, "smb/cifs network filesystem is unsupported"),
            Self::Fuse => write!(f, "fuse user-space filesystem is unsupported"),
            Self::Unknown(magic) => {
                write!(f, "unknown or unsupported filesystem magic: 0x{magic:x}")
            }
        }
    }
}

// ANCHOR[contextra-sys:ERR-SYS-ERROR-001]
/// Übergreifender Systemfehler für Crate `contextra-sys` (SY-10).
#[derive(Debug)]
#[allow(dead_code)]
pub enum SysError {
    /// Ein zugrundeliegender I/O-Fehler ist aufgetreten.
    Io(std::io::Error),
    /// Das Dateisystem wird für dauerhafte Ablage nicht unterstützt (D-56).
    Unsupported(UnsupportedKind),
}

impl fmt::Display for SysError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // T-LOG-1 / SY-10: Fester Text ohne Pfade oder variable Nutzerinhalte
        match self {
            Self::Io(err) => {
                if let Some(code) = err.raw_os_error() {
                    write!(f, "system call IO error occurred (os error {code})")
                } else {
                    write!(f, "system call IO error occurred")
                }
            }
            Self::Unsupported(kind) => write!(f, "filesystem probe rejected target: {kind}"),
        }
    }
}

impl std::error::Error for SysError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Unsupported(_) => None,
        }
    }
}

impl From<std::io::Error> for SysError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_failure_error_traits() {
        let io_err = std::io::Error::from_raw_os_error(libc::EIO);
        let sync_err: SyncFailure = io_err.into();
        assert_eq!(sync_err.raw_os_error(), Some(libc::EIO));

        let disp = format!("{sync_err}");
        assert!(!disp.contains('/'));
        assert!(disp.contains("5")); // libc::EIO is 5 on Linux

        let back_to_io: std::io::Error = sync_err.into();
        assert_eq!(back_to_io.raw_os_error(), Some(libc::EIO));
    }

    #[test]
    fn test_sys_error_traits() {
        let err = SysError::Unsupported(UnsupportedKind::Nfs);
        let disp = format!("{err}");
        assert!(!disp.contains('/'));
        assert!(disp.contains("nfs"));

        let io_err = SysError::Io(std::io::Error::from_raw_os_error(libc::EBADF));
        let io_disp = format!("{io_err}");
        assert!(!io_disp.contains('/'));
    }
}
