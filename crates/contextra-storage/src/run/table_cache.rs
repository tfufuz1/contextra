//! # TableCache: Deskriptor-Verwaltung für offene Dateien
//!
//! Gemäß SPEC §11, §17.1:
//! Hält höchstens `max_open_files` offene Dateien.
//! Integriert mit der Governor-Quote (P08-Schnittstelle).
//! Verdrängung bei Überschreitung des Limits.
//! Keine Integer-Casts (M-02 = 0).

use super::error::RunError;
use contextra_storage_ports::{File, Fs, OpenMode};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Standardwert für maximale offene Dateien (SPEC §6.6: 512).
pub const DEFAULT_MAX_OPEN_FILES: usize = 512;

/// Schnittstelle für Governor-Quoten bezüglich offener Dateideskriptoren.
pub trait FileQuota: Send + Sync {
    /// Versucht, einen Dateideskriptor aus dem Prozesskontingent zu belegen.
    fn acquire_file_fd(&self) -> bool;
    /// Gibt einen belegten Dateideskriptor an das Kontingent zurück.
    fn release_file_fd(&self);
}

/// Standard-Quotenverwaltung, die rein auf der lokalen Höchstgrenze basiert.
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalQuota;

impl FileQuota for LocalQuota {
    fn acquire_file_fd(&self) -> bool {
        true
    }
    fn release_file_fd(&self) {}
}

struct CachedFile {
    id: u64,
    _path: PathBuf,
    file: Arc<dyn File>,
}

struct TableCacheInner {
    files: Vec<CachedFile>,
    lru_order: VecDeque<u64>,
    max_open_files: usize,
}

/// Cache für offene SSTable- und Blob-Dateien.
#[derive(Clone)]
pub struct TableCache {
    fs: Arc<dyn Fs>,
    quota: Arc<dyn FileQuota>,
    inner: Arc<Mutex<TableCacheInner>>,
}

impl TableCache {
    /// Erstellt einen neuen TableCache mit Höchstgrenze und Quotenprüfung.
    pub fn new(fs: Arc<dyn Fs>, max_open_files: usize, quota: Arc<dyn FileQuota>) -> Self {
        let max_open = std::cmp::max(1, max_open_files);
        Self {
            fs,
            quota,
            inner: Arc::new(Mutex::new(TableCacheInner {
                files: Vec::new(),
                lru_order: VecDeque::new(),
                max_open_files: max_open,
            })),
        }
    }

    /// Öffnet oder liefert das offene Datei-Handle für `file_id` am Pfad `path`.
    pub fn get_or_open(&self, file_id: u64, path: &Path) -> Result<Arc<dyn File>, RunError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| RunError::Corrupt("TableCache Sperre vergiftet".into()))?;

        // 1. Bereits im Cache?
        let found = guard
            .files
            .iter()
            .find(|f| f.id == file_id)
            .map(|f| Arc::clone(&f.file));

        if let Some(file) = found {
            touch_lru(&mut guard.lru_order, file_id);
            return Ok(file);
        }

        // 2. Kapazität und Quote sicherstellen
        while guard.files.len() >= guard.max_open_files {
            if let Some(evict_id) = guard.lru_order.pop_front() {
                if let Some(pos) = guard.files.iter().position(|f| f.id == evict_id) {
                    guard.files.remove(pos);
                    self.quota.release_file_fd();
                }
            } else {
                break;
            }
        }

        if !self.quota.acquire_file_fd() {
            return Err(RunError::Limit(
                "Governor-Quote für offene Dateien erschöpft".into(),
            ));
        }

        // 3. Datei über Fs öffnen
        let opened_file = self
            .fs
            .open(path, OpenMode::Read)
            .map_err(|e| {
                self.quota.release_file_fd();
                RunError::Io(e)
            })?;

        let arc_file: Arc<dyn File> = Arc::from(opened_file);
        guard.files.push(CachedFile {
            id: file_id,
            _path: path.to_path_buf(),
            file: Arc::clone(&arc_file),
        });
        guard.lru_order.push_back(file_id);

        Ok(arc_file)
    }

    /// Schließt eine Datei und entfernt sie aus dem Cache (z. B. beim Löschen).
    pub fn close(&self, file_id: u64) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(pos) = guard.files.iter().position(|f| f.id == file_id) {
                guard.files.remove(pos);
                guard.lru_order.retain(|&id| id != file_id);
                self.quota.release_file_fd();
            }
        }
    }

    /// Liefert die Anzahl aktuell offener Dateien im Cache.
    pub fn open_count(&self) -> usize {
        self.inner.lock().map(|g| g.files.len()).unwrap_or(0)
    }
}

fn touch_lru(order: &mut VecDeque<u64>, file_id: u64) {
    order.retain(|&id| id != file_id);
    order.push_back(file_id);
}
