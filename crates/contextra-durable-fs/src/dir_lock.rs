//! # Exklusiver Verzeichnis-Lock (`dir_lock`)
//!
//! Zweck: Implementierung eines exklusiven Verzeichnis-Locks auf der Datei `LOCK`
//! mit `flock`-Semantik und Dateisystem-Validierung (DF-05, T-LOCK-1).
//!
//! Ring-Zugehörigkeit: Ring 1 (Dateisystem-Durability).
//! Wichtigste Typen:
//! - [`DirLock`]: Exklusiver Sperr-Guard für das Datenverzeichnis.
//! - [`LockMode`]: Konfiguration des Sperrmodus (`Normal` vs. `MemoryOnly`).
//!
//! Invarianten:
//! - `INV-LOCK-1`: Ein Datenverzeichnis darf zu jedem Zeitpunkt nur von genau einem Prozess geöffnet sein (D-50).
//! - `INV-LOCK-2`: Netzwerk- und FUSE-Dateisysteme werden strikt abgewiesen (`ErrorKind::Unsupported`).
//! - `INV-LOCK-3`: Ein zweiter Prozess scheitert deterministisch mit dem festen Fehlertext
//!   `"Datenverzeichnis bereits in Benutzung"` (DF-05).
//! - `INV-PERM-1`: Die `LOCK`-Datei wird bei Anlage mit restriktiven Rechten `0600` geschützt (DF-08).

use contextra_sys::{fs_class, set_restrictive_file_acl, FsClass};
use std::fs::{File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};

/// Fester Fehlertext bei Verzeichnis-Sperrkonflikten gemäß DF-05 und SPEC D-50.
pub const DIR_LOCKED_MSG: &str = "Datenverzeichnis bereits in Benutzung";

/// Sperrmodus für Verzeichnissperren.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LockMode {
    /// Standard-Sperre: Erfordert ein unterstütztes lokales Dateisystem.
    #[default]
    Normal,
    /// Reiner In-Memory-Modus: Umgeht Dateisystemprüfungen.
    MemoryOnly,
}

// ANCHOR[contextra-durable-fs:DIR-LOCK-STRUCT-001]
/// RAII-Guard für eine exklusive Verzeichnissperre auf der Datei `LOCK` im Datenverzeichnis.
///
/// Hält einen exklusiven System-Lock ([`flock`]-Semantik über [`File::try_lock`]).
/// Gibt den Lock beim Verlassen des Gültigkeitsbereichs (`Drop`) sicher frei.
#[derive(Debug)]
pub struct DirLock {
    lock_path: PathBuf,
    file: Option<File>,
    mode: LockMode,
}

impl DirLock {
    // ANCHOR[contextra-durable-fs:DIR-LOCK-ACQUIRE-001]
    /// Erwirbt einen exklusiven Lock auf `dir/LOCK` im Standardmodus [`LockMode::Normal`] (DF-05, T-LOCK-1).
    pub fn acquire(dir: &Path) -> std::io::Result<Self> {
        Self::acquire_with_mode(dir, LockMode::Normal)
    }

    /// Erwirbt einen exklusiven Lock auf `dir/LOCK` im angegebenen [`LockMode`].
    ///
    /// Ablauf:
    /// 1. Dateisystem-Klasse via [`fs_class`] ermitteln.
    /// 2. Netzwerklaufwerke (`FsClass::Network`), FUSE (`FsClass::Fuse`) und Cloud-Sync-Ordner
    ///    strikt mit [`std::io::ErrorKind::Unsupported`] ablehnen.
    /// 3. Lock-Datei `LOCK` im Zielordner im Modus `0600` anlegen bzw. öffnen.
    /// 4. Exklusiven, nicht-blockierenden Lock via [`File::try_lock`] anfordern.
    /// 5. Scheitert `try_lock` mit [`TryLockError::WouldBlock`], schlägt der Aufruf fail-closed
    ///    mit festem Text [`DIR_LOCKED_MSG`] fehl.
    pub fn acquire_with_mode(dir: &Path, mode: LockMode) -> std::io::Result<Self> {
        let class = fs_class(dir)?;

        if mode == LockMode::Normal && !class.is_supported_local() {
            let class_str = match class {
                FsClass::Network => "network",
                FsClass::Fuse => "fuse",
                FsClass::CloudSync => "cloud_sync",
                _ => "unsupported_filesystem",
            };
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("directory locking rejected for filesystem class: {class_str}"),
            ));
        }

        if mode == LockMode::MemoryOnly {
            return Ok(Self {
                lock_path: dir.join("LOCK"),
                file: None,
                mode,
            });
        }

        let lock_path = dir.join("LOCK");

        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_access(7);
        }

        let file = options.open(&lock_path)?;
        set_restrictive_file_acl(&lock_path)?;

        match file.try_lock() {
            Ok(()) => Ok(Self {
                lock_path,
                file: Some(file),
                mode,
            }),
            Err(TryLockError::WouldBlock) => Err(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                DIR_LOCKED_MSG,
            )),
            Err(TryLockError::Error(err)) => Err(err),
        }
    }

    // ANCHOR[contextra-durable-fs:DIR-LOCK-PATH-001]
    /// Liefert den Pfad zur `LOCK`-Datei.
    #[must_use]
    pub fn lock_path(&self) -> &Path {
        &self.lock_path
    }

    /// Liefert den aktiven [`LockMode`].
    #[must_use]
    pub fn mode(&self) -> LockMode {
        self.mode
    }

    // ANCHOR[contextra-durable-fs:DIR-LOCK-UNLOCK-001]
    /// Gibt die Verzeichnissperre explizit vorzeitig frei.
    pub fn unlock(mut self) {
        if let Some(file) = self.file.take() {
            let _ = file.unlock();
        }
    }
}

// ANCHOR[contextra-durable-fs:DIR-LOCK-DROP-001]
// Ablauf von `Drop for DirLock`:
// 1. Lock-Handle entnehmen.
// 2. flock-Sperre freigeben via `file.unlock()`.
// 3. LOCK-Datei bleibt auf Festplatte, um Inode-Wettläufe zwischen Prozessen zu verhindern.
impl Drop for DirLock {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            let _ = file.unlock();
            // Die LOCK-Datei verbleibt auf der Festplatte, um Inode-Wettläufe zwischen
            // nebenläufigen Prozessen zu verhindern.
        }
    }
}
