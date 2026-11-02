#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

//! # Dauerhafte Dateisystem-Operationen und synchrone Disk-Flushes (`contextra-durable-fs`)
//!
//! Zweck: Dateisystem-Implementierung [`real_vfs::RealVfs`] mit dauerhaften Schreib-
//! und Austauschoperationen gemäß Spezifikation §8.5 (DF-01 bis DF-12).
//!
//! Ring-Zugehörigkeit: Ring 1 (Dateisystem-Durability; in `Cargo.toml` als `ring = "1"` annotiert).
//!
//! Bereitgestellte Module:
//! - [`real_vfs`]: Produktions-Implementierung des kanonischen [`contextra_ports::vfs::Vfs`]-Traits.
//! - [`atomic`]: Atomare Austausch- und Schreiboperationen (`write_atomic`).
//! - [`dir_lock`]: Exklusive Verzeichnissperre (`LOCK`-Datei) mit Schutz vor unsicheren Netzdateisystemen.
//! - [`probe`]: Startprobe zur Validierung von Plattformstufen und Dateisystemgarantien.
//! - [`scrub`]: Sicheres Überschreiben vor dem Unlink nur auf rotierenden Medien ohne Copy-on-Write.

use std::path::Path;

/// Quittung über das sichere Überschreiben (Scrubbing) einer Datei.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrubReceipt {
    /// Gesamtanzahl der vor dem Löschen mit Nullen überschriebenen Bytes.
    pub bytes_scrubbed: u64,
}

/// Trait für deterministische Fehlerinjektion in Persistenzpfaden (intern für scrub-Kompatibilität).
pub trait FaultInjector: Send + Sync {
    /// Hook vor oder während eines Schreibvorgangs.
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> std::io::Result<()> {
        Ok(())
    }
    /// Hook vor dem Hardware-Flush einer Datei.
    fn on_fsync(&self, _path: Option<&Path>) -> std::io::Result<()> {
        Ok(())
    }
    /// Hook vor einer atomaren Umbenennungsoperation.
    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        Ok(())
    }
    /// Hook vor dem Hardware-Flush eines Verzeichnisses.
    fn on_dir_fsync(&self, _dir: &Path) -> std::io::Result<()> {
        Ok(())
    }
}

/// Null-Kosten-Standardinjektor für Produktivumgebungen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoFault;

impl FaultInjector for NoFault {}

impl<T: FaultInjector + ?Sized> FaultInjector for &T {
    fn on_write(&self, path: Option<&Path>, bytes: usize) -> std::io::Result<()> {
        (**self).on_write(path, bytes)
    }
    fn on_fsync(&self, path: Option<&Path>) -> std::io::Result<()> {
        (**self).on_fsync(path)
    }
    fn on_rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        (**self).on_rename(from, to)
    }
    fn on_dir_fsync(&self, dir: &Path) -> std::io::Result<()> {
        (**self).on_dir_fsync(dir)
    }
}

pub(crate) fn durable_remove_with_injector<F: FaultInjector>(
    path: &Path,
    injector: &F,
) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target path has no parent directory",
        )
    })?;

    std::fs::remove_file(path)?;
    injector.on_dir_fsync(parent)?;
    contextra_sys::flush_dir(parent)?;
    Ok(())
}

/// Reales Virtuelles Dateisystem (`RealVfs`) als primäre Produktionsimplementierung.
pub mod real_vfs {
    use contextra_ports::vfs::{Vfs, VfsFile, VfsOpenOptions};
    use contextra_ports::{ContextraError, Result};
    use contextra_sys::{flush_dir, flush_file, rename_durable, FlushKind};
    use std::fs::{self, File, OpenOptions};
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Mutex;

    static VFS_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Dateihandle für VfsFile mit dauerhafter Vergiftung bei I/O- oder fsync-Fehlern (DF-03, INV-FSYNC-1).
    pub struct RealFile {
        file: Mutex<Option<File>>,
        poisoned: AtomicBool,
    }

