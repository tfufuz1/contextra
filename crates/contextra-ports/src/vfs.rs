//! Virtual File System (VFS) port trait definition.
//!
//! # SPEC-REF & Rule 5 Compliance
//! In accordance with AGENTS.md Rule 5 ("I/O nur über `Vfs`") and contract V-VFS,
//! all file system I/O operations in storage, engine, and index layers must interact
//! exclusively through this abstract VFS interface.
//!
//! # Crash Consistency & Durability Contract
//! Concrete implementations of [`Vfs`] must satisfy strict crash-consistency guarantees:
//! - **Atomic Write (`write_atomic`)**:
//!   1. Data must first be written completely to a temporary file in the target directory (or sibling location on the same filesystem).
//!   2. The temporary file must be explicitly flushed/synced to disk (`fsync` / `FlushFileBuffers`).
//!   3. The temporary file must be atomically renamed to the target `path`.
//!   4. The parent directory must be synced (`sync_dir`) to ensure directory entry durability on filesystems like ext4/xfs.
//!   5. If any failure occurs prior to final rename, the original file at `path` must remain untouched and uncorrupted.
//! - **Directory Sync (`sync_dir`)**:
//!   Flushes pending directory metadata changes (file additions/deletions/renames) to persistent storage.
//! - **Fail-Closed Propagation**:
//!   I/O errors must be converted into [`ContextraError::Io`](crate::error::ContextraError::Io) or appropriate domain errors
//!   and propagated immediately. Swallowing I/O errors or ignoring sync returns is strictly prohibited.
//! - **Cancel-Safety**:
//!   Synchronous VFS operations are inherently non-preemptible by async cancellations once executing within OS boundaries.

use crate::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Method via which medium verification was performed (SPEC D-113).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerifiedVia {
    /// Read bypassed page cache via Direct I/O (`O_DIRECT` or `F_NOCACHE`).
    Direct,
    /// Read performed after page cache eviction hint (`posix_fadvise(POSIX_FADV_DONTNEED)`).
    CacheDropped,
    /// Read performed through ordinary cache path (fallback if direct I/O or drop unsupported).
    Cached,
}

/// Options configuring how a file is opened via [`Vfs::open`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VfsOpenOptions {
    /// Opens the file with read access.
    pub read: bool,
    /// Opens the file with write access.
    pub write: bool,
    /// Creates a new file if it does not exist.
    pub create: bool,
    /// Creates a new file, failing if it already exists (`create_new`).
    pub create_new: bool,
    /// Truncates the file length to 0 upon opening.
    pub truncate: bool,
    /// Sets append mode for write operations.
    pub append: bool,
}

impl VfsOpenOptions {
    /// Creates options configured for read-only access.
    pub fn read_only() -> Self {
        Self {
            read: true,
            ..Default::default()
        }
    }

    /// Creates options configured for write-only access, creating the file if missing.
    pub fn write_create() -> Self {
        Self {
            write: true,
            create: true,
            ..Default::default()
        }
    }
}

/// Abstract file handle providing positional I/O without modifying internal file offsets.
pub trait VfsFile: Send + Sync {
    /// Reads data from byte offset `offset` into `buf` without moving the file pointer.
    ///
    /// # Fehler
    /// Liefert `std::io::Error` bei Lesefehlern oder ungültigem Offset.
    ///
    /// # Crash
    /// Reiner positioneller Lesezugriff; verändert keine Dateimetadaten oder persistente Zustände.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;

    /// Reads verified data from storage medium bypassing or dropping the page cache (SPEC D-113).
    ///
    /// In accordance with SPEC D-113:
    /// Returns read bytes and classification `VerifiedVia ∈ {Direct, CacheDropped, Cached}`.
    /// Default implementation reads positionally via `read_at` classified as `VerifiedVia::Cached`.
    ///
    /// # Fehler
    /// Liefert `std::io::Error` bei I/O-Fehlern oder Leseproblemen.
    ///
    /// # Crash
    /// Reiner Lesezugriff zur Verifikation von Slots oder Medienblöcken.
    fn read_verified(&self, offset: u64, len: usize) -> std::io::Result<(Vec<u8>, VerifiedVia)> {
        let mut buf = vec![0u8; len];
        let n = self.read_at(&mut buf, offset)?;
        buf.truncate(n);
        Ok((buf, VerifiedVia::Cached))
    }

    /// Writes data from `buf` to byte offset `offset` without moving the file pointer.
    ///
    /// # Fehler
    /// Liefert `std::io::Error` bei Schreibfehlern oder Speicherplatzmangel.
    ///
    /// # Crash
    /// Schreibt Datenblöcke; Persistenz erfordert nachfolgendes `sync_data` oder `sync_all`.
    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>;

