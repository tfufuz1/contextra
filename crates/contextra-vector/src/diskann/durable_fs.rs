// FILE-CONTEXT
// STAND: 2026-10-06
// ZWECK: Durable Filesystem Operations (APM-1) für DiskANN Index Persistence using contextra-durable-fs.

use contextra_core::{ContextraError, Result};
use std::path::Path;

/// Atomically replaces target file `path` with `bytes` adhering to APM-1 durability rules via `contextra_durable_fs::atomic_replace`.
pub fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<()> {
    contextra_durable_fs::atomic_replace(path, bytes).map_err(|e| {
        ContextraError::Storage(format!("Failed atomic replace for DiskANN index file: {e}"))
    })
}

/// Durably removes `path` and fsyncs its parent directory using `contextra_durable_fs::durable_remove`.
pub fn durable_remove(path: &Path) -> Result<()> {
    if path.exists() {
        contextra_durable_fs::durable_remove(path).map_err(|e| {
            ContextraError::Storage(format!("Failed durable remove for DiskANN file: {e}"))
        })?;
    }
    Ok(())
}

/// Durably syncs the parent directory of `path` via `contextra_durable_fs::sync_dir`.
pub fn sync_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        contextra_durable_fs::sync_dir(parent).map_err(|e| {
            ContextraError::Storage(format!("Failed to sync parent directory: {e}"))
        })?;
    }
    Ok(())
}
