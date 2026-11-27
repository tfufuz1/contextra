//! # Manifest-Rollover (Ring 1)
//!
//! Schreibt periodisch kompakte Manifest-Snapshots, um Logdateien klein zu halten.

use contextra_types::{ContextraError, Result};
use crate::io_seam::append_file::AppendFile;
use crate::io_seam::StoreIo;
use contextra_ports::VfsOpenOptions;
use std::path::Path;
use std::sync::Arc;
use parking_lot::Mutex;

use super::core::Manifest;

async fn write_durable_temp(io: &StoreIo, path: &Path, contents: Vec<u8>) -> Result<()> {
    let options = VfsOpenOptions {
        write: true,
        create_new: true,
        ..Default::default()
    };
    let file = io.open_async(path, options).await?;
    let file = Arc::new(Mutex::new(AppendFile::new(file, 0)));
    let append_file = Arc::clone(&file);
    tokio::task::spawn_blocking(move || append_file.lock().append_durable(&contents))
        .await
        .map_err(|error| ContextraError::Storage(format!("Temporary MANIFEST write task failed: {error}")))?
        .map_err(|error| ContextraError::Storage(error.to_string()))?;
    let sync_file = Arc::clone(&file);
    tokio::task::spawn_blocking(move || sync_file.lock().sync_all())
        .await
        .map_err(|error| ContextraError::Storage(format!("Temporary MANIFEST sync task failed: {error}")))?
        .map_err(|error| ContextraError::Storage(error.to_string()))?;
    drop(file);
    Ok(())
}

impl Manifest {
    /// Forces an atomic rollover from the state owned by this Manifest.
    ///
    /// Writes live entries into `MANIFEST.new.<pid>.<rand>`, flushes, fsyncs,
    /// atomically renames to `MANIFEST`, and fsyncs the parent directory.
    pub async fn rollover(&self) -> Result<()> {
        let _ = self.maybe_rollover(0).await?;
        Ok(())
    }

    /// Evaluates whether the MANIFEST file size exceeds `threshold_bytes` and performs an atomic
    /// rollover if needed.
    ///
    /// Rollover process:
    /// (a) Write this Manifest's reduced state as a snapshot into a temporary file.
    /// (b) `flush` + `sync_all` on the temporary file.
    /// (c) Atomically `rename` temporary file to `MANIFEST`.
    /// (d) `fsync` the parent directory.
    /// (e) Re-open `MANIFEST` handle in append mode and update `self.file`.
    ///
    /// If an error occurs prior to `rename`, the temporary file is cleaned up and the original `MANIFEST`
    /// remains untouched and valid.
    pub async fn maybe_rollover(&self, threshold_bytes: u64) -> Result<bool> {
        let _operation = self.operation_lock.lock().await;
        let current_len = self.file.lock().committed_len();
        let state_snapshot = self.state.lock().await.clone();
        let snapshot_frame = state_snapshot.snapshot_frame()?;
        if current_len < threshold_bytes {
            return Ok(false);
        }

        tracing::info!(
            manifest_size = current_len,
            threshold_bytes = threshold_bytes,
            "MANIFEST size threshold exceeded; initiating atomic rollover"
        );

        let parent = self
            .path
            .parent()
            .ok_or_else(|| ContextraError::Storage("Invalid MANIFEST parent directory".into()))?;

        let pid = std::process::id();

        const MAX_ATTEMPTS: usize = 100;
        let mut attempt = 0;
        let tmp_path = loop {
            attempt += 1;
            let counter = crate::lsm::recovery::next_temp_file_counter();
            let path = parent.join(format!("MANIFEST.new.{}.{}", pid, counter));
            if !self.io.exists(&path).await? {
                break path;
            }
            if attempt >= MAX_ATTEMPTS {
                return Err(ContextraError::Storage(format!(
                    "Exhausted {} attempts trying to create temporary MANIFEST file {:?}",
                    MAX_ATTEMPTS, path
                )));
            }
        };

        let mut renamed = false;
        let rollover_res: Result<()> = async {
            let mut header = Vec::with_capacity(5);
            header.extend_from_slice(super::core::MANIFEST_HEADER_MAGIC);
            header.push(super::core::CURRENT_MANIFEST_VERSION);
            header.extend_from_slice(&snapshot_frame);

            write_durable_temp(&self.io, &tmp_path, header).await?;

            self.io.rename(&tmp_path, &self.path).await.map_err(|e| {
                ContextraError::Storage(format!(
                    "Failed to rename temporary MANIFEST from {:?} to {:?}: {e}",
                    tmp_path, self.path
                ))
            })?;
            renamed = true;

            if let Some(parent) = self.path.parent() {
                self.io.sync_dir(parent).await?;
            }

            let options = VfsOpenOptions {
                read: true,
                write: true,
                ..Default::default()
            };
            let reopened_file = self.io.open_async(&self.path, options).await?;
            let (reopened_file, len) = tokio::task::spawn_blocking(move || {
                let len = reopened_file.len().map_err(ContextraError::Io)?;
                Ok::<_, ContextraError>((reopened_file, len))
            })
            .await
            .map_err(|error| ContextraError::Storage(format!("Reopen MANIFEST metadata task failed: {error}")))??;

            self.file.lock().replace_file(reopened_file, len);
            Ok(())
        }
        .await;

        if let Err(ref e) = rollover_res {
            if renamed {
                self.is_poisoned.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            tracing::error!(
                path = ?self.path,
                tmp_path = ?tmp_path,
                error = %e,
                "MANIFEST rollover failed"
            );
            if !renamed {
                if let Err(clean_err) = self.io.remove_file(&tmp_path).await {
                    if !matches!(clean_err, ContextraError::NotFound(_)) {
                    tracing::warn!(
                        tmp_path = ?tmp_path,
                        error = %clean_err,
                        "Failed to remove temporary MANIFEST file after failed rollover"
                    );
                    }
                }
            }
            rollover_res?;
        }

        tracing::info!(
            path = ?self.path,
            live_entries_count = state_snapshot.live().len(),
            "MANIFEST rollover completed successfully"
        );

        Ok(true)
    }
}
