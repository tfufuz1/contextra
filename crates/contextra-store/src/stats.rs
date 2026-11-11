//! Storage statistics definitions for contextra-store.

/// Detailed operational and storage statistics.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StorageStats {
    /// Number of SSTable segments.
    pub num_segments: usize,
    /// Total size of all SSTables in bytes.
    pub total_size_bytes: u64,
    /// Total size of memtables in bytes.
    pub memtable_size_bytes: u64,
}
