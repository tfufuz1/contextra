// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: Traits und Typen für Purge-Compaction Record Filter und Disk Space Probes.

/// Hook/Filter called during compaction to decide whether a record should be dropped.
pub trait CompactionRecordFilter: Send + Sync {
    /// Returns true if the record with key and transaction sequence/id should be dropped.
    fn drop_record(&self, key: &[u8], tx: u64) -> bool;
}

/// Trait probing available disk space for Purge-Compaction pre-flight checks.
pub trait FreeSpaceProbe: Send + Sync {
    /// Returns available disk space in bytes, or None if space estimation failed/unsupported.
    fn free_space_bytes(&self) -> Option<u64>;
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurgeBlocked {
    /// Active MVCC snapshot pins a sequence number <= as_of_tx.
    ActiveSnapshot { min_seq: u64 },
    /// Checkpoint clones present.
    CheckpointClones { copy_ids: Vec<String> },
    /// Reader is pinned in active memtable / scan.
    ReaderPinned,
    /// Background compaction currently busy.
    CompactionBusy,
    /// Insufficient disk space to write compacted SSTables safely.
    InsufficientDiskSpace { needed: u64, free: u64 },
    /// Unclassified files present in DB directory.
    UnclassifiedFiles { count: usize },
    /// Secure deletion is unsupported on current filesystem/environment.
    SecureDeleteUnsupported { reason: String },
    /// Purge compaction implementation is not yet active (fail-closed).
    Unimplemented,
    /// Scan timeout occurred.
    ScanTimeout,
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
}
