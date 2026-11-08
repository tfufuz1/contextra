//! # LSM-Konfiguration (Ring 1)
//!
//! Konfiguriert Schwellenwerte fuer MemTable-Groesse, Flush-Intervalle und WAL-Pfade.

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
    /// Validates the durability mode against required integrity guarantees and persistence constraints.
    pub fn validate_durability(
        self,
        integrity_chain_required: bool,
        require_persistence: bool,
    ) -> Result<(), DurabilityConfigError> {
        use DurabilityMode::*;
        match self {
            Full => Ok(()),
            WalNoHmac if integrity_chain_required => {
                Err(DurabilityConfigError::IncompatibleCombination {
                    mode: self,
                    feature: "integrity-chain",
                    reason: "no HMAC integrity chain to anchor the proof or audit trail",
                })
            }
            MemoryOnly if integrity_chain_required || require_persistence => {
                Err(DurabilityConfigError::IncompatibleCombination {
                    mode: self,
                    feature: "persistence-required",
                    reason: "no persistence layer exists to prove integrity or durability from",
                })
            }
            _ => Ok(()),
        }
    }

    /// Legacy-Validierung gegen Feature-Flags und Feature-Ringe.
    #[deprecated(
        since = "0.1.0",
        note = "use validate_durability(integrity_chain_required, require_persistence) instead (DEBT-01)"
    )]
    pub fn validate_against_features(
        self,
        deletion_proof_active: bool,
        feature_ring: contextra_ports::license::FeatureRing,
    ) -> Result<(), DurabilityConfigError> {
        let require_persistence =
            feature_ring == contextra_ports::license::FeatureRing::Sovereign;
        self.validate_durability(deletion_proof_active, require_persistence)
            .map_err(|err| match err {
                DurabilityConfigError::IncompatibleCombination { mode, feature, reason } => {
                    if feature == "integrity-chain" {
                        DurabilityConfigError::IncompatibleCombination {
                            mode,
                            feature: "deletion-proof",
                            reason: "no HMAC integrity chain to anchor the proof",
                        }
                    } else if feature == "persistence-required" {
                        DurabilityConfigError::IncompatibleCombination {
                            mode,
                            feature: "deletion-proof / FeatureRing::Sovereign",
                            reason: "no persistence layer exists to prove deletion from",
                        }
                    } else {
                        DurabilityConfigError::IncompatibleCombination { mode, feature, reason }
                    }
                }
            })
    }
}

/// LSM storage configuration.
// SEC-001 — Erweitere LsmConfig um `encryption_passphrase` und AES-256.
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
}

impl Default for LsmConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("contextra_data"),
            memtable_size_limit: 64 * 1024 * 1024,
            max_ram_mb: 2048,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            group_commit_window_micros: 500,
            block_cache_shards: 64,
            durability_mode: DurabilityMode::default(),
            ssi_max_tracked_keys: 1_000_000,
        }
    }
}
