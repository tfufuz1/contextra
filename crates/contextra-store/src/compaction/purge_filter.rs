// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Traits und Typen für Purge-Compaction Record Filter und Disk Space Probes.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Hook/Filter called during compaction to decide whether a record should be dropped.
pub trait CompactionRecordFilter: Send + Sync {
    /// Returns true if the record with key and transaction sequence/id should be dropped.
    fn drop_record(&self, key: &[u8], tx: u64) -> bool;
}

pub use contextra_ports::erasure::PurgeError;
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    /// Serializes scope into canonical TLV bytes.
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            PurgeScope::Keys(keys) => {
                buf.push(0u8);
                buf.extend_from_slice(&(keys.len() as u64).to_le_bytes());
                for k in keys {
                    buf.extend_from_slice(&(k.len() as u64).to_le_bytes());
                    buf.extend_from_slice(k);
                }
            }
            PurgeScope::Prefixes(prefixes) => {
                buf.push(1u8);
                buf.extend_from_slice(&(prefixes.len() as u64).to_le_bytes());
                for p in prefixes {
                    buf.extend_from_slice(&(p.len() as u64).to_le_bytes());
                    buf.extend_from_slice(p);
                }
            }
        }
        buf
    }

    /// Deserializes scope from canonical TLV bytes.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 9 {
            return Err("scope bytes too short");
        }
        let kind = bytes[0];
        let count = u64::from_le_bytes(bytes[1..9].try_into().map_err(|_| "invalid count")?) as usize;
        let mut offset = 9;
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            if offset + 8 > bytes.len() {
                return Err("truncated item len");
            }
            let len = u64::from_le_bytes(bytes[offset..offset + 8].try_into().map_err(|_| "invalid item len")?) as usize;
            offset += 8;
            if offset + len > bytes.len() {
                return Err("truncated item bytes");
            }
            items.push(bytes[offset..offset + len].to_vec());
            offset += len;
        }
        if offset != bytes.len() {
            return Err("trailing bytes in scope encoding");
        }
        match kind {
            0 => Ok(PurgeScope::Keys(items)),
            1 => Ok(PurgeScope::Prefixes(items)),
            _ => Err("invalid scope kind"),
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

impl PurgeBlocked {
    /// Actionable retry recommendation for the caller.
    #[must_use]
    pub fn retry_hint(&self) -> &'static str {
        match self {
            Self::ActiveSnapshot { .. } => "Close open read transactions / snapshots",
            Self::InsufficientDiskSpace { .. } => "Free disk space or increase storage capacity",
            Self::UnclassifiedFiles { .. } => {
                "Clean up untracked or stale files in storage directory"
            }
            Self::CheckpointClones { .. } => "Purge or release registered checkpoint clones",
            Self::ReaderPinned => "Wait for active reader scan to finish",
            Self::CompactionBusy => "Retry after current background compaction completes",
            Self::SecureDeleteUnsupported { .. } => {
                "Disable scrub requirement or run on supported filesystem"
            }
            Self::Unimplemented => "Enable purge compaction feature",
            Self::ScanTimeout => "Retry with smaller batch size or increased timeout",
            Self::UnknownFreeSpace => {
                "Provide a FreeSpaceProbe implementation or ensure disk space is queryable"
            }
            Self::ReadError { .. } => "Check disk health or file permissions",
            Self::WriteError { .. } => "Check disk space or write permissions",
            Self::ManifestError { .. } => "Check manifest file integrity",
        }
    }
}

