use crate::error::{DurableFsError, Result};
use crate::vfs::Vfs;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// Schnittstelle zur Injektion von I/O-Fehlern in Persistenzpfade.
pub trait FaultInjector: Send + Sync {
    /// Hook vor oder während eines Schreibvorgangs.
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> io::Result<()> {
        Ok(())
    }

    /// Hook vor dem Hardware-Flush (`fsync`) einer Datei.
    fn on_fsync(&self, _path: Option<&Path>) -> io::Result<()> {
        Ok(())
    }

    /// Hook vor einer atomaren Umbenennung (`rename`).
    fn on_rename(&self, _from: &Path, _to: &Path) -> io::Result<()> {
        Ok(())
    }

    /// Hook vor dem Hardware-Flush (`fsync`) eines Verzeichnisses.
    fn on_dir_fsync(&self, _dir: &Path) -> io::Result<()> {
        Ok(())
    }
}

/// Null-Kosten-Standardinjektor für Produktivumgebungen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoFault;

impl FaultInjector for NoFault {
    #[inline(always)]
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_fsync(&self, _path: Option<&Path>) -> io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_rename(&self, _from: &Path, _to: &Path) -> io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_dir_fsync(&self, _dir: &Path) -> io::Result<()> {
        Ok(())
    }
}

impl<T: FaultInjector + ?Sized> FaultInjector for &T {
    #[inline]
    fn on_write(&self, path: Option<&Path>, bytes: usize) -> io::Result<()> {
        (**self).on_write(path, bytes)
    }

    #[inline]
    fn on_fsync(&self, path: Option<&Path>) -> io::Result<()> {
        (**self).on_fsync(path)
    }

    #[inline]
    fn on_rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        (**self).on_rename(from, to)
    }

    #[inline]
    fn on_dir_fsync(&self, dir: &Path) -> io::Result<()> {
        (**self).on_dir_fsync(dir)
    }
}

/// Skriptbarer Fehlerzustand für Tests und Chaos-Simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectedFault {
    /// Speichermedium voll (`ENOSPC`).
    DiskFull,
    /// Hardware- oder I/O-Fehler (`EIO`).
    IoError,
    /// Vorzeitiger Abbruch bei fsync.
    FsyncFailure,
    /// Fehler beim atomaren Umbenennen.
    RenameFailure,
}

/// VFS-Dekorator zur präzisen Fehlerinjektion und Crash-Punkt-Simulation.
#[derive(Debug)]
pub struct FaultInjectingVfs<V: Vfs> {
    inner: V,
    fail_next_write: AtomicBool,
    fail_next_fsync: AtomicBool,
    fail_next_rename: AtomicBool,
    fail_next_dir_fsync: AtomicBool,
    synced_snapshots: Mutex<HashMap<PathBuf, Vec<u8>>>,
    dirty_snapshots: Mutex<HashMap<PathBuf, Vec<u8>>>,
}

impl<V: Vfs> FaultInjectingVfs<V> {
    /// Kapselt ein bestehendes [`Vfs`] in einen Injektor-Dekorator.
    pub fn new(inner: V) -> Self {
        Self {
            inner,
            fail_next_write: AtomicBool::new(false),
            fail_next_fsync: AtomicBool::new(false),
            fail_next_rename: AtomicBool::new(false),
            fail_next_dir_fsync: AtomicBool::new(false),
            synced_snapshots: Mutex::new(HashMap::new()),
            dirty_snapshots: Mutex::new(HashMap::new()),
        }
    }

    /// Konfiguriert, dass der nächste Schreibaufruf mit `ENOSPC` fehlschlägt.
    pub fn inject_write_disk_full(&self) {
        self.fail_next_write.store(true, Ordering::Release);
    }

    /// Konfiguriert, dass der nächste `sync_file`-Aufruf mit EIO fehlschlägt.
    pub fn inject_fsync_failure(&self) {
        self.fail_next_fsync.store(true, Ordering::Release);
    }

    /// Konfiguriert, dass das nächste `rename` fehlschlägt.
    pub fn inject_rename_failure(&self) {
        self.fail_next_rename.store(true, Ordering::Release);
    }

    /// Konfiguriert, dass der nächste Verzeichnis-Sync fehlschlägt.
    pub fn inject_dir_fsync_failure(&self) {
        self.fail_next_dir_fsync.store(true, Ordering::Release);
    }

