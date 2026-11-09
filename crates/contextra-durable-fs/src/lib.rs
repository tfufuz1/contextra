#![allow(clippy::disallowed_types, clippy::disallowed_methods)]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

//! # Dauerhafte Dateisystem-Operationen und synchrone Disk-Flushes (`contextra-durable-fs`)
//!
//! Autarker Baustein der Klasse A (B1–B12) für crash-sichere Dateioperationen,
//! synchrones fsync-Poisoning (`INV-FSYNC-1`) und atomare Dateieretzungen (`INV-VFS-ATOMIC-1`).
//!
//! ## Invarianten
//! - `INV-FSYNC-1`: Schlägt ein Flush/fsync fehl, wird das Handle unwiderruflich vergiftet.
//!   Keine nachfolgende Schreiboperation wird auf demselben Deskriptor ausgeführt.
//! - `INV-VFS-ATOMIC-1`: Atomares Schreiben (`write_atomic`) erfolgt strikt über temporäre Dateien
//!   im selben Zielverzeichnis, gefolgt von `sync_file`, atomarem Rename und Verzeichnis-Sync.

/// Fehlertypen und Result-Definitionen für Dateisystemoperationen.
pub mod error;
/// Fehlerinjektor und VFS-Dekorator zur Simulation von Ausfällen und Crash-Punkten.
pub mod fault_injector;
/// Fail-Closed-Zustandsverfolgung und Poisoning-Zustandsautomat (§18.4).
pub mod health;
/// Append-only-Journal mit Prüfsummen und Pending-Delete-Journal für Crash-Resilienz.
pub mod journal;
/// Plattformspezifische synchrone Flush-Primitive für Dateien und Verzeichnisse.
pub mod platform;
/// Sicheres Überschreiben (Scrubbing) und Integritätsprüfung mit Ratenbegrenzung.
pub mod scrub;
/// Virtuelles Dateisystem (VFS) mit atomaren Ersetzungen und synchroner Schnittstelle.
pub mod vfs;

pub use error::{DurableFsError, Error, Result};
pub use fault_injector::{FaultInjector, NoFault};
pub use health::{AtomicHealthState, HealthEvent, HealthState};
pub use journal::{
    AppendOnlyJournal, JournalSyncPolicy, PendingDeleteJournal, RecordRecoveryPolicy,
};
pub use platform::FlushKind;
pub use scrub::{
    scrub_file_with_rate_limit, scrub_if_safe, scrub_if_safe_with_injector, ScrubOutcome,
    ScrubPolicy, ScrubReceipt, ScrubRefusal,
};
pub use vfs::{RealFs, StdVfs, Vfs};

use crate::platform::{sync_dir_path, sync_file};
use crate::vfs::{generate_tmp_path, TmpFileGuard};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Dateisystemklassifikation für Persistenz- und Locking-Garantien (§23.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FsClass {
    /// Lokales Dateisystem mit unterstützten POSIX/Win32-Durability-Garantien.
    LocalSupported,
    /// Netzlaufwerk (NFS, SMB, CIFS).
    Network,
    /// Benutzerdefiniertes Dateisystem im Userspace (FUSE).
    Fuse,
    /// Cloud-synchronisierter Ordner (OneDrive, Dropbox, iCloud Drive, etc.).
    CloudSync,
    /// Unbekanntes oder nicht klassifizierbares Dateisystem.
    Unknown,
}

impl FsClass {
    /// Liefert `true`, wenn das Dateisystem vollständige lokale Persistenz und Locking unterstützt.
    #[must_use]
    pub const fn is_supported_local(&self) -> bool {
        matches!(self, Self::LocalSupported)
    }
}

/// Flusht eine geöffnete Datei auf die Festplatte.
pub fn flush_file(file: &File, kind: FlushKind) -> Result<()> {
    sync_file(file, kind).map_err(DurableFsError::Io)
}

/// Flusht Verzeichniseinträge auf stabile Speichermedien.
pub fn flush_dir(dir: &Path) -> Result<()> {
    sync_dir_path(dir).map_err(DurableFsError::Io)
}

fn is_cloud_sync_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    path_str.contains("onedrive")
        || path_str.contains("dropbox")
        || path_str.contains("icloud")
        || path_str.contains("mobile documents")
        || path_str.contains("google drive")
        || path_str.contains("googledrive")
        || path_str.contains("box sync")
        || path_str.contains("owncloud")
        || path_str.contains("nextcloud")
}