    impl RealFile {
        /// Erstellt ein neues [`RealFile`]-Handle.
        pub fn new(file: File) -> Self {
            Self {
                file: Mutex::new(Some(file)),
                poisoned: AtomicBool::new(false),
            }
        }

        /// Prüft, ob das Handle vergiftet ist.
        #[must_use]
        pub fn is_poisoned(&self) -> bool {
            self.poisoned.load(Ordering::Acquire)
        }

        fn check_not_poisoned(&self) -> std::io::Result<()> {
            if self.is_poisoned() {
                Err(std::io::Error::other(
                    "handle is poisoned due to previous I/O or fsync failure",
                ))
            } else {
                Ok(())
            }
        }

        fn poison_on_err<T>(&self, res: std::io::Result<T>) -> std::io::Result<T> {
            if res.is_err() {
                self.poisoned.store(true, Ordering::Release);
            }
            res
        }
    }

    impl VfsFile for RealFile {
        fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
            self.check_not_poisoned()?;
            let Ok(mut guard) = self.file.lock() else {
                self.poisoned.store(true, Ordering::Release);
                return Err(std::io::Error::other("lock poisoned"));
            };
            let Some(file) = guard.as_mut() else {
                return Err(std::io::Error::other("file closed"));
            };

            let res = (|| -> std::io::Result<usize> {
                file.seek(SeekFrom::Start(offset))?;
                file.read(buf)
            })();
            self.poison_on_err(res)
        }

        fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
            self.check_not_poisoned()?;
            let Ok(mut guard) = self.file.lock() else {
                self.poisoned.store(true, Ordering::Release);
                return Err(std::io::Error::other("lock poisoned"));
            };
            let Some(file) = guard.as_mut() else {
                return Err(std::io::Error::other("file closed"));
            };