    /// Simuliert einen plötzlichen Prozessabsturz (Crash-Punkt-Simulation):
    /// Alle noch nicht synchronisierten Datenänderungen gehen unwiderruflich verloren!
    pub fn simulate_crash(&self) -> Result<()> {
        let mut dirty = match self.dirty_snapshots.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let synced = match self.synced_snapshots.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        for (path, synced_data) in synced.iter() {
            self.inner.write_atomic(path, synced_data)?;
        }

        for path in dirty.keys() {
            if !synced.contains_key(path) && self.inner.exists(path) {
                let _ = self.inner.remove(path);
            }
        }

        dirty.clear();
        Ok(())
    }
}

impl<V: Vfs> Vfs for FaultInjectingVfs<V> {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        self.inner.read(path)
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        if self.fail_next_write.swap(false, Ordering::AcqRel) {
            return Err(DurableFsError::Io(io::Error::new(
                io::ErrorKind::StorageFull,
                "injected disk full (ENOSPC)",
            )));
        }

        if self.fail_next_fsync.swap(false, Ordering::AcqRel) {
            return Err(DurableFsError::Io(io::Error::other(
                "injected fsync failure (INV-FSYNC-1)",
            )));
        }

        if self.fail_next_rename.swap(false, Ordering::AcqRel) {
            return Err(DurableFsError::Io(io::Error::other(
                "injected rename failure",
            )));
        }

        if self.fail_next_dir_fsync.swap(false, Ordering::AcqRel) {
            return Err(DurableFsError::Io(io::Error::other(
                "injected dir fsync failure",
            )));
        }

        self.inner.write_atomic(path, data)?;

        if let Ok(mut synced) = self.synced_snapshots.lock() {
            synced.insert(path.to_path_buf(), data.to_vec());
        }
        if let Ok(mut dirty) = self.dirty_snapshots.lock() {
            dirty.remove(path);
        }

        Ok(())
    }

    fn append(&self, path: &Path, data: &[u8]) -> Result<()> {
        if self.fail_next_write.swap(false, Ordering::AcqRel) {
            return Err(DurableFsError::Io(io::Error::new(
                io::ErrorKind::StorageFull,
                "injected disk full on append",
            )));
        }

        if self.fail_next_fsync.swap(false, Ordering::AcqRel) {
            return Err(DurableFsError::Io(io::Error::other(
                "injected fsync failure on append",
            )));
        }

        self.inner.append(path, data)?;

        let current = self.inner.read(path)?;
        if let Ok(mut synced) = self.synced_snapshots.lock() {
            synced.insert(path.to_path_buf(), current);
        }

        Ok(())
    }

    fn sync_file(&self, path: &Path) -> Result<()> {
        if self.fail_next_fsync.swap(false, Ordering::AcqRel) {
            return Err(DurableFsError::Io(io::Error::other(
                "injected sync_file failure",
            )));
        }

        self.inner.sync_file(path)?;

        let current = self.inner.read(path)?;
        if let Ok(mut synced) = self.synced_snapshots.lock() {
            synced.insert(path.to_path_buf(), current);
        }

        Ok(())
    }

    fn sync_dir(&self, path: &Path) -> Result<()> {
        if self.fail_next_dir_fsync.swap(false, Ordering::AcqRel) {
            return Err(DurableFsError::Io(io::Error::other(
                "injected sync_dir failure",
            )));
        }
        self.inner.sync_dir(path)
    }

    fn remove(&self, path: &Path) -> Result<()> {
        self.inner.remove(path)?;
        if let Ok(mut synced) = self.synced_snapshots.lock() {
            synced.remove(path);
        }
        if let Ok(mut dirty) = self.dirty_snapshots.lock() {
            dirty.remove(path);
        }
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        if self.fail_next_rename.swap(false, Ordering::AcqRel) {
            return Err(DurableFsError::Io(io::Error::other(
                "injected rename failure",
            )));
        }
        self.inner.rename(from, to)?;
        if let Ok(mut synced) = self.synced_snapshots.lock() {
            if let Some(val) = synced.remove(from) {
                synced.insert(to.to_path_buf(), val);
            }
        }
        Ok(())
    }

    fn list(&self, path: &Path) -> Result<Vec<String>> {
        self.inner.list(path)
    }

    fn exists(&self, path: &Path) -> bool {
        self.inner.exists(path)
    }

    fn len(&self, path: &Path) -> Result<u64> {
        self.inner.len(path)
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        self.inner.create_dir_all(path)
    }
}