/// Ermittelt die Dateisystemklasse für den angegebenen Pfad (§23.4).
pub fn fs_class(path: &Path) -> Result<FsClass> {
    if is_cloud_sync_path(path) {
        return Ok(FsClass::CloudSync);
    }

    if !path.exists() && path.parent().is_none_or(|p| !p.exists()) {
        return Err(DurableFsError::NotFound {
            path: path.to_path_buf(),
        });
    }

    Ok(FsClass::LocalSupported)
}

/// Benennt eine Datei atomar um und flusht das übergeordnete Verzeichnis.
pub fn rename_durable(from: &Path, to: &Path) -> Result<()> {
    let parent = to.parent().ok_or_else(|| {
        DurableFsError::InvalidArgument("target path has no parent directory".to_string())
    })?;

    std::fs::rename(from, to).map_err(DurableFsError::Io)?;
    flush_dir(parent)?;
    Ok(())
}

/// Markierungstyp zur Kennzeichnung eines durch `fsync`- oder Schreibfehler vergifteten Handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoisonedHandle;

/// Ein Wrapper um [`File`], der das Handle bei einem `fsync`- oder Schreibfehler dauerhaft vergiftet (`INV-FSYNC-1`).
#[derive(Debug)]
pub struct DurableHandle<F: FaultInjector = NoFault> {
    file: Option<File>,
    is_poisoned: bool,
    injector: F,
    path: Option<PathBuf>,
}

impl DurableHandle<NoFault> {
    /// Kapselt eine geöffnete [`File`] in ein neues [`DurableHandle`] mit standardmäßigem [`NoFault`].
    pub fn new(file: File) -> Self {
        Self::new_with_injector(file, NoFault)
    }
}

impl<F: FaultInjector> DurableHandle<F> {
    /// Kapselt eine geöffnete [`File`] mit einem benutzerdefinierten [`FaultInjector`].
    pub fn new_with_injector(file: File, injector: F) -> Self {
        Self {
            file: Some(file),
            is_poisoned: false,
            injector,
            path: None,
        }
    }

    /// Kapselt eine geöffnete [`File`] mit zugehörigem Pfad und benutzerdefiniertem [`FaultInjector`].
    pub fn new_with_path_and_injector(file: File, path: PathBuf, injector: F) -> Self {
        Self {
            file: Some(file),
            is_poisoned: false,
            injector,
            path: Some(path),
        }
    }

    /// Prüft, ob dieses Handle aufgrund eines vorangegangenen Flush-Fehlers vergiftet ist (`INV-FSYNC-1`).
    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.is_poisoned
    }

    /// Flusht Dateiinhalte synchron auf die Festplatte und führt Fehlerinjektions-Hooks aus.
    pub fn flush(&mut self, kind: FlushKind) -> Result<()> {
        if self.is_poisoned {
            return Err(DurableFsError::PoisonedHandle);
        }

        let Some(file) = self.file.as_ref() else {
            return Err(DurableFsError::InvalidArgument(
                "file handle has been closed".to_string(),
            ));
        };

        if let Err(err) = self.injector.on_fsync(self.path.as_deref()) {
            self.is_poisoned = true;
            return Err(DurableFsError::Io(err));
        }

        if let Err(err) = sync_file(file, kind) {
            self.is_poisoned = true;
            Err(DurableFsError::Io(err))
        } else {
            Ok(())
        }
    }

    /// Liefert eine unveränderliche Referenz auf die zugrundeliegende Datei, oder `None`, falls das Handle vergiftet ist.
    #[must_use]
    pub fn get_ref(&self) -> Option<&File> {
        if self.is_poisoned {
            None
        } else {
            self.file.as_ref()
        }
    }

    /// Liefert eine veränderliche Referenz auf die zugrundeliegende Datei, oder `None`, falls das Handle vergiftet ist.
    pub fn get_mut(&mut self) -> Option<&mut File> {
        if self.is_poisoned {
            None
        } else {
            self.file.as_mut()
        }
    }

    /// Vergiftet dieses Handle explizit manuell.
    pub fn poison(&mut self) {
        self.is_poisoned = true;
    }
}