            let res = (|| -> std::io::Result<usize> {
                file.seek(SeekFrom::Start(offset))?;
                file.write(buf)
            })();
            self.poison_on_err(res)
        }

        fn set_len(&self, len: u64) -> std::io::Result<()> {
            self.check_not_poisoned()?;
            let Ok(guard) = self.file.lock() else {
                self.poisoned.store(true, Ordering::Release);
                return Err(std::io::Error::other("lock poisoned"));
            };
            let Some(file) = guard.as_ref() else {
                return Err(std::io::Error::other("file closed"));
            };

            let res = file.set_len(len);
            self.poison_on_err(res)
        }

        fn len(&self) -> std::io::Result<u64> {
            self.check_not_poisoned()?;
            let Ok(guard) = self.file.lock() else {
                self.poisoned.store(true, Ordering::Release);
                return Err(std::io::Error::other("lock poisoned"));
            };
            let Some(file) = guard.as_ref() else {
                return Err(std::io::Error::other("file closed"));
            };

            let res = file.metadata().map(|m| m.len());
            self.poison_on_err(res)
        }

        fn sync_data(&self) -> std::io::Result<()> {
            self.check_not_poisoned()?;
            let Ok(guard) = self.file.lock() else {
                self.poisoned.store(true, Ordering::Release);
                return Err(std::io::Error::other("lock poisoned"));
            };
            let Some(file) = guard.as_ref() else {
                return Err(std::io::Error::other("file closed"));
            };

            let res = flush_file(file, FlushKind::DataOnly);
            self.poison_on_err(res)
        }

        fn sync_all(&self) -> std::io::Result<()> {
            self.check_not_poisoned()?;
            let Ok(guard) = self.file.lock() else {
                self.poisoned.store(true, Ordering::Release);
                return Err(std::io::Error::other("lock poisoned"));
            };
            let Some(file) = guard.as_ref() else {
                return Err(std::io::Error::other("file closed"));
            };

            let res = flush_file(file, FlushKind::DataAndMetadata);
            self.poison_on_err(res)
        }
    }

    /// Produktionsimplementierung des [`contextra_ports::vfs::Vfs`]-Traits.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct RealVfs;

    impl RealVfs {
        /// Erstellt eine neue Instanz von [`RealVfs`].
        #[must_use]
        pub const fn new() -> Self {
            Self
        }
    }

    impl Vfs for RealVfs {
        fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
            let mut options = OpenOptions::new();
            options.read(opts.read);
            options.write(opts.write);
            options.append(opts.append);
            options.truncate(opts.truncate);
            if opts.create_new {
                options.create_new(true);
            } else if opts.create {
                options.create(true);
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }

            let file = options.open(path).map_err(ContextraError::Io)?;
            Ok(Box::new(RealFile::new(file)))
        }

        fn read(&self, path: &Path) -> Result<Vec<u8>> {
            if !path.exists() {
                return Err(ContextraError::NotFound("file not found".to_string()));
            }
            fs::read(path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ContextraError::NotFound("file not found".to_string())
                } else {
                    ContextraError::Io(e)
                }
            })
        }

        fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
            let parent = path.parent().ok_or_else(|| {
                ContextraError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "target path has no parent directory",
                ))
            })?;

            if !parent.exists() {
                self.create_dir_all(parent)?;
            }

            let file_stem = path.file_name().and_then(|s| s.to_str()).unwrap_or("tmp");
            let count = VFS_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let pid = std::process::id();
            let tmp_filename = format!(".{}.tmp.{}.{}", file_stem, pid, count);
            let tmp_path = parent.join(tmp_filename);

            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }

            let mut tmp_file = options.open(&tmp_path).map_err(ContextraError::Io)?;

            struct TmpGuard<'a>(&'a Path);
            impl<'a> Drop for TmpGuard<'a> {
                fn drop(&mut self) {
                    let _ = fs::remove_file(self.0);
                }
            }
            let guard = TmpGuard(&tmp_path);

            tmp_file.write_all(data).map_err(ContextraError::Io)?;
            flush_file(&tmp_file, FlushKind::DataAndMetadata).map_err(ContextraError::Io)?;
            drop(tmp_file);

            rename_durable(&tmp_path, path).map_err(ContextraError::Io)?;
            flush_dir(parent).map_err(ContextraError::Io)?;

            std::mem::forget(guard);
            Ok(())
        }

        fn create_dir_all(&self, path: &Path) -> Result<()> {
            fs::create_dir_all(path).map_err(ContextraError::Io)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(path, fs::Permissions::from_mode(0o700))
                    .map_err(ContextraError::Io)?;
            }
            Ok(())
        }

        fn remove_file(&self, path: &Path) -> Result<()> {
            if !path.exists() {
                return Err(ContextraError::NotFound("file not found".to_string()));
            }
            let parent = path.parent();
            fs::remove_file(path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ContextraError::NotFound("file not found".to_string())
                } else {
                    ContextraError::Io(e)
                }
            })?;

            if let Some(dir) = parent {
                if dir.exists() {
                    flush_dir(dir).map_err(ContextraError::Io)?;
                }
            }
            Ok(())
        }

        fn exists(&self, path: &Path) -> Result<bool> {
            Ok(path.exists())
        }

        fn len(&self, path: &Path) -> Result<u64> {
            if !path.exists() {
                return Err(ContextraError::NotFound("file not found".to_string()));
            }
            fs::metadata(path).map(|m| m.len()).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ContextraError::NotFound("file not found".to_string())
                } else {
                    ContextraError::Io(e)
                }
            })
        }

        fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
            if !path.exists() {
                return Err(ContextraError::NotFound("directory not found".to_string()));
            }
            let entries = fs::read_dir(path).map_err(ContextraError::Io)?;
            let mut names = Vec::new();
            for entry_res in entries {
                let entry = entry_res.map_err(ContextraError::Io)?;
                let file_name = entry.file_name();
                names.push(file_name.to_string_lossy().to_string());
            }
            names.sort();
            Ok(names)
        }

        fn sync_dir(&self, path: &Path) -> Result<()> {
            if !path.exists() {
                return Err(ContextraError::NotFound("directory not found".to_string()));
            }
            flush_dir(path).map_err(ContextraError::Io)
        }
    }
}

