// FILE-CONTEXT
// STAND: 2026-10-06
// ZWECK: Durable Filesystem Operations (APM-1) für HNSW Index Persistence.

use contextra_core::{ContextraError, Result};
use std::path::Path;

/// Atomically replaces target file `dst` with temporary file `src` adhering to APM-1 durability rules:
/// 1. fsync the source file (`src`).
/// 2. Atomically rename `src` to `dst`.
/// 3. fsync the parent directory containing `dst`.
pub fn atomic_replace(src: &Path, dst: &Path) -> Result<()> {
    if let Ok(src_file) = std::fs::File::open(src) {
        src_file
            .sync_all()
            .map_err(|e| ContextraError::Storage(format!("Failed to fsync temporary file: {e}")))?;
    }

    std::fs::rename(src, dst).map_err(|e| {
        ContextraError::Storage(format!(
            "Failed to rename temporary file to destination: {e}"
        ))
    })?;

    if let Some(parent) = dst.parent() {
        let dir = std::fs::File::open(parent)
            .map_err(|e| ContextraError::Storage(format!("Failed to open parent directory: {e}")))?;
        dir.sync_all()
            .map_err(|e| ContextraError::Storage(format!("Failed to fsync parent directory: {e}")))?;
    }

    Ok(())
}

/// Durably removes `path` and fsyncs its parent directory.
pub fn durable_remove(path: &Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path)
            .map_err(|e| ContextraError::Storage(format!("Failed to remove file: {e}")))?;
    }

    if let Some(parent) = path.parent() {
        let dir = std::fs::File::open(parent)
            .map_err(|e| ContextraError::Storage(format!("Failed to open parent directory: {e}")))?;
        dir.sync_all()
            .map_err(|e| ContextraError::Storage(format!("Failed to fsync parent directory: {e}")))?;
    }

    Ok(())
}
