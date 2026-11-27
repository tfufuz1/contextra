//! Atomare Pending-Delete-Liste für sichere Datei-Entsorgung (§8, WP-4.1).
//!
//! Persistiert zu löschende Pfade vor dem `remove_file` über `StoreIo::write_atomic`,
//! sodass Crashs zwischen Commit und Unlink/Sync beim nächsten Boot bereinigt werden.

use crate::erasure::journal::ByteReader;
use crate::io_seam::StoreIo;
use contextra_types::ErasureError;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

const PDL_MAGIC: &[u8; 8] = b"CTXPDL01";
const PDL_FILE_NAME: &str = "pending-delete.list";

/// Verwaltet die durabel persistierte Liste ausstehender Dateilöschungen.
pub(crate) struct PendingDelete {
    io: StoreIo,
    data_dir: PathBuf,
    list_path: PathBuf,
    state: Mutex<PendingState>,
    notify: Arc<Notify>,
    retry_limit: u32,
    remaining_count: AtomicU64,
}

struct PendingState {
    /// Relative Pfade (relativ zu `data_dir`).
    entries: BTreeSet<String>,
    retries: u32,
}

impl PendingDelete {
    fn is_safe_relative(path: &Path) -> bool {
        !path.as_os_str().is_empty()
            && !path.is_absolute()
            && path
                .components()
                .all(|component| matches!(component, std::path::Component::Normal(_)))
    }

    /// Öffnet oder initialisiert die Pending-Delete-Liste.
    pub async fn open(
        io: StoreIo,
        erasure_dir: &Path,
        data_dir: PathBuf,
        retry_limit: u32,
    ) -> Result<Self, ErasureError> {
        io.create_dir_all(erasure_dir)
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;
        let list_path = erasure_dir.join(PDL_FILE_NAME);
        let entries = if io
            .exists(&list_path)
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?
        {
            let bytes = io
                .read(&list_path)
                .await
                .map_err(|e| ErasureError::Io(e.to_string()))?;
            Self::decode(&bytes)?
        } else {
            BTreeSet::new()
        };
        let rem = entries.len() as u64;
        Ok(Self {
            io,
            data_dir,
            list_path,
            state: Mutex::new(PendingState {
                entries,
                retries: 0,
            }),
            notify: Arc::new(Notify::new()),
            retry_limit,
            remaining_count: AtomicU64::new(rem),
        })
    }

    pub fn remaining(&self) -> u64 {
        self.remaining_count.load(Ordering::Acquire)
    }

    pub fn notify_handle(&self) -> Arc<Notify> {
        Arc::clone(&self.notify)
    }

    fn to_absolute(&self, relative: &str) -> PathBuf {
        self.data_dir.join(relative)
    }

    fn encode(entries: &BTreeSet<String>) -> Result<Vec<u8>, ErasureError> {
        let mut buf = Vec::with_capacity(16 + entries.len() * 32);
        buf.extend_from_slice(PDL_MAGIC);
        let n = u32::try_from(entries.len())
            .map_err(|_| ErasureError::InvalidTarget("pending-delete list too large"))?;
        buf.extend_from_slice(&n.to_le_bytes());
        for entry in entries {
            let b = entry.as_bytes();
            let len = u16::try_from(b.len())
                .map_err(|_| ErasureError::InvalidTarget("pending-delete path > 64 KiB"))?;
            buf.extend_from_slice(&len.to_le_bytes());
            buf.extend_from_slice(b);
        }
        let crc = crc32fast::hash(&buf);
        buf.extend_from_slice(&crc.to_le_bytes());
        Ok(buf)
    }

    fn decode(bytes: &[u8]) -> Result<BTreeSet<String>, ErasureError> {
        if bytes.is_empty() {
            return Ok(BTreeSet::new());
        }
        if bytes.len() < 16 {
            return Err(ErasureError::Io("pending-delete.list truncated".into()));
        }
        if bytes.get(..8) != Some(PDL_MAGIC.as_slice()) {
            return Err(ErasureError::Io("invalid pending-delete.list magic".into()));
        }
        let payload_end = bytes.len() - 4;
        let payload = bytes
            .get(..payload_end)
            .ok_or_else(|| ErasureError::Io("invalid pending-delete.list".into()))?;
        let crc_bytes: [u8; 4] = bytes
            .get(payload_end..)
            .and_then(|s| s.try_into().ok())
            .ok_or_else(|| ErasureError::Io("missing pending-delete.list crc".into()))?;
        let expected_crc = u32::from_le_bytes(crc_bytes);
        if crc32fast::hash(payload) != expected_crc {
            return Err(ErasureError::Io("pending-delete.list crc mismatch".into()));
        }

        let mut r = ByteReader::new(
            payload
                .get(8..)
                .ok_or_else(|| ErasureError::Io("pending-delete.list header error".into()))?,
        );
        let n = r.read_u32()? as usize;
        let mut set = BTreeSet::new();
        for _ in 0..n {
            let len = r.read_u16()? as usize;
            let pbytes = r.read_bytes(len)?;
            let s = std::str::from_utf8(pbytes)
                .map_err(|_| ErasureError::Io("invalid utf-8 in pending-delete".into()))?;
            let path = Path::new(s);
            if !Self::is_safe_relative(path) {
                return Err(ErasureError::Io(
                    "pending-delete.list contains an unsafe path".into(),
                ));
            }
            set.insert(s.to_string());
        }
        if !r.is_empty() {
            return Err(ErasureError::Io(
                "pending-delete.list contains trailing bytes".into(),
            ));
        }
        Ok(set)
    }