impl From<PurgeBlocked> for PurgeError {
    fn from(b: PurgeBlocked) -> Self {
        match b {
            PurgeBlocked::ActiveSnapshot { min_seq } => PurgeError::SnapshotPinned {
                oldest_pin_age: min_seq,
            },
            PurgeBlocked::CheckpointClones { copy_ids } => {
                PurgeError::CheckpointClones { copy_ids }
            }
            PurgeBlocked::ReaderPinned => PurgeError::ReaderPinned,
            PurgeBlocked::CompactionBusy => PurgeError::CompactionBusy,
            PurgeBlocked::InsufficientDiskSpace { needed, free } => {
                PurgeError::InsufficientDiskSpace {
                    needed,
                    available: Some(free),
                }
            }
            PurgeBlocked::UnclassifiedFiles { count } => PurgeError::UnclassifiedFiles { count },
            PurgeBlocked::SecureDeleteUnsupported { reason } => {
                PurgeError::SecureDeleteUnsupported { reason }
            }
            PurgeBlocked::Unimplemented => PurgeError::Unimplemented,
            PurgeBlocked::ScanTimeout => PurgeError::ScanTimeout,
            PurgeBlocked::UnknownFreeSpace => PurgeError::UnknownFreeSpace,
            PurgeBlocked::ReadError {
                file,
                offset,
                details,
            } => PurgeError::ReadError {
                file,
                offset,
                source_details: details,
            },
            PurgeBlocked::WriteError { file, details } => PurgeError::WriteError { file, details },
            PurgeBlocked::ManifestError { details } => PurgeError::Manifest(details),
        }
    }
}

impl From<PurgeError> for PurgeBlocked {
    fn from(e: PurgeError) -> Self {
        match e {
            PurgeError::SnapshotPinned { oldest_pin_age } => PurgeBlocked::ActiveSnapshot {
                min_seq: oldest_pin_age,
            },
            PurgeError::CheckpointClones { copy_ids } => {
                PurgeBlocked::CheckpointClones { copy_ids }
            }
            PurgeError::ReaderPinned => PurgeBlocked::ReaderPinned,
            PurgeError::CompactionBusy => PurgeBlocked::CompactionBusy,
            PurgeError::InsufficientDiskSpace { needed, available } => {
                PurgeBlocked::InsufficientDiskSpace {
                    needed,
                    free: available.unwrap_or(0),
                }
            }
            PurgeError::UnclassifiedFiles { count } => PurgeBlocked::UnclassifiedFiles { count },
            PurgeError::SecureDeleteUnsupported { reason } => {
                PurgeBlocked::SecureDeleteUnsupported { reason }
            }
            PurgeError::Unimplemented => PurgeBlocked::Unimplemented,
            PurgeError::ScanTimeout => PurgeBlocked::ScanTimeout,
            PurgeError::UnknownFreeSpace => PurgeBlocked::UnknownFreeSpace,
            PurgeError::ReadError {
                file,
                offset,
                source_details,
            } => PurgeBlocked::ReadError {
                file,
                offset,
                details: source_details,
            },
            PurgeError::WriteError { file, details } => PurgeBlocked::WriteError { file, details },
            PurgeError::Manifest(details) => PurgeBlocked::ManifestError { details },
            PurgeError::FenceConflict => PurgeBlocked::CompactionBusy,
            PurgeError::Io(io_err) => PurgeBlocked::WriteError {
                file: String::new(),
                details: io_err.to_string(),
            },
            PurgeError::ProbeFailed(_details) => PurgeBlocked::UnknownFreeSpace,
            PurgeError::CorruptedIntent(reason) => PurgeBlocked::ManifestError { details: reason },
            PurgeError::FailpointTriggered(fp) => PurgeBlocked::WriteError {
                file: format!("failpoint: {fp}"),
                details: "failpoint triggered".into(),
            },
        }
    }
}

/// Residual risk factors associated with a physical purge operation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResidualRisk {
    SsdWearLeveling,
    FilesystemJournal,
    CopyOnWrite,
    Backup(String),
    SwapOrPageCache,
    UnscrubbedRemoval(PathBuf),
}

/// Report summarizing results of a successful purge compaction.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PurgeReport {
    /// Number of records/versions dropped.
    pub records_purged: u64,
    /// Total bytes reclaimed.
    pub bytes_reclaimed: u64,
    /// SSTable files removed/replaced.
    pub sstables_replaced: usize,
    /// List of obsolete file paths replaced or made obsolete during purge.
    pub obsolete_files: Vec<PathBuf>,
    /// Residual risks identified during purge operation.
    #[serde(default)]
    pub residual_risks: Vec<ResidualRisk>,
}
