// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: Sichere Dateiberechtigungen (0o700 / 0o600) und Eigentuemer-Verifikation (Ring 1 / SY-09)
// INVARIANTEN: Zero-Panic; keine Pfade in Display

//! # Modul: Dateiberechtigungen (`perm`)
//!
//! Stellt sichere Berechtigungs- und Eigentumsprüfungen fuer POSIX-Systeme bereit (0700/0600).

use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::error::SysError;

/// Setzt die Berechtigungsmaske einer Datei oder eines Verzeichnisses (z. B. `0o700` oder `0o600`).
///
/// Gemäß SY-09 sind ausschließlich `0o700` (Verzeichnisse) und `0o600` (Dateien) zulässig.
pub fn set_mode(path: &Path, mode: u32) -> Result<(), SysError> {
    if mode != 0o700 && mode != 0o600 {
        return Err(SysError::InvalidInput);
    }

    #[cfg(unix)]
    {
        std::fs::set_permissions(path, Permissions::from_mode(mode)).map_err(SysError::from)
    }

    #[cfg(not(unix))]
    {
        let _ = path;
        Err(SysError::Unsupported)
    }
}

/// Prueft, dass die Datei oder das Verzeichnis ausschliesslich dem aktuellen Prozess-Besitzer
/// zugaenglich ist und keine unberechtigten Lese-/Schreib-/Ausfuehrungsrechte fuer Gruppe/Andere gesetzt sind.
///
/// Unter POSIX:
/// - Prueft `st_uid == geteuid()`
/// - Prueft, dass Gruppe und Andere keinerlei Rechte besitzen (`mode & 0o077 == 0`).
pub fn verify_owner_only(path: &Path) -> Result<(), SysError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta = std::fs::metadata(path).map_err(SysError::from)?;
        let mode = meta.mode();

        // Keine Rechte fuer Group oder Others (letzte 6 Bits)
        if (mode & 0o077) != 0 {
            return Err(SysError::PermissionDenied);
        }

        // SAFETY: geteuid() ist ein sicherer POSIX-Syscall ohne Parameter.
        let current_uid = unsafe { libc::geteuid() };
        if meta.uid() != current_uid {
            return Err(SysError::PermissionDenied);
        }

        Ok(())
    }

    #[cfg(not(unix))]
    {
        let _ = path;
        Err(SysError::Unsupported)
    }
}

// ============================================================================
// Kompatibilitaets-Wrapper fuer bestehende Konsumenten (z. B. contextra-durable-fs / contextra-store)
// ============================================================================

/// Abwaertskompatibler Wrapper: Setzt restriktive Rechte (0o600 fuer Dateien, 0o700 fuer Verzeichnisse).
pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> {
    let is_dir = std::fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false);
    let mode = if is_dir { 0o700 } else { 0o600 };
    set_mode(path, mode).map_err(Into::into)
}

/// Abwaertskompatibler Wrapper: Verifiziert Owner-Only-Rechte.
pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> {
    verify_owner_only(path).map_err(Into::into)
}