    /// Truncates or extends the underlying file length to `len` bytes.
    ///
    /// # Fehler
    /// Liefert `std::io::Error` bei Kürzungs- oder Allokationsfehlern.
    ///
    /// # Crash
    /// Modifiziert die Dateigröße; wird erst durch `sync_all` crash-resistent festgeschrieben.
    fn set_len(&self, len: u64) -> std::io::Result<()>;

    /// Queries the current physical file length in bytes.
    ///
    /// # Fehler
    /// Liefert `std::io::Error` falls Dateimetadaten nicht abgefragt werden können.
    ///
    /// # Crash
    /// Reines Abfragen von Dateimetadaten ohne Seiteneffekte.
    fn len(&self) -> std::io::Result<u64>;

    /// Returns `true` if the file length is zero.
    ///
    /// # Fehler
    /// Liefert `std::io::Error` falls die Längenabfrage fehlschlägt.
    ///
    /// # Crash
    /// Reine Längenprüfung ohne Seiteneffekte.
    fn is_empty(&self) -> std::io::Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Flushes data blocks to disk (`fdatasync`).
    ///
    /// # Fehler
    /// Liefert `std::io::Error` bei fehlschlagendem Flush. Bei Sync-Fehlern muss das Handle vergiftet werden (DF-03, D-55, T-FSYNC-1).
    ///
    /// # Crash
    /// Garantiert die Haltbarkeit der geschriebenen Datenblöcke bis zum Aufrufzeitpunkt.
    fn sync_data(&self) -> std::io::Result<()>;

    /// Flushes data blocks and metadata to disk (`fsync`).
    ///
    /// # Fehler
    /// Liefert `std::io::Error` bei Synchronisationsfehlern. Scheitert `sync_all`, ist das Handle dauerhaft vergiftet und darf nicht weiterverwendet werden (D-55, I-10).
    ///
    /// # Crash
    /// Bewirkt Haltbarkeit aller Datenblöcke und Inode-Metadaten (z. B. Dateilänge).
    fn sync_all(&self) -> std::io::Result<()>;
}

