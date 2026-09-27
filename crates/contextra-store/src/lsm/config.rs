use crate::compaction::CompactionConfig;
use contextra_core::ContextraError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Explizite Persistenz-/Durability-Stufe einer Collection oder eines
/// gesamten `ContextraDb`-Deployments. Ersetzt die bisherige implizite
/// Annahme „WAL ist immer an" durch eine bewusste, typisierte Wahl.
///
/// # Kompatibilitätsmatrix (siehe INV-DURABILITY-RING)
/// | Mode        | `deletion-proof` | `FeatureRing::Sovereign` | `encryption-at-rest` |
/// |-------------|:---:|:---:|:---:|
/// | `Full`       | ✅ | ✅ | ✅ |
/// | `WalNoHmac`  | ❌ (kein Integritätsanker für Beweiskette) | ❌ | ✅ |
/// | `MemoryOnly` | ❌ | ❌ | ❌ |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DurabilityMode {
    /// WAL + HMAC-Integritätskette + `fsync` pro Group-Commit.
    Full,
    /// WAL + `fsync`, aber ohne HMAC-Integritätskette.
    WalNoHmac,
    /// Reiner In-Memory-Betrieb. Kein WAL-Append, kein `fsync`.
    MemoryOnly,
}

impl Default for DurabilityMode {
    fn default() -> Self {
        Self::Full
    }
}

/// Fehlerform für ungültige Mode×Feature-Kombinationen.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DurabilityConfigError {
    #[error("DurabilityMode::{mode:?} is incompatible with feature '{feature}': {reason}")]
    IncompatibleCombination {
        mode: DurabilityMode,
        feature: &'static str,
        reason: &'static str,
    },
}

impl From<DurabilityConfigError> for ContextraError {
    fn from(err: DurabilityConfigError) -> Self {
        ContextraError::DurabilityConfig(err.to_string())
    }
}

impl DurabilityMode {
    /// Laufzeit-Prüfung der Kompatibilitätsmatrix (INV-DURABILITY-RING).
    pub fn validate_against_features(
        self,
        deletion_proof_active: bool,
        feature_ring: contextra_ports::license::FeatureRing,
    ) -> Result<(), DurabilityConfigError> {
        use DurabilityMode::*;
        match self {
            Full => Ok(()),
            WalNoHmac => {
                if deletion_proof_active {
                    Err(DurabilityConfigError::IncompatibleCombination {
                        mode: self,
                        feature: "deletion-proof",
                        reason: "no HMAC integrity chain to anchor the proof",
                    })
                } else if feature_ring == contextra_ports::license::FeatureRing::Sovereign {
                    Err(DurabilityConfigError::IncompatibleCombination {
                        mode: self,
                        feature: "FeatureRing::Sovereign",
                        reason: "sovereign mode requires full integrity chain",
                    })
                } else {
                    Ok(())
                }
            }
            MemoryOnly => {
                if deletion_proof_active || feature_ring == contextra_ports::license::FeatureRing::Sovereign {
                    Err(DurabilityConfigError::IncompatibleCombination {
                        mode: self,
                        feature: "deletion-proof / FeatureRing::Sovereign",
                        reason: "no persistence layer exists to prove deletion from",
                    })
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// LSM storage configuration.
// SEC-001 — Erweitere LsmConfig um `encryption_passphrase` und AES-256.
// TEST: cargo test -p contextra-store test_encrypted_db_unreadable_without_key
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
    /// Configured durability mode.
    pub durability_mode: DurabilityMode,
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
        }
    }
}
