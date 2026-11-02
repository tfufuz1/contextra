// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: POSIX file mode setting and ownership verification (Ring 0 / SPEC §3, §4, SY-09, SY-10)
// INVARIANTEN: Zero panic; no indexing; fixed error messages without file paths in Display/Debug; fail-closed

//! # Modul: Dateiberechtigungen und Besitzverifikation (`perm`)
//!
//! Dieses Modul stellt sichere Wrapper für POSIX-Dateiberechtigungen bereit:
//! - Setzen restriktiver Dateimodi (`0o700` für Verzeichnisse, `0o600` für Dateien) via [`set_mode`].
//! - Strikte Verifikation der Eigentümerschaft und des Modus via [`verify_owner_only`].
//!
//! ## Sicherheitsanforderungen
//! - **SY-09:** Verzeichnisse `0700`, Dateien `0600`; `verify_owner_only` prüft Eigentümer und Modus.
//! - **SY-10:** Keine Panics: keine Indexierung, kein `unwrap`; alle Fehler als `SysError` mit festem Text, ohne Pfad im `Display`.

use std::fmt;
use std::path::Path;

/// Fehlerzustände bei System- und Berechtigungsoperationen.
///
/// Alle Fehlermeldungen sind statische Texte ohne Pfadangaben oder Benutzerinhalte (SY-10).
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SysError {
    /// Die angegebene Datei oder das Verzeichnis wurde nicht gefunden.
    NotFound(&'static str),
    /// Zugriff verweigert oder unzureichende Rechte (z. B. falscher Eigentümer oder zu weite Rechte).
    PermissionDenied(&'static str),
    /// Ungültiger oder nicht unterstützter Dateimodus.
    InvalidMode(&'static str),
    /// Ein allgemeiner I/O- oder Betriebssystemfehler ist aufgetreten.
    IoFailure(&'static str),
    /// Prozesshärtung ist fehlgeschlagen.
    HardeningFailed(&'static str),
}

impl fmt::Display for SysError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "{}", msg),
            Self::PermissionDenied(msg) => write!(f, "{}", msg),
            Self::InvalidMode(msg) => write!(f, "{}", msg),
            Self::IoFailure(msg) => write!(f, "{}", msg),
            Self::HardeningFailed(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for SysError {}

/// Setzt den Dateimodus für den angegebenen Pfad strikt auf `0o700` (Verzeichnis) oder `0o600` (Datei).
///
/// Andere Modi werden abgewiesen (SY-09).
///
/// # Fehler
/// Gibt [`SysError::InvalidMode`] zurück, wenn ein anderer Modus als `0o700` oder `0o600` übergeben wird.
/// Gibt [`SysError::NotFound`] oder [`SysError::IoFailure`] zurück, falls die I/O-Operation fehlschlägt.
pub fn set_mode(path: &Path, mode: u32) -> Result<(), SysError> {
    if mode != 0o700 && mode != 0o600 {
        return Err(SysError::InvalidMode(
            "invalid mode: only 0700 or 0600 permitted",
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(mode);
        std::fs::set_permissions(path, perms).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => SysError::NotFound("file or directory not found"),
            std::io::ErrorKind::PermissionDenied => {
                SysError::PermissionDenied("permission denied while setting mode")
            }
            _ => SysError::IoFailure("failed to set permissions"),
        })
    }

    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

/// Verifiziert, dass die Datei oder das Verzeichnis ausschließlich dem Eigentümer gehört und keine Gruppen- oder Fremdrechte gewährt.
///
/// # Ablauf
/// 1. Liest Metadaten des Pfades.
/// 2. Prüft, ob Gruppen- oder Others-Bits gesetzt sind (`mode & 0o077 != 0`). Wenn ja, wird [`SysError::PermissionDenied`] gemeldet.
/// 3. Ermittelt die effektive UID des Prozesses via `libc::geteuid()` und vergleicht sie mit der Datei-UID.
///
/// # Fehler
/// Meldet [`SysError::PermissionDenied`], falls Fremdrechte gesetzt sind oder der Eigentümer nicht mit dem Prozess übereinstimmt.
pub fn verify_owner_only(path: &Path) -> Result<(), SysError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta = std::fs::metadata(path).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => SysError::NotFound("file or directory not found"),
            std::io::ErrorKind::PermissionDenied => {
                SysError::PermissionDenied("permission denied while querying metadata")
            }
            _ => SysError::IoFailure("failed to read file metadata"),
        })?;

        let mode = meta.mode();
        if (mode & 0o077) != 0 {
            return Err(SysError::PermissionDenied(
                "permissions allow group or world access; expected owner-only",
            ));
        }

        let file_uid = meta.uid();
        // SAFETY: `libc::geteuid` ist ein nebenwirkungsfreier, zustandsloser POSIX-Aufruf, der die effektive User-ID zurückgibt.
        let proc_uid = unsafe { libc::geteuid() };
        if file_uid != proc_uid {
            return Err(SysError::PermissionDenied(
                "file owner does not match process owner",
            ));
        }

        Ok(())
    }

    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}
