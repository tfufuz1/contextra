//! # Real Fs
//!
//! Produktive Implementierung des `Fs`-Ports über das Standard-Dateisystem (`std::fs`).
//! Unterstützt Rechteverwaltung (`0600`/`0700`), atomare Umbenennungen,
//! synchrone Directory-Syncs und exklusive/geteilte Locks nach SPEC §4, §15.6.

use contextra_storage_ports::{DirEntry, File, Fs, FsCaps, LockGuard, LockMode, OpenMode, Perm};
use std::fs::{self, OpenOptions};
use std::io;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::{FileExt, PermissionsExt};

/// Produktive Dateisystem-Implementierung.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealFs;

impl RealFs {
    /// Erstellt eine neue Instanz von `RealFs`.
    pub fn new() -> Self {
        Self
    }

    /// Prüft das Wurzelverzeichnis per `symlink_metadata` (SPEC §15.6).
    ///
    /// Verifiziert, dass das Wurzelverzeichnis existiert, ein Verzeichnis ist und kein Symlink ist.
    pub fn verify_root(root: &Path) -> io::Result<()> {
        let meta = fs::symlink_metadata(root)?;
        if meta.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Wurzelverzeichnis darf kein Symlink sein (SPEC §15.6)",
            ));
        }
        if !meta.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Wurzelverzeichnis muss ein Verzeichnis sein",
            ));
        }
        Ok(())
    }

    /// Hilfsfunktion zum Auslesen der Dateiberechtigungen (für Tests und Suites ohne direkte fs-Aufrufe, M-03).
    #[cfg(unix)]
    pub fn get_mode(path: &Path) -> io::Result<u32> {
        let meta = fs::metadata(path)?;
        Ok(meta.permissions().mode())
    }

    /// Entropiequelle für `RealEntropy` über `/dev/urandom` (M-03).
    #[cfg(unix)]
    pub(crate) fn fill_entropy(dst: &mut [u8]) -> io::Result<()> {
        use std::io::Read;
        let mut file = fs::File::open("/dev/urandom")?;
        file.read_exact(dst)
    }
}

/// Dateisperren-Wächter auf Basis einer offenen Datei.
pub struct RealLockGuard {
    file: fs::File,
}

impl LockGuard for RealLockGuard {}

impl Drop for RealLockGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

/// Implementierung von `File` über `std::fs::File`.
pub struct RealFile {
    inner: fs::File,
}

impl RealFile {
    /// Hüllt eine `std::fs::File` ein.
    pub fn new(inner: fs::File) -> Self {
        Self { inner }
    }
}

impl File for RealFile {
    #[cfg(unix)]
    fn read_at(&self, buf: &mut [u8], off: u64) -> io::Result<usize> {
        self.inner.read_at(buf, off)
    }

    #[cfg(not(unix))]
    fn read_at(&self, _buf: &mut [u8], _off: u64) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "read_at nur auf Unix-Plattformen unterstützt",
        ))
    }

    #[cfg(unix)]
    fn write_all_at(&self, buf: &[u8], off: u64) -> io::Result<()> {
        self.inner.write_all_at(buf, off)
    }

    #[cfg(not(unix))]
    fn write_all_at(&self, _buf: &[u8], _off: u64) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "write_all_at nur auf Unix-Plattformen unterstützt",
        ))
    }

    fn len(&self) -> io::Result<u64> {
        Ok(self.inner.metadata()?.len())
    }

    fn set_len(&self, len: u64) -> io::Result<()> {
        self.inner.set_len(len)
    }

    fn sync_data(&self) -> io::Result<()> {
        self.inner.sync_data()
    }

    fn sync_all(&self) -> io::Result<()> {
        self.inner.sync_all()
    }
}

impl Fs for RealFs {
    fn create_dir_all(&self, p: &Path, perm: Perm) -> io::Result<()> {
        fs::create_dir_all(p)?;
        #[cfg(unix)]
        if perm == Perm::Private {
            fs::set_permissions(p, fs::Permissions::from_mode(0o700))?;
        }
        Ok(())
    }

    fn open(&self, p: &Path, mode: OpenMode) -> io::Result<Box<dyn File>> {
        let file = match mode {
            OpenMode::Read => OpenOptions::new().read(true).open(p)?,
            OpenMode::ReadWrite => OpenOptions::new().read(true).write(true).open(p)?,
            OpenMode::CreateNew(perm) => {
                let f = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .open(p)?;
                #[cfg(unix)]
                if perm == Perm::Private {
                    f.set_permissions(fs::Permissions::from_mode(0o600))?;
                }
                f
            }
        };
        Ok(Box::new(RealFile::new(file)))
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn remove_file(&self, p: &Path) -> io::Result<()> {
        fs::remove_file(p)
    }

    fn remove_dir(&self, p: &Path) -> io::Result<()> {
        fs::remove_dir(p)
    }

    fn list_dir(&self, p: &Path) -> io::Result<Vec<DirEntry>> {
        let read_dir = fs::read_dir(p)?;
        let mut entries = Vec::new();
        for entry in read_dir {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let len = entry.metadata()?.len();
            entries.push(DirEntry { name, len });
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(entries)
    }

    fn sync_dir(&self, p: &Path) -> io::Result<()> {
        #[cfg(unix)]
        {
            let dir = fs::File::open(p)?;
            dir.sync_all()
        }
        #[cfg(not(unix))]
        {
            let _ = p;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "sync_dir nur auf Unix-Plattformen unterstützt (NZ-6, A-02)",
            ))
        }
    }

    fn lock(&self, p: &Path, mode: LockMode) -> io::Result<Box<dyn LockGuard>> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(p)?;

        #[cfg(unix)]
        {
            let _ = file.set_permissions(fs::Permissions::from_mode(0o600));
        }

        match mode {
            LockMode::Exclusive => file.try_lock()?,
            LockMode::Shared => file.try_lock_shared()?,
        }
        Ok(Box::new(RealLockGuard { file }))
    }

    fn caps(&self) -> FsCaps {
        FsCaps {
            durable_file_sync: cfg!(unix),
            durable_dir_sync: cfg!(unix),
            atomic_rename: true,
            private_perm: cfg!(unix),
            no_follow_root: cfg!(unix),
        }
    }

    fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::hard_link(from, to)
    }
}