/// Abstract object-safe Virtual File System interface.
///
/// Implements required file operations for LSM storage engine, SSTables, WAL sidecars,
/// vault slot storage, and vector index persistence (SPEC 3.1, D-113, DF-01..DF-03).
pub trait Vfs: Send + Sync + 'static {
    /// Opens a file according to the specified options.
    ///
    /// # Contract V-VFS
    /// Default implementation delegates to `std::io::ErrorKind::Unsupported` unless overridden.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) bei Berechtigungs-, Nicht-Gefunden- oder I/O-Fehlern.
    ///
    /// # Crash
    /// Öffnet Dateideskriptor; erzeugt ggf. eine leere Datei bei `create_new`/`create`. Keine implizite Datenpersistenz.
    fn open(&self, _path: &Path, _opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        Err(crate::ContextraError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "positional VfsFile open is not supported by default in this Vfs implementation",
        )))
    }

    /// Reads data from byte offset `offset` into `buf` for file at `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) bei I/O-Fehlern oder Leseproblemen.
    ///
    /// # Crash
    /// Reiner positioneller Lesezugriff; modifiziert keine persistenten Dateiinhalte oder Metadaten.
    fn read_at(&self, path: &Path, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let file = self.open(path, &VfsOpenOptions::read_only())?;
        file.read_at(buf, offset).map_err(crate::ContextraError::Io)
    }

    /// Reads verified bytes directly from media or cache-dropped layer (SPEC D-113).
    ///
    /// Returns the verified byte payload and the verification classification `VerifiedVia`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) bei Lesefehlern.
    ///
    /// # Crash
    /// Reiner Lesezugriff zur Medien- und Cache-Verifikation; keine Modifikation auf dem Speichermedium.
    fn read_verified(
        &self,
        path: &Path,
        offset: u64,
        len: usize,
    ) -> Result<(Vec<u8>, VerifiedVia)> {
        let file = self.open(path, &VfsOpenOptions::read_only())?;
        file.read_verified(offset, len)
            .map_err(crate::ContextraError::Io)
    }

    /// Reads the entire contents of a file at `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
    /// falls die Datei fehlt oder nicht gelesen werden kann.
    ///
    /// # Crash
    /// Reiner Lesezugriff; keine Zustandsänderung auf dem Speichermedium.
    fn read(&self, path: &Path) -> Result<Vec<u8>>;

    /// Atomically writes `data` to `path` with crash-consistency guarantees.
    ///
    /// # Crash-Consistency Contract
    /// Writes to a temporary file, calls `fsync` on the file handle, renames the temporary
    /// file over `path`, and syncs the parent directory (`sync_dir`).
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls Schreiben, Synchronisieren oder Umbenennen fehlschlägt.
    ///
    /// # Crash
    /// Crash-konsistente All-or-Nothing-Garantie: Bricht der Vorgang vor atomarem Rename ab, bleibt die Originaldatei unberührt.
    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>;

    /// Creates a directory and all parent directories along `path` if missing.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls Verzeichnisanlage fehlschlägt.
    ///
    /// # Crash
    /// Idempotente Verzeichnisanlage; Durability der Verzeichnisstruktur wird über `sync_dir` sichergestellt.
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Removes a file at `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
    /// falls das Entfernen fehlschlägt oder die Datei fehlt.
    ///
    /// # Crash
    /// Entfernt Verzeichniseintrag; Persistenz erfordert `sync_dir` des übergeordneten Verzeichnisses.
    fn remove_file(&self, path: &Path) -> Result<()>;

    /// Canonical alias for removing a file (SPEC D-113 / Purge-Primitive).
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
    /// falls das Entfernen fehlschlägt oder die Datei fehlt.
    ///
    /// # Crash
    /// Entfernt Verzeichniseintrag; Persistenz erfordert `sync_dir` des übergeordneten Verzeichnisses.
    fn unlink(&self, path: &Path) -> Result<()> {
        self.remove_file(path)
    }

    /// Atomically renames a file from `from` to `to`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls das Umbenennen fehlschlägt.
    ///
    /// # Crash
    /// Atomarer Dateisystem-Rename; bei Systemabsturz existiert entweder der Quell- oder der Zielpfad.
    fn rename(&self, _from: &Path, _to: &Path) -> Result<()> {
        Err(crate::ContextraError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "atomic rename is not supported by default in this Vfs implementation",
        )))
    }

    /// Reads all entry paths within directory `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) bei Lesefehlern des Verzeichnisses.
    ///
    /// # Crash
    /// Reiner Lesezugriff; keine Verzeichnisänderung.
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries = self.list_dir(path)?;
        Ok(entries.into_iter().map(|name| path.join(name)).collect())
    }

    /// Returns `true` if a file or directory exists at `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls Pfadstatus nicht ermittelt werden kann.
    ///
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn exists(&self, path: &Path) -> Result<bool>;

    /// Returns the length in bytes of the file at `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
    /// falls Metadaten nicht abgefragt werden können.
    ///
    /// # Crash
    /// Reines Abfragen der Dateilänge ohne Seiteneffekte.
    fn len(&self, path: &Path) -> Result<u64>;

    /// Lists entry names (file and directory names, excluding `.` and `..`) inside directory `path`.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) oder [`ContextraError::NotFound`](crate::error::ContextraError::NotFound)
    /// falls Pfad kein Verzeichnis ist oder nicht gelesen werden kann.
    ///
    /// # Crash
    /// Reiner Lesezugriff auf Verzeichniseinträge.
    fn list_dir(&self, path: &Path) -> Result<Vec<String>>;

    /// Canonical alias for listing entries in directory `path` (SPEC 3.1).
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) bei Lesefehlern des Verzeichnisses.
    ///
    /// # Crash
    /// Reiner Lesezugriff auf Verzeichniseinträge; keine Seiteneffekte.
    fn list(&self, path: &Path) -> Result<Vec<String>> {
        self.list_dir(path)
    }

    /// Flushes directory metadata to persistent storage for durable file additions and renames.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::Io`](crate::error::ContextraError::Io) falls Directory-Sync fehlschlägt.
    ///
    /// # Crash
    /// Schreibt Verzeichnis-Metadaten dauerhaft auf Festplatte; schützt vor Datenverlust bei Stromausfall.
    fn sync_dir(&self, path: &Path) -> Result<()>;
}

impl<T: Vfs + ?Sized> Vfs for Arc<T> {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        (**self).open(path, opts)
    }

    fn read_at(&self, path: &Path, offset: u64, buf: &mut [u8]) -> Result<usize> {
        (**self).read_at(path, offset, buf)
    }

    fn read_verified(
        &self,
        path: &Path,
        offset: u64,
        len: usize,
    ) -> Result<(Vec<u8>, VerifiedVia)> {
        (**self).read_verified(path, offset, len)
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        (**self).read(path)
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        (**self).write_atomic(path, data)
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        (**self).create_dir_all(path)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        (**self).remove_file(path)
    }

    fn unlink(&self, path: &Path) -> Result<()> {
        (**self).unlink(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        (**self).rename(from, to)
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        (**self).read_dir(path)
    }

    fn exists(&self, path: &Path) -> Result<bool> {
        (**self).exists(path)
    }

    fn len(&self, path: &Path) -> Result<u64> {
        (**self).len(path)
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        (**self).list_dir(path)
    }

    fn list(&self, path: &Path) -> Result<Vec<String>> {
        (**self).list(path)
    }

    fn sync_dir(&self, path: &Path) -> Result<()> {
        (**self).sync_dir(path)
    }
}
