//! Manifest Edit-Anwendung, atomares Anhängen und Snapshot-Umschaltung (Ring 1)
//!
//! Stellt atomare Append-Batch-Operationen mit Flush-before-Visible- und fsync-Garantien
//! bereit. Vergiftet das Handle bei fsync-Fehlern sofort fail-closed (INV-FSYNC-1).

use std::sync::atomic::Ordering;
use tokio::io::AsyncWriteExt;

use super::entry::ManifestEntry;
use super::handle::Manifest;
use crate::error::{StoreError, StoreResult};

impl Manifest {
    /// Appends a new entry to the manifest and performs flush + fsync.
    pub async fn append(&self, entry: &ManifestEntry) -> StoreResult<()> {
        self.append_batch(std::slice::from_ref(entry)).await
    }

    /// Appends multiple entries to the manifest in a single atomic batch with flush + fsync.
    pub async fn append_batch(&self, entries: &[ManifestEntry]) -> StoreResult<()> {
        if self.is_poisoned() {
            return Err(StoreError::Storage(format!(
                "Manifest handle for {} is poisoned due to a previous fsync failure (INV-FSYNC-1)",
                self.path.display()
            )));
        }

        if entries.is_empty() {
            return Ok(());
        }
        let mut batch_bytes = Vec::new();
        for entry in entries {
            batch_bytes.extend(entry.to_bytes()?);
        }

        let mut file = self.file.lock().await;
        if let Err(e) = file.write_all(&batch_bytes).await {
            return Err(StoreError::Storage(format!(
                "MANIFEST write failed for {}: {}",
                self.path.display(),
                e
            )));
        }
        if let Err(e) = file.flush().await {
            return Err(StoreError::Storage(format!(
                "MANIFEST flush failed for {}: {}",
                self.path.display(),
                e
            )));
        }
        if let Err(e) = file.sync_all().await {
            self.is_poisoned.store(true, Ordering::SeqCst);
            return Err(StoreError::Storage(format!(
                "MANIFEST fsync failed for {}: {} (handle poisoned, INV-FSYNC-1)",
                self.path.display(),
                e
            )));
        }
        Ok(())
    }
}