impl<F: FaultInjector> Write for DurableHandle<F> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.is_poisoned {
            return Err(std::io::Error::other(
                "handle is poisoned due to previous fsync failure (INV-FSYNC-1)",
            ));
        }

        let Some(file) = self.file.as_mut() else {
            return Err(std::io::Error::other("file handle has been closed"));
        };

        self.injector.on_write(self.path.as_deref(), buf.len())?;
        file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        DurableHandle::flush(self, FlushKind::DataAndMetadata).map_err(|e| match e {
            DurableFsError::Io(io_err) => io_err,
            other => std::io::Error::other(other.to_string()),
        })
    }
}

/// Sperrmodus für Verzeichnissperren gemäß Spezifikation §7.3.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LockMode {
    /// Standard-Sperre: Erfordert ein unterstütztes lokales Dateisystem.
    #[default]
    Normal,
    /// Reiner In-Memory-Modus: Umgeht Dateisystemprüfungen.
    MemoryOnly,
}

/// RAII-Guard für eine Verzeichnissperre (`.dirlock`).
#[derive(Debug)]
pub struct DirLock {
    lock_path: PathBuf,
    _file: Option<File>,
    mode: LockMode,
}

impl DirLock {
    /// Erwirbt eine Sperre auf `dir` im angegebenen [`LockMode`].
    pub fn acquire(dir: &Path, mode: LockMode) -> Result<Self> {
        let class = fs_class(dir)?;

        if mode == LockMode::Normal && !class.is_supported_local() {
            let class_str = match class {
                FsClass::Network => "network",
                FsClass::Fuse => "fuse",
                FsClass::CloudSync => "cloud_sync",
                _ => "unsupported_filesystem",
            };
            return Err(DurableFsError::UnsupportedFilesystem(format!(
                "directory locking rejected for filesystem class: {}",
                class_str
            )));
        }

        let lock_path = dir.join(".dirlock");
        let file = if mode == LockMode::Normal {
            let mut options = OpenOptions::new();
            options.write(true).create(true);
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                options.share_access(7);
            }
            Some(options.open(&lock_path).map_err(DurableFsError::Io)?)
        } else {
            None
        };

        Ok(Self {
            lock_path,
            _file: file,
            mode,
        })
    }

    /// Liefert den aktiven [`LockMode`] dieser Verzeichnissperre.
    #[must_use]
    pub fn mode(&self) -> LockMode {
        self.mode
    }
}

impl Drop for DirLock {
    fn drop(&mut self) {
        if self.mode == LockMode::Normal {
            let _file = self._file.take();
            drop(_file);
            let _ = std::fs::remove_file(&self.lock_path);
        }
    }
}

/// Synchronisiert die Verzeichnismetadaten eines Ordners mit Standard-Injektor [`NoFault`].
pub fn sync_dir(dir: &Path) -> Result<()> {
    sync_dir_with_injector(dir, &NoFault)
}

/// Synchronisiert die Verzeichnismetadaten eines Ordners mit benutzerdefiniertem [`FaultInjector`].
pub fn sync_dir_with_injector<F: FaultInjector>(dir: &Path, injector: &F) -> Result<()> {
    injector.on_dir_fsync(dir).map_err(DurableFsError::Io)?;
    flush_dir(dir)
}

/// Ersetzt den Inhalt der Datei unter `path` atomar mit `bytes` unter Verwendung des Standardinjektors [`NoFault`].
pub fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<()> {
    atomic_replace_with_injector(path, bytes, &NoFault)
}

