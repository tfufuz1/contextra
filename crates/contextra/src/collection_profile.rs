// FILE-CONTEXT
// ZWECK: CollectionProfile & DeploymentTier Presets for Contextra (Spec B.1.8, AP-P0-05)
// INVARIANTEN: INV-COLLECTION-PROFILE-1, INV-COLLECTION-PROFILE-2

use crate::performance_profile::{PerformanceProfile, PerformanceProfileError};
use contextra_store::kv::delete_mode::KvDeleteMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LsmTuning {
    pub memtable_size_limit: u64,
    pub max_ram_mb: u32,
    pub group_commit_window_micros: u32,
    pub block_cache_shards: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectionProfile {
    pub performance: PerformanceProfile,
    pub kv_delete_mode: KvDeleteMode,
    pub lsm_tuning: LsmTuning,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CollectionProfileError {
    #[error("performance profile invalid: {0}")]
    Performance(#[from] PerformanceProfileError),
    #[error("deletion-proof active but KvDeleteMode::TombstoneOnly cannot satisfy a KV-side DeletionProof")]
    KvDeleteMismatch,
}

impl CollectionProfile {
    pub fn validate(&self) -> Result<(), CollectionProfileError> {
        let resolved = self.performance.resolve();
        resolved.validate()?;
        if resolved.deletion_proof_active && self.kv_delete_mode == KvDeleteMode::TombstoneOnly {
            return Err(CollectionProfileError::KvDeleteMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeploymentTier {
    EdgeMinimal,
    PowerUserLocal,
    EnterpriseShared,
    EnterpriseRegulated,
}

impl DeploymentTier {
    pub fn resolve(self) -> CollectionProfile {
        match self {
            Self::EdgeMinimal => CollectionProfile {
                performance: PerformanceProfile::BareMetal,
                kv_delete_mode: KvDeleteMode::TombstoneOnly,
                lsm_tuning: LsmTuning {
                    memtable_size_limit: 8 << 20,
                    max_ram_mb: 64,
                    group_commit_window_micros: 200,
                    block_cache_shards: 2,
                },
            },
            Self::PowerUserLocal => CollectionProfile {
                performance: PerformanceProfile::Balanced,
                kv_delete_mode: KvDeleteMode::TombstoneOnly,
                lsm_tuning: LsmTuning {
                    memtable_size_limit: 64 << 20,
                    max_ram_mb: 512,
                    group_commit_window_micros: 2_000,
                    block_cache_shards: 8,
                },
            },
            Self::EnterpriseShared => CollectionProfile {
                performance: PerformanceProfile::Balanced,
                kv_delete_mode: KvDeleteMode::CryptoShred,
                lsm_tuning: LsmTuning {
                    memtable_size_limit: 256 << 20,
                    max_ram_mb: 4_096,
                    group_commit_window_micros: 10_000,
                    block_cache_shards: 32,
                },
            },
            Self::EnterpriseRegulated => CollectionProfile {
                performance: PerformanceProfile::Compliance,
                kv_delete_mode: KvDeleteMode::CryptoShred,
                lsm_tuning: LsmTuning {
                    memtable_size_limit: 256 << 20,
                    max_ram_mb: 4_096,
                    group_commit_window_micros: 10_000,
                    block_cache_shards: 32,
                },
            },
        }
    }
}
