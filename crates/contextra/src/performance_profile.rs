// FILE-CONTEXT
// ZWECK: PerformanceProfile Presets for Contextra (Spec B.1.6, AP-P0-05)
// INVARIANTEN: INV-PERF-PROFILE-1, INV-PERF-PROFILE-2

use contextra_ports::license::FeatureRing;
use contextra_store::lsm::config::{DurabilityConfigError, DurabilityMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum VectorDeleteMode {
    SynchronousRepair,
    BackgroundRepair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PerformanceProfile {
    Compliance,
    Balanced,
    BareMetal,
}

impl Default for PerformanceProfile {
    fn default() -> Self {
        Self::Compliance
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedProfileConfig {
    pub durability_mode: DurabilityMode,
    pub vector_delete_mode: VectorDeleteMode,
    pub deletion_proof_active: bool,
    pub feature_ring: FeatureRing,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PerformanceProfileError {
    #[error("durability configuration invalid: {0}")]
    Durability(#[from] DurabilityConfigError),
    #[error("DurabilityMode::Full combined with VectorDeleteMode::BackgroundRepair while deletion-proof is active breaks the deletion-proof guarantee for the vector index")]
    DurabilityVectorMismatch,
}

impl PerformanceProfile {
    pub fn resolve(self) -> ResolvedProfileConfig {
        match self {
            Self::Compliance => ResolvedProfileConfig {
                durability_mode: DurabilityMode::Full,
                vector_delete_mode: VectorDeleteMode::SynchronousRepair,
                deletion_proof_active: true,
                feature_ring: FeatureRing::Sovereign,
            },
            Self::Balanced => ResolvedProfileConfig {
                durability_mode: DurabilityMode::WalNoHmac,
                vector_delete_mode: VectorDeleteMode::BackgroundRepair,
                deletion_proof_active: false,
                feature_ring: FeatureRing::Fast,
            },
            Self::BareMetal => ResolvedProfileConfig {
                durability_mode: DurabilityMode::MemoryOnly,
                vector_delete_mode: VectorDeleteMode::BackgroundRepair,
                deletion_proof_active: false,
                feature_ring: FeatureRing::Fast,
            },
        }
    }
}

impl ResolvedProfileConfig {
    pub fn validate(&self) -> Result<(), PerformanceProfileError> {
        self.durability_mode
            .validate_against_features(self.deletion_proof_active, self.feature_ring)?;

        if self.durability_mode == DurabilityMode::Full
            && self.vector_delete_mode == VectorDeleteMode::BackgroundRepair
            && self.deletion_proof_active
        {
            return Err(PerformanceProfileError::DurabilityVectorMismatch);
        }

        Ok(())
    }
}
