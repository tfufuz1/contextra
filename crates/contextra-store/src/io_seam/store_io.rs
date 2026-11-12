//! Gemeinsame asynchrone I/O-Hülle `StoreIo` für Dateizugriffe und Pufferung.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::error::{StoreError, StoreResult as Result};
use super::types::{Vfs, VfsFile, VfsOpenOptions};
use super::real_fs::RealFs;

/// Konkrete asynchrone I/O-Hülle für den LSM-Kern.
#[derive(Clone)]
pub struct StoreIo {
    vfs: Arc<dyn Vfs>,
}

impl Default for StoreIo {
    fn default() -> Self {
        Self {
            vfs: Arc::new(RealFs),
        }
    }
}

impl StoreIo {
    /// Erzeugt ein neues `StoreIo` mit dem angegebenen VFS.
    pub fn new(vfs: Arc<dyn Vfs>) -> Self {
        Self { vfs }
    }

    /// Liefert eine Referenz auf das innere VFS.
    pub fn vfs(&self) -> &Arc<dyn Vfs> {
        &self.vfs
    }

    /// Öffnet eine Datei unter Verwendung von `VfsOpenOptions`.
    pub fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        self.vfs.open(path, opts)
    }

    /// Liest eine Datei vollständig asynchron ein.
    pub async fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.read(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Liest Daten positionell ab `offset` mit Länge `len` asynchron ein.
    pub async fn read_at(&self, path: &Path, offset: u64, len: usize) -> Result<Vec<u8>> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
            let mut buf = vec![0u8; len];
            let file = vfs.open(&p, &VfsOpenOptions::read_only())?;
            let mut read_bytes = 0;
            while read_bytes < len {
                let n = file
                    .read_at(
                        buf.get_mut(read_bytes..).ok_or_else(|| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "buffer underflow",
                            )
                        })?,
                        offset + read_bytes as u64,
                    )
                    .map_err(StoreError::Io)?;
                if n == 0 {
                    return Err(StoreError::Storage(
                        "Unexpected EOF reading block".into(),
                    ));
                }
                read_bytes += n;
            }
            Ok(buf)
        })
        .await
        .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Schreibt Daten atomar und crash-konsistent.
    pub async fn write_atomic(&self, path: &Path, data: Vec<u8>) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.write_atomic(&p, &data))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Erzeugt Verzeichnisse rekursiv.
    pub async fn create_dir_all(&self, path: &Path) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.create_dir_all(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Löscht eine Datei.
    pub async fn remove_file(&self, path: &Path) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.remove_file(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Benennt eine Datei atomar um.
    pub async fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let f = from.to_path_buf();
        let t = to.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.rename(&f, &t))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Prüft auf Existenz einer Datei.
    pub async fn exists(&self, path: &Path) -> Result<bool> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.exists(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Liefert die Dateilänge in Bytes.
    pub async fn len(&self, path: &Path) -> Result<u64> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.len(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Liest Verzeichniseinträge als Pfadliste.
    pub async fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.read_dir(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }

    /// Synchronisiert Verzeichnis-Metadaten.
    pub async fn sync_dir(&self, path: &Path) -> Result<()> {
        let vfs = Arc::clone(&self.vfs);
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || vfs.sync_dir(&p))
            .await
            .map_err(|e| StoreError::Storage(format!("Join error: {}", e)))?
    }
}
