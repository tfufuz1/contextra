// FILE-CONTEXT
// ZWECK: Hintergrund- und Delta-Persistenz für DiskANN (WP-VEC-07).
// INVARIANTEN: Lock-Hierarchie; atomares Ersetzen von Delta-Dateien; fsync auf Verzeichnisebene.
// STAND: 2026-11-11

use super::super::types::DiskAnnIndex;
use crate::error::{Result, VectorError};

impl DiskAnnIndex {
    pub fn trigger_background_persist_delta(&self) {
        if !self.inner.flushing_in_progress.swap(true, Ordering::AcqRel) {
            let index_clone = self.clone();
            self.inner.compute_pool.execute(move || {
                struct FlushGuard(Arc<DiskAnnIndexInner>);
                impl Drop for FlushGuard {
                    fn drop(&mut self) {
                        self.0.flushing_in_progress.store(false, Ordering::Release);
                    }
                }
                let _guard = FlushGuard(Arc::clone(&index_clone.inner));
                if let Err(e) = index_clone.persist_delta_sync() {
                    tracing::error!(error = %e, "DiskANN Hintergrund-persist_delta fehlgeschlagen");
                }
            });
        }
    }

    pub async fn persist_delta(&self) -> Result<()> {
        self.persist_delta_sync()
    }

    pub fn persist_delta_sync(&self) -> Result<()> {
        let pending = {
            let mut guard = self.inner.pending_inserts.write();
            self.inner.pending_count.store(0, Ordering::Relaxed);
            std::mem::take(&mut *guard)
        };

        if pending.is_empty() {
            let pending_wal = self.inner.config.index_path.with_extension("pending.wal");
            if pending_wal.exists() {
                let _ = crate::diskann::io::remove_if_exists(&pending_wal);
            }
            return Ok(());
        }

        let existing_count = self.len_sync();
        let total = existing_count + pending.len();
        let pending_ratio = pending.len() as f64 / total.max(1) as f64;

        if pending_ratio > 0.10 || existing_count == 0 {
            // Vollrebuild: bestehende + pending
            let (mut all_vecs, mut all_ids) = self.load_all_vectors_from_mmap()?;
            for (id, vec) in &pending {
                all_vecs.push(vec.clone());
                all_ids.push(*id);
            }
            return self.build_sync(&all_vecs, &all_ids);
        }

        // Inkrementeller Pfad
        let tmp_path = self.inner.config.index_path.with_extension("delta.tmp");
        self.write_incremental_to_file_sync(&tmp_path, &pending)?;

        // Atomares Rename + Parent-fsync (INV-DISKANN-1, P3)
        let index_path = self.inner.config.index_path.clone();
        crate::diskann::io::atomic_replace(&tmp_path, &index_path)?;
        let pending_wal = self.inner.config.index_path.with_extension("pending.wal");
        crate::diskann::io::remove_if_exists(&pending_wal)?;
        self.load_sync() // Mmap neu laden
    }
}