/// Ersetzt den Inhalt der Datei unter `path` atomar mit `bytes` unter Verwendung eines [`FaultInjector`] (`INV-VFS-ATOMIC-1`).
pub fn atomic_replace_with_injector<F: FaultInjector>(
    path: &Path,
    bytes: &[u8],
    injector: &F,
) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        DurableFsError::InvalidArgument("target path has no parent directory".to_string())
    })?;

    let tmp_path = generate_tmp_path(path)?;

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_access(7);
    }

    let mut tmp_file = options.open(&tmp_path).map_err(DurableFsError::Io)?;
    let guard = TmpFileGuard(&tmp_path);

    injector
        .on_write(Some(&tmp_path), bytes.len())
        .map_err(DurableFsError::Io)?;
    tmp_file.write_all(bytes).map_err(DurableFsError::Io)?;

    injector
        .on_fsync(Some(&tmp_path))
        .map_err(DurableFsError::Io)?;
    sync_file(&tmp_file, FlushKind::DataAndMetadata).map_err(DurableFsError::Io)?;
    drop(tmp_file);

    injector
        .on_rename(&tmp_path, path)
        .map_err(DurableFsError::Io)?;
    std::fs::rename(&tmp_path, path).map_err(DurableFsError::Io)?;

    injector.on_dir_fsync(parent).map_err(DurableFsError::Io)?;
    flush_dir(parent)?;

    std::mem::forget(guard);

    Ok(())
}

/// Entfernt eine Datei dauerhaft und flusht das Elternverzeichnis mit Standardinjektor [`NoFault`].
pub fn durable_remove(path: &Path) -> Result<()> {
    durable_remove_with_injector(path, &NoFault)
}

/// Entfernt eine Datei dauerhaft und flusht das Elternverzeichnis mit benutzerdefiniertem [`FaultInjector`].
pub fn durable_remove_with_injector<F: FaultInjector>(path: &Path, injector: &F) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        DurableFsError::InvalidArgument("target path has no parent directory".to_string())
    })?;

    std::fs::remove_file(path).map_err(DurableFsError::Io)?;
    injector.on_dir_fsync(parent).map_err(DurableFsError::Io)?;
    flush_dir(parent)?;
    Ok(())
}

/// Überschreibt eine Datei vollständig mit Nullen und entfernt sie dauerhaft (mit [`NoFault`]).
pub fn scrub_and_remove(path: &Path) -> Result<ScrubReceipt> {
    scrub_and_remove_with_injector(path, &NoFault)
}

/// Überschreibt eine Datei mit Nullen und entfernt sie dauerhaft unter Verwendung eines [`FaultInjector`].
pub fn scrub_and_remove_with_injector<F: FaultInjector>(
    path: &Path,
    injector: &F,
) -> Result<ScrubReceipt> {
    let metadata = path.symlink_metadata().map_err(DurableFsError::Io)?;
    let file_type = metadata.file_type();

    if file_type.is_symlink() || !file_type.is_file() {
        return Err(DurableFsError::InvalidArgument(
            "path is a symlink or not a regular file".to_string(),
        ));
    }

    let len = metadata.len();
    if len > 0 {
        let mut options = OpenOptions::new();
        options.write(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_access(7);
        }

        let mut file = options.open(path).map_err(DurableFsError::Io)?;

        let zeros = [0u8; 64 * 1024];
        let mut remaining = len;

        while remaining > 0 {
            let chunk_size = usize::try_from(remaining)
                .unwrap_or(zeros.len())
                .min(zeros.len());
            injector
                .on_write(Some(path), chunk_size)
                .map_err(DurableFsError::Io)?;
            let slice = zeros.get(..chunk_size).ok_or_else(|| {
                DurableFsError::InvalidArgument("chunk_size out of bounds".to_string())
            })?;
            file.write_all(slice).map_err(DurableFsError::Io)?;
            remaining -= chunk_size as u64;
        }

        injector.on_fsync(Some(path)).map_err(DurableFsError::Io)?;
        sync_file(&file, FlushKind::DataAndMetadata).map_err(DurableFsError::Io)?;
    }

    durable_remove_with_injector(path, injector)?;

    Ok(ScrubReceipt {
        bytes_scrubbed: len,
    })
}
