// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: Traits und Typen für Purge-Compaction Record Filter und Disk Space Probes.

use std::path::PathBuf;
use thiserror::Error;

/// Hook/Filter called during compaction to decide whether a record should be dropped.
pub trait CompactionRecordFilter: Send + Sync {
    /// Returns true if the record with key and transaction sequence/id should be dropped.
    fn drop_record(&self, key: &[u8], tx: u64) -> bool;
}

pub use contextra_ports::{FreeSpaceProbe, ProbeError};

/// Production disk space probe using `statvfs` via `contextra-sys`.
pub struct StatvfsSpaceProbe {
    path: PathBuf,
}

impl StatvfsSpaceProbe {
    /// Creates a new `StatvfsSpaceProbe` monitoring the filesystem at `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl FreeSpaceProbe for StatvfsSpaceProbe {
    fn available_bytes(&self) -> std::result::Result<Option<u64>, ProbeError> {
        Ok(contextra_sys::statvfs_available_bytes(&self.path).ok())
    }
}

/// Purge scope specification.
#[derive(Debug, Clone)]
pub enum PurgeScope {
    /// Delete all versions for an explicit list of keys.
    Keys(Vec<Vec<u8>>),
    /// Delete all versions for a list of key prefixes.
    Prefixes(Vec<Vec<u8>>),
}

impl PurgeScope {
    /// Checks if a given key matches this purge scope.
    pub fn matches_key(&self, key: &[u8]) -> bool {
        match self {
            PurgeScope::Keys(keys) => keys.iter().any(|k| k.as_slice() == key),
            PurgeScope::Prefixes(prefixes) => {
                prefixes.iter().any(|p| key.starts_with(p.as_slice()))
            }
        }
    }
}

/// Reasons why a purge compaction operation is blocked.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PurgeBlocked {
    /// Active MVCC snapshot pins a sequence number <= as_of_tx.
    #[error("Active MVCC snapshot pins min_seq {min_seq}")]
    ActiveSnapshot { min_seq: u64 },
    /// Checkpoint clones present.
    #[error("Checkpoint clones present: {copy_ids:?}")]
    CheckpointClones { copy_ids: Vec<String> },
    /// Reader is pinned in active memtable / scan.
    #[error("Reader pinned in active memtable or scan")]
    ReaderPinned,
    /// Background compaction currently busy.
    #[error("Background compaction currently busy")]
    CompactionBusy,
    /// Insufficient disk space to write compacted SSTables safely.
    #[error("Insufficient disk space: needed {needed}, free {free}")]
    InsufficientDiskSpace { needed: u64, free: u64 },
    /// Unclassified files present in DB directory.
    #[error("Unclassified files present in DB directory: {count}")]
    UnclassifiedFiles { count: usize },
    /// Secure deletion is unsupported on current filesystem/environment.
    #[error("Secure deletion unsupported: {reason}")]
    SecureDeleteUnsupported { reason: String },
    /// Purge compaction implementation is not yet active (fail-closed).
    #[error("Purge compaction implementation unimplemented")]
    Unimplemented,
    /// Scan timeout occurred.
    #[error("Scan timeout occurred during purge")]
    ScanTimeout,
    /// Unknown free space.
    #[error("Unknown free space")]
    UnknownFreeSpace,
    /// Read error during purge compaction.
    #[error("Read error at file {file}, offset {offset}: {details}")]
    ReadError {
        file: String,
        offset: u64,
        details: String,
    },
    /// Write error during purge compaction.
    #[error("Write error at file {file}: {details}")]
    WriteError { file: String, details: String },
    /// Manifest error during purge compaction.
    #[error("Manifest error during purge: {details}")]
    ManifestError { details: String },
}

/// Report summarizing results of a successful purge compaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurgeReport {
    /// Number of records/versions dropped.
    pub records_purged: u64,
    /// Total bytes reclaimed.
    pub bytes_reclaimed: u64,
    /// SSTable files removed/replaced.
    pub sstables_replaced: usize,
    /// List of obsolete file paths replaced or made obsolete during purge.
    pub obsolete_files: Vec<PathBuf>,
}
