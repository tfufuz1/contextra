use crate::compaction::CompactionConfig;
use std::path::PathBuf;
use std::time::Duration;

/// LSM storage configuration.
// SEC-001 — Erweitere LsmConfig um `encryption_passphrase` und AES-256.
// TEST: cargo test -p memfuse-store test_encrypted_db_unreadable_without_key
// DONE: LsmConfig akzeptiert Passphrase, AES-256 wird für Disk-I/O verwendet.
#[derive(Clone, Debug)]
/// Configuration for the LSM storage engine.
pub struct LsmConfig {
    /// Path to the data directory.
    pub path: PathBuf,
    /// Maximum size of the memtable before flushing to disk.
    pub memtable_size_limit: usize,
    /// Maximum RAM usage for the storage engine in MB.
    pub max_ram_mb: u64,
    /// Timeout for transactions in the buffer.
    pub tx_timeout: Duration,
    /// Configuration for background compaction.
    pub compaction: CompactionConfig,
    pub encryption_passphrase: Option<String>,
    /// Time window in microseconds to batch concurrent WAL commits before issuing fsync.
    /// Set to 0 to disable group commit batching (immediate single commit).
    pub group_commit_window_micros: u64,
    /// Number of shards for the block cache.
    /// Default is 64 (increased from 16 to reduce lock contention during concurrent BM25 range scans).
    pub block_cache_shards: usize,
}

impl Default for LsmConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("memfuse_data"),
            memtable_size_limit: 64 * 1024 * 1024,
            max_ram_mb: 2048,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            group_commit_window_micros: 500,
            block_cache_shards: 64,
        }
    }
}