    /// Fügt Pfade atomar zur Liste hinzu, löscht sie physisch, synchronisiert das Verzeichnis
    /// und entfernt erledigte Pfade wieder aus der Liste (§8).
    pub async fn remove_after_commit(&self, paths: &[PathBuf]) -> Result<u32, ErasureError> {
        if paths.is_empty() {
            return Ok(0);
        }
        let mut relative_paths = Vec::with_capacity(paths.len());
        for path in paths {
            let relative = path.strip_prefix(&self.data_dir).map_err(|_| {
                ErasureError::InvalidTarget(
                    "pending-delete path must reside inside the store data directory",
                )
            })?;
            if !Self::is_safe_relative(relative) {
                return Err(ErasureError::InvalidTarget(
                    "pending-delete path must be a safe relative path",
                ));
            }
            relative_paths.push(relative.to_string_lossy().into_owned());
        }
        {
            let mut st = self.state.lock().await;
            for relative in relative_paths {
                st.entries.insert(relative);
            }
            let encoded = Self::encode(&st.entries)?;
            self.io
                .write_atomic(&self.list_path, encoded)
                .await
                .map_err(|e| ErasureError::Io(e.to_string()))?;
            self.remaining_count
                .store(st.entries.len() as u64, Ordering::Release);
        }

        self.drain_pending().await
    }

    /// Versucht, alle aktuell in der Liste stehenden Dateien physisch zu entfernen.
    pub async fn drain_pending(&self) -> Result<u32, ErasureError> {
        let mut st = self.state.lock().await;
        if st.entries.is_empty() {
            self.remaining_count.store(0, Ordering::Release);
            return Ok(0);
        }

        let snapshot: Vec<String> = st.entries.iter().cloned().collect();
        let mut removed_rel = Vec::new();
        let mut parents_to_sync = BTreeSet::new();
        let mut had_error = false;

        for rel in &snapshot {
            let full = self.to_absolute(rel);
            match self.io.remove_file(&full).await {
                Ok(()) | Err(contextra_types::ContextraError::NotFound(_)) => {
                    removed_rel.push(rel.clone());
                    if let Some(parent) = full.parent() {
                        if !parent.as_os_str().is_empty() {
                            parents_to_sync.insert(parent.to_path_buf());
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(path = ?full, error = %e, "pending-delete remove_file failed; keeping entry");
                    had_error = true;
                }
            }
        }

        for parent in parents_to_sync {
            if let Err(e) = self.io.sync_dir(&parent).await {
                tracing::warn!(dir = ?parent, error = %e, "pending-delete sync_dir failed");
                had_error = true;
            }
        }

        let removed_count = removed_rel.len() as u32;
        if !had_error {
            for rel in &removed_rel {
                st.entries.remove(rel);
            }
            if st.entries.is_empty() {
                st.retries = 0;
            }
        } else {
            st.retries = st.retries.saturating_add(1);
            if st.retries >= self.retry_limit {
                tracing::error!(
                    retries = st.retries,
                    remaining = st.entries.len(),
                    "erasure_pending_delete_remaining exceeded retry limit"
                );
            }
            self.notify.notify_one();
        }

        let encoded = Self::encode(&st.entries)?;
        self.io
            .write_atomic(&self.list_path, encoded)
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;
        self.remaining_count
            .store(st.entries.len() as u64, Ordering::Release);

        Ok(removed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_paths_must_be_safe_and_store_relative() {
        assert!(PendingDelete::is_safe_relative(Path::new("sst-1.sst")));
        assert!(!PendingDelete::is_safe_relative(Path::new("../outside")));
        assert!(!PendingDelete::is_safe_relative(Path::new("/outside")));
        assert!(!PendingDelete::is_safe_relative(Path::new("")));
    }

    #[tokio::test]
    async fn test_pending_delete_roundtrip_and_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let erasure_dir = dir.path().join("erasure");
        let io = StoreIo::default();
        let pdl = PendingDelete::open(io.clone(), &erasure_dir, dir.path().to_path_buf(), 10)
            .await
            .unwrap();

        let f1 = dir.path().join("sst-1.sst");
        let f2 = dir.path().join("already-gone.sst");
        std::fs::write(&f1, b"abc").unwrap();

        let removed = pdl.remove_after_commit(&[f1.clone(), f2]).await.unwrap();
        assert_eq!(removed, 2);
        assert!(!f1.exists());
        assert_eq!(pdl.remaining(), 0);
    }
}
