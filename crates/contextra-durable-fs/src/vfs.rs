use crate::error::{DurableFsError, Result};
use crate::platform::{sync_file, FlushKind};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Lokaler Zähler für temporäre Dateinamen bei atomaren Schreibvorgängen.
static ATOMIC_TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Synchroner Trait für virtuelle Dateisystemoperationen (Klasse A).
pub trait Vfs: Send + Sync {
    /// Liest den gesamten Inhalt einer Datei.
    fn read(&self, path: &Path) -> Result<Vec<u8>>;

    /// Schreibt Daten atomar und crash-sicher in eine Datei (`INV-VFS-ATOMIC-1`).
    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>;

    /// Hängt Daten an eine Datei an und flusht synchron.
    fn append(&self, path: &Path, data: &[u8]) -> Result<()>;

    /// Synchronisiert eine Datei auf stabilen Speicher.
    fn sync_file(&self, path: &Path) -> Result<()>;

    /// Synchronisiert die Metadaten eines Verzeichnisses auf stabilen Speicher.
    fn sync_dir(&self, path: &Path) -> Result<()>;

    /// Entfernt eine Datei dauerhaft und flusht das übergeordnete Verzeichnis.
    fn remove(&self, path: &Path) -> Result<()>;

    /// Benennt eine Datei atomar um und flusht das übergeordnete Verzeichnis.
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;

    /// Listet alle Dateinamen in einem Verzeichnis lexikografisch sortiert auf.
    fn list(&self, path: &Path) -> Result<Vec<String>>;

    /// Prüft, ob ein Pfad existiert.
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    /// Gibt die Größe einer Datei in Bytes zurück.
    fn len(&self, path: &Path) -> Result<u64> {
        fs::metadata(path)
            .map(|m| m.len())
            .map_err(|e| map_io_err(e, path))
    }

    /// Erstellt ein Verzeichnis inklusive aller Zwischenverzeichnisse.
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path).map_err(DurableFsError::Io)
    }
}

/// Standard-Implementierung von [`Vfs`] auf Basis von `std::fs`.
#[derive(Debug, Clone, Copy, Default)]
pub struct StdVfs;

/// Alias für bestehende Workspace-Kompatibilität (`RealFs`).
pub type RealFs = StdVfs;

impl Vfs for StdVfs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        fs::read(path).map_err(|e| map_io_err(e, path))
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        crate::atomic_replace(path, data)
    }

    fn append(&self, path: &Path, data: &[u8]) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| map_io_err(e, path))?;
        file.write_all(data).map_err(DurableFsError::Io)?;
        sync_file(&file, FlushKind::DataAndMetadata).map_err(DurableFsError::Io)
    }

    fn sync_file(&self, path: &Path) -> Result<()> {
        let file = File::open(path).map_err(|e| map_io_err(e, path))?;
        sync_file(&file, FlushKind::DataAndMetadata).map_err(DurableFsError::Io)
    }

    fn sync_dir(&self, path: &Path) -> Result<()> {
        crate::sync_dir(path)
    }

    fn remove(&self, path: &Path) -> Result<()> {
        crate::durable_remove(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        crate::rename_durable(from, to)
    }

    fn list(&self, path: &Path) -> Result<Vec<String>> {
        let entries = fs::read_dir(path).map_err(|e| map_io_err(e, path))?;
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(DurableFsError::Io)?;
            if let Some(s) = entry.file_name().to_str() {
                names.push(s.to_string());
            }
        }
        names.sort();
        Ok(names)
    }
}

fn map_io_err(e: std::io::Error, path: &Path) -> DurableFsError {
    if e.kind() == std::io::ErrorKind::NotFound {
        DurableFsError::NotFound {
            path: path.to_path_buf(),
        }
    } else {
        DurableFsError::Io(e)
    }
}

/// RAII-Guard zum automatischen Löschen temporärer Zwischendateien bei Fehlern oder Panics.
pub(crate) struct TmpFileGuard<'a>(pub &'a Path);

impl<'a> Drop for TmpFileGuard<'a> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0);
    }
}

/// Hilfsfunktion zur Erzeugung eines eindeutigen temporären Dateipfads im Zielordner.
pub(crate) fn generate_tmp_path(target: &Path) -> Result<PathBuf> {
    let parent = target.parent().ok_or_else(|| {
        DurableFsError::InvalidArgument("target path has no parent directory".to_string())
    })?;
    let stem = target.file_name().and_then(|s| s.to_str()).unwrap_or("tmp");
    let count = ATOMIC_TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let tmp_name = format!(".{}.tmp.{}.{}", stem, pid, count);
    Ok(parent.join(tmp_name))
}
