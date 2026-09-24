use contextra_core::{ContextraError, Result};
use tokio::io::AsyncWriteExt;

use super::core::Manifest;
use super::entry::ManifestEntry;

impl Manifest {
    /// Evaluates whether the MANIFEST file size exceeds `threshold_bytes` and performs an atomic
    /// rollover if needed.
    ///
    /// Rollover process:
    /// (a) Write current live entries (`live_entries`) as a snapshot into a temporary `MANIFEST.new` file.
    /// (b) `flush` + `sync_all` on the temporary file.
    /// (c) Atomically `rename` temporary file to `MANIFEST`.
    /// (d) `fsync` the parent directory.
    /// (e) Re-open `MANIFEST` handle in append mode and update `self.file`.
    ///
    /// If an error occurs prior to `rename`, the temporary file is cleaned up and the original `MANIFEST`
    /// remains untouched and valid.
    /// Forces an atomic rollover of the MANIFEST file regardless of current size.
    ///
    /// Writes live entries into `MANIFEST.new.<pid>.<rand>`, flushes, fsyncs,
    /// atomically renames to `MANIFEST`, and fsyncs the parent directory.
    pub async fn rollover(&self, live_entries: &[ManifestEntry]) -> Result<()> {
        let _ = self.maybe_rollover(live_entries, 0).await?;
        Ok(())
    }

    /// Evaluates whether the MANIFEST file size exceeds `threshold_bytes` and performs an atomic
    /// rollover if needed.
    ///
    /// Rollover process:
    /// (a) Write current live entries (`live_entries`) as a snapshot into a temporary `MANIFEST.new` file.
    /// (b) `flush` + `sync_all` on the temporary file.
    /// (c) Atomically `rename` temporary file to `MANIFEST`.
    /// (d) `fsync` the parent directory.
    /// (e) Re-open `MANIFEST` handle in append mode and update `self.file`.
    ///
    /// If an error occurs prior to `rename`, the temporary file is cleaned up and the original `MANIFEST`
    /// remains untouched and valid.
    pub async fn maybe_rollover(
        &self,
        live_entries: &[ManifestEntry],
        threshold_bytes: u64,
    ) -> Result<bool> {
        let mut file = self.file.lock().await;
        let metadata = file.metadata().await.map_err(|e| {
            ContextraError::Storage(format!("Failed to get MANIFEST file metadata: {e}"))
        })?;

        if metadata.len() < threshold_bytes {
            return Ok(false);
        }

        tracing::info!(
            manifest_size = metadata.len(),
            threshold_bytes = threshold_bytes,
            "MANIFEST size threshold exceeded; initiating atomic rollover"
        );

        let parent = self
            .path
            .parent()
            .ok_or_else(|| ContextraError::Storage("Invalid MANIFEST parent directory".into()))?;

        let pid = std::process::id();
        let rand_val: u64 = rand::random();
        let tmp_path = parent.join(format!("MANIFEST.new.{}.{}", pid, rand_val));

        let rollover_res: Result<()> = async {
            let mut new_file = tokio::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&tmp_path)
                .await
                .map_err(|e| {
                    ContextraError::Storage(format!(
                        "Failed to create temporary MANIFEST file {:?}: {e}",
                        tmp_path
                    ))
                })?;

            for entry in live_entries {
                let bytes = entry.to_bytes()?;
                new_file.write_all(&bytes).await.map_err(|e| {
                    ContextraError::Storage(format!(
                        "Failed to write entry to temporary MANIFEST {:?}: {e}",
                        tmp_path
                    ))
                })?;
            }

            new_file.flush().await.map_err(|e| {
                ContextraError::Storage(format!(
                    "Failed to flush temporary MANIFEST {:?}: {e}",
                    tmp_path
                ))
            })?;
            new_file.sync_all().await.map_err(|e| {
                ContextraError::Storage(format!(
                    "Failed to fsync temporary MANIFEST {:?}: {e}",
                    tmp_path
                ))
            })?;

            drop(new_file);

            tokio::fs::rename(&tmp_path, &self.path)
                .await
                .map_err(|e| {
                    ContextraError::Storage(format!(
                        "Failed to rename temporary MANIFEST from {:?} to {:?}: {e}",
                        tmp_path, self.path
                    ))
                })?;

            crate::util::fsync_parent_dir(&self.path).await?;

            let reopened_file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .read(true)
                .open(&self.path)
                .await
                .map_err(|e| {
                    ContextraError::Storage(format!(
                        "Failed to reopen MANIFEST after rollover {:?}: {e}",
                        self.path
                    ))
                })?;

            *file = reopened_file;
            Ok(())
        }
        .await;

        if let Err(ref e) = rollover_res {
            tracing::error!(
                path = ?self.path,
                tmp_path = ?tmp_path,
                error = %e,
                "MANIFEST rollover failed"
            );
            if let Err(clean_err) = tokio::fs::remove_file(&tmp_path).await {
                if clean_err.kind() != std::io::ErrorKind::NotFound {
                    tracing::warn!(
                        tmp_path = ?tmp_path,
                        error = %clean_err,
                        "Failed to remove temporary MANIFEST file after failed rollover"
                    );
                }
            }
            rollover_res?;
        }

        tracing::info!(
            path = ?self.path,
            live_entries_count = live_entries.len(),
            "MANIFEST rollover completed successfully"
        );

        Ok(true)
    }
}