/// Atomare Austausch- und Schreiboperationen (`write_atomic`, DF-02).
pub mod atomic {
    use contextra_sys::{flush_dir, flush_file, rename_durable, FlushKind};
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    static ATOMIC_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TmpFileGuard<'a>(&'a Path);
    impl<'a> Drop for TmpFileGuard<'a> {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(self.0);
        }
    }

    /// Ersetzt den Inhalt der Datei unter `path` atomar mit `bytes` (DF-02).
    pub fn atomic_replace(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        let parent = path.parent().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "target path has no parent directory",
            )
        })?;

        let file_stem = path.file_name().and_then(|s| s.to_str()).unwrap_or("tmp");
        let count = ATOMIC_COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let tmp_filename = format!(".{}.tmp.{}.{}", file_stem, pid, count);
        let tmp_path = parent.join(tmp_filename);

        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        let mut tmp_file = options.open(&tmp_path)?;
        let guard = TmpFileGuard(&tmp_path);

        tmp_file.write_all(bytes)?;
        flush_file(&tmp_file, FlushKind::DataAndMetadata)?;
        drop(tmp_file);

        rename_durable(&tmp_path, path)?;
        flush_dir(parent)?;

        std::mem::forget(guard);
        Ok(())
    }
}

/// Exklusive Verzeichnissperre (`LOCK`-Datei) gemäß SPEC D-50 und DF-05.
pub mod dir_lock {
    use contextra_sys::fs_class;
    use std::fs::{File, OpenOptions};
    use std::path::{Path, PathBuf};

    /// Sperrmodus für Verzeichnissperren.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum LockMode {
        /// Standard-Sperre: Erfordert ein unterstütztes lokales Dateisystem.
        #[default]
        Normal,
        /// Reiner In-Memory-Modus: Umgeht Dateisystemprüfungen.
        MemoryOnly,
    }

    /// RAII-Guard für eine Verzeichnissperre (`LOCK`).
    #[derive(Debug)]
    pub struct DirLock {
        lock_path: PathBuf,
        _file: Option<File>,
        mode: LockMode,
    }

    impl DirLock {
        /// Erwirbt eine exklusive Sperre auf `dir` im angegebenen [`LockMode`] (DF-05).
        pub fn acquire(dir: &Path, mode: LockMode) -> std::io::Result<Self> {
            let class = fs_class(dir)?;

            if mode == LockMode::Normal && !class.is_supported_local() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "directory locking rejected on unsupported filesystem",
                ));
            }

            let lock_path = dir.join("LOCK");
            let file = if mode == LockMode::Normal {
                let mut options = OpenOptions::new();
                options.write(true).create(true);
                Some(options.open(&lock_path)?)
            } else {
                None
            };

            Ok(Self {
                lock_path,
                _file: file,
                mode,
            })
        }

        /// Liefert den aktiven [`LockMode`].
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
}

/// Startprobe zur Validierung von Plattform- und Dateisystem-Garantien (DF-06).
pub mod probe;

/// Sicheres Überschreiben (Scrubbing) und Schutzmechanismen vor dem Löschen (DF-07).
pub mod scrub;

// Re-Exports für primäre Komponenten
pub use atomic::atomic_replace;
pub use dir_lock::{DirLock, LockMode};
pub use probe::{
    free_space, probe_dir, verdict, DetectedFs, FsProbe, ProbeVerdict, Tier, UnsupportedFilesystem,
};
pub use real_vfs::{RealFile, RealVfs};
pub use scrub::{scrub_if_safe, ScrubOutcome, ScrubPolicy, ScrubRefusal};
