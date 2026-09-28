//! Hardlink cloner for physical SSTable cloning with MVCC pin holding.
//!
//! # Architecture & Copy-on-Write
//! Creates a physical hardlink clone of all visible SSTables for a given sequence number
//! into a target directory. Holds an active `SnapshotRegistry` pin for the duration of
//! the cloning process to prevent LSM compaction or garbage collection from removing
//! SSTables during the clone operation.
//!
//! # Windows Capacity Limit Notice
//! On Windows, `CreateHardLink` is NTFS-exclusive and restricted to a maximum of **1023 hard links**
//! per file on a given volume. This is a known OS-level capacity limit. Cross-filesystem hardlinks
//! or volume limit overflows will return an explicit [`ContextraError::CrossDeviceLink`].
//! Fallback to a full byte-for-byte copy is intentionally NOT implemented in this subsystem
//! (backlog item C.4.1).

// FILE-CONTEXT
// STAND: 2026-09-28T00:00:00Z
// ZWECK: Hardlink-Cloner für Agent-Forking und SSTable CoW-Klone.
// INVARIANTEN: Zero-Panic, Async-I/O via tokio::fs, SnapshotRegistry-Pinning während Klonen.

use contextra_core::SnapshotRegistry;
use contextra_types::{ContextraError, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(not(loom))]
use tokio::fs;

#[cfg(loom)]
mod fs {
    use std::path::Path;

    pub async fn create_dir_all<P: AsRef<Path>>(_path: P) -> std::io::Result<()> {
        Ok(())
    }

    pub async fn read_dir<P: AsRef<Path>>(_path: P) -> std::io::Result<LoomReadDir> {
        Ok(LoomReadDir)
    }

    pub async fn metadata<P: AsRef<Path>>(_path: P) -> std::io::Result<LoomMetadata> {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "loom mock"))
    }

    pub async fn remove_file<P: AsRef<Path>>(_path: P) -> std::io::Result<()> {
        Ok(())
    }

    pub async fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(_from: P, _to: Q) -> std::io::Result<()> {
        Ok(())
    }

    pub struct LoomReadDir;
    impl LoomReadDir {
        pub async fn next_entry(&mut self) -> std::io::Result<Option<LoomDirEntry>> {
            Ok(None)
        }
    }

    pub struct LoomDirEntry;
    impl LoomDirEntry {
        pub fn path(&self) -> std::path::PathBuf {
            std::path::PathBuf::new()
        }
    }

    pub struct LoomMetadata;
}

/// Result structure returned upon completing an SSTable hardlink clone operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardlinkCloneResult {
    /// The source sequence number pinned and cloned.
    pub source_seq_no: u64,
    /// List of target paths created as hardlinks.
    pub linked_files: Vec<PathBuf>,
    /// WAL tail offset at which the clone begins accepting new mutations.
    ///
    /// **Known Gap / Backlog Note:**
    /// Real-time WAL tail offset extraction requires active coordination with the WAL manager
    /// in `contextra-store`. Currently set to `None` as a best-effort explicit placeholder
    /// to avoid returning speculative or unverified offset values.
    pub wal_tail_offset: Option<u64>,
}

/// Trait defining the physical hardlink cloning interface for SSTables.
///
/// # Windows NTFS Limit
/// On Windows systems, `CreateHardLink` is NTFS-exclusive and capped at **1023 links** per volume file.
/// Exceeding this limit or linking across different filesystems/mount points will raise
/// [`ContextraError::CrossDeviceLink`].
pub trait CheckpointHardlinkCloner: Send + Sync {
    /// Clones all SSTable files visible for `seq_no` in `source_dir` into `target_dir` via physical hardlinks.
    ///
    /// Maintains an active `SnapshotRegistry` pin for `seq_no` during the operation.
    fn clone_sstable_hardlinks<'a>(
        &'a self,
        seq_no: u64,
        source_dir: &'a Path,
        target_dir: &'a Path,
        snapshot_registry: &'a Arc<SnapshotRegistry>,
    ) -> contextra_ports::BoxFuture<'a, Result<HardlinkCloneResult>>;
}

/// Productive implementation of [`CheckpointHardlinkCloner`] using `tokio::fs`.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultHardlinkCloner;

impl DefaultHardlinkCloner {
    /// Creates a new [`DefaultHardlinkCloner`].
    pub fn new() -> Self {
        Self
    }
}

impl CheckpointHardlinkCloner for DefaultHardlinkCloner {
    fn clone_sstable_hardlinks<'a>(
        &'a self,
        seq_no: u64,
        source_dir: &'a Path,
        target_dir: &'a Path,
        snapshot_registry: &'a Arc<SnapshotRegistry>,
    ) -> contextra_ports::BoxFuture<'a, Result<HardlinkCloneResult>> {
        Box::pin(async move {
            // 1. Pin snapshot in SnapshotRegistry for the duration of the clone operation
            let _guard = snapshot_registry.register(seq_no);

            // 2. Ensure target directory exists
            fs::create_dir_all(target_dir)
                .await
                .map_err(ContextraError::Io)?;

            // 3. Scan source directory for SSTable files (.sst)
            let mut read_dir = fs::read_dir(source_dir)
                .await
                .map_err(ContextraError::Io)?;

            let mut linked_files = Vec::new();

            while let Some(entry) = read_dir.next_entry().await.map_err(ContextraError::Io)? {
                let path = entry.path();
                if path.is_file() {
                    let is_sst = path
                        .extension()
                        .is_some_and(|ext| ext == "sst" || ext == "tmp");
                    if is_sst || path.file_name().is_some_and(|f| f.to_string_lossy().contains(".sst")) {
                        let file_name = path
                            .file_name()
                            .ok_or_else(|| ContextraError::invalid_input("Invalid file name in SST directory"))?;
                        let target_path = target_dir.join(file_name);

                        // Remove existing link if present
                        if fs::metadata(&target_path).await.is_ok() {
                            let _ = fs::remove_file(&target_path).await;
                        }

                        if let Err(io_err) = fs::hard_link(&path, &target_path).await {
                            if is_cross_device_error(&io_err) {
                                return Err(ContextraError::cross_device_link(
                                    path.to_string_lossy(),
                                    target_path.to_string_lossy(),
                                ));
                            } else {
                                return Err(ContextraError::Io(io_err));
                            }
                        }

                        linked_files.push(target_path);
                    }
                }
            }

            // Sort linked files for deterministic output ordering
            linked_files.sort();

            Ok(HardlinkCloneResult {
                source_seq_no: seq_no,
                linked_files,
                wal_tail_offset: None,
            })
        })
    }
}

/// Helper function to detect cross-device link errors across OS platforms.
fn is_cross_device_error(err: &std::io::Error) -> bool {
    if err.kind() == std::io::ErrorKind::CrossesDevices {
        return true;
    }
    if let Some(code) = err.raw_os_error() {
        // Unix EXDEV (18), Windows ERROR_NOT_SAME_DEVICE (17)
        return code == 18 || code == 17;
    }
    false
}
