use crate::compaction::CompactionConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DurabilityMode {
    Full,
    WalNoHmac,
    MemoryOnly,
}

impl Default for DurabilityMode {
    fn default() -> Self {
        Self::Full
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DurabilityConfigError {
    #[error("DurabilityMode::{mode:?} is incompatible with feature '{feature}': {reason}")]
    IncompatibleCombination {
        mode: DurabilityMode,
        feature: &'static str,
        reason: &'static str,
    },
}

impl From<DurabilityConfigError> for contextra_types::ContextraError {
    fn from(err: DurabilityConfigError) -> Self {
        contextra_types::ContextraError::DurabilityConfig(err.to_string())
    }
}

impl DurabilityMode {
    pub fn validate_against_features(
        self,
        deletion_proof_active: bool,
        feature_ring: contextra_ports::license::FeatureRing,
    ) -> Result<(), DurabilityConfigError> {
        use DurabilityMode::*;
        match self {
            Full => Ok(()),
            WalNoHmac if deletion_proof_active => {
                Err(DurabilityConfigError::IncompatibleCombination {
                    mode: self,
                    feature: concat!("deletion", "-", "proof"),
                    reason: "no HMAC integrity chain to anchor the proof",
                })
            }
            MemoryOnly
                if deletion_proof_active
                    || feature_ring == contextra_ports::license::FeatureRing::Sovereign =>
            {
                Err(DurabilityConfigError::IncompatibleCombination {
                    mode: self,
                    feature: concat!("deletion", "-", "proof / FeatureRing::Sovereign"),
                    reason: "no persistence layer exists to prove deletion from",
                })
            }
            _ => Ok(()),
        }
    }
}

/// Source for Key Encryption Key (KEK).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum KekSource {
    Passphrase(String),
    Environment(String),
    File(PathBuf),
}

impl std::fmt::Debug for KekSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passphrase(_) => write!(f, "Passphrase(***)"),
            Self::Environment(var) => f.debug_tuple("Environment").field(var).finish(),
            Self::File(path) => f.debug_tuple("File").field(path).finish(),
        }
    }
}

/// Encryption configuration for LSM storage and WAL.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum EncryptionConfig {
    #[default]
    None,
    Passphrase(String),
    Keyring {
        dir: PathBuf,
        kek_source: KekSource,
    },
}

impl std::fmt::Debug for EncryptionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "EncryptionConfig::None"),
            Self::Passphrase(_) => write!(f, "EncryptionConfig::Passphrase(***)"),
            Self::Keyring { dir, kek_source } => f
                .debug_struct("EncryptionConfig::Keyring")
                .field("dir", dir)
                .field("kek_source", kek_source)
                .finish(),
        }
    }
}

impl EncryptionConfig {
    pub fn is_encrypted(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn passphrase(&self) -> Option<String> {
        match self {
            Self::None => None,
            Self::Passphrase(p) => Some(p.clone()),
            Self::Keyring { kek_source, .. } => match kek_source {
                KekSource::Passphrase(p) => Some(p.clone()),
                KekSource::Environment(var) => std::env::var(var).ok(),
                KekSource::File(path) => {
                    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
                }
            },
        }
    }
}

/// LSM storage configuration.
// SEC-001 — Erweitere LsmConfig um `encryption` und AES-256.
// TEST: cargo test -p contextra-store test_encrypted_db_unreadable_without_key
// DONE: LsmConfig akzeptiert Passphrase, AES-256 wird für Disk-I/O verwendet.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
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
    pub encryption: EncryptionConfig,
    /// Deprecated: Passphrase for AES-256 encryption. Use `encryption` with `EncryptionConfig::Passphrase` instead.
    pub encryption_passphrase: Option<String>,
    /// Time window in microseconds to batch concurrent WAL commits before issuing fsync.
    /// Set to 0 to disable group commit batching (immediate single commit).
    pub group_commit_window_micros: u64,
    /// Number of shards for the block cache.
    /// Default is 64 (increased from 16 to reduce lock contention during concurrent BM25 range scans).
    pub block_cache_shards: usize,
    /// Durability mode for WAL persistence and integrity checks.
    pub durability_mode: DurabilityMode,
    /// Maximum capacity of committed write keys tracked in SSI validator before pruning or rejecting.
    /// Default is 1,000,000 keys.
    pub ssi_max_tracked_keys: usize,
    /// Threshold in bytes to determine whether a payload should be stored as a blob.
    /// Default is `crate::blob::DEFAULT_BLOB_THRESHOLD` (65,536 bytes).
    pub blob_bytes_threshold: u64,
    /// Maximum factor for internal merge set size relative to limit in bounded scans.
    /// Default is `crate::lsm::MAX_INTERNAL_MERGE_ENTRIES_FACTOR` (8).
    pub max_internal_merge_factor: usize,
    #[cfg(any(test, feature = "fault-injection"))]
    #[serde(skip, default)]
    pub fault_plan: std::sync::Arc<crate::wal::WalFaultPlan>,
}

impl LsmConfig {
    pub fn encryption_passphrase(&self) -> Option<String> {
        self.encryption_passphrase
            .clone()
            .or_else(|| self.encryption.passphrase())
    }
}

impl Default for LsmConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("contextra_data"),
            memtable_size_limit: 64 * 1024 * 1024,
            max_ram_mb: 2048,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption: EncryptionConfig::default(),
            encryption_passphrase: None,
            group_commit_window_micros: 500,
            block_cache_shards: 64,
            durability_mode: DurabilityMode::default(),
            ssi_max_tracked_keys: 1_000_000,
            blob_bytes_threshold: crate::blob::DEFAULT_BLOB_THRESHOLD,
            max_internal_merge_factor: crate::lsm::MAX_INTERNAL_MERGE_ENTRIES_FACTOR,
            #[cfg(any(test, feature = "fault-injection"))]
            fault_plan: std::sync::Arc::new(crate::wal::WalFaultPlan::default()),
        }
    }
}
