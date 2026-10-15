// FILE-CONTEXT
// ZWECK: PerformanceProfile Presets for Contextra (Spec B.1.6, EP1)
// INVARIANTEN: PerformanceProfile::default() == PerformanceProfile::Balanced

pub use contextra_ports::license::VectorDeleteMode;
use contextra_ports::erasure::ErasureMode;
use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};
use contextra_store::lsm::config::{DurabilityConfigError, DurabilityMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PerformanceProfile {
    Balanced,
}

impl Default for PerformanceProfile {
    fn default() -> Self {
        Self::Balanced
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedProfileConfig {
    pub durability_mode: DurabilityMode,
    pub vector_delete_mode: VectorDeleteMode,
    pub erasure_mode: ErasureMode,
    pub feature_ring: FeatureRing,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PerformanceProfileError {
    #[error("durability configuration invalid: {0}")]
    Durability(#[from] DurabilityConfigError),
    #[error("DurabilityMode::Full combined with VectorDeleteMode::BackgroundRepair while erasure_mode is active breaks deletion guarantees for vector index")]
    DurabilityVectorMismatch,
    #[error("feature ring {0:?} requires a valid license: {1}")]
    LicenseRequired(FeatureRing, LicenseError),
}

impl PerformanceProfile {
    pub fn resolve(self) -> ResolvedProfileConfig {
        match self {
            Self::Balanced => ResolvedProfileConfig {
                durability_mode: DurabilityMode::WalNoHmac,
                vector_delete_mode: VectorDeleteMode::BackgroundRepair,
                erasure_mode: ErasureMode::Off,
                feature_ring: FeatureRing::Fast,
            },
        }
    }
}

impl ResolvedProfileConfig {
    pub fn enforce_license(&self, gate: &dyn LicenseGate) -> Result<(), PerformanceProfileError> {
        if self.feature_ring != FeatureRing::Fast {
            gate.check_ring(self.feature_ring)
                .map_err(|e| PerformanceProfileError::LicenseRequired(self.feature_ring, e))?;
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), PerformanceProfileError> {
        self.durability_mode
            .validate_against_features(self.erasure_mode != ErasureMode::Off, self.feature_ring)?;

        if self.durability_mode == DurabilityMode::Full
            && self.vector_delete_mode == VectorDeleteMode::BackgroundRepair
            && self.erasure_mode != ErasureMode::Off
        {
            return Err(PerformanceProfileError::DurabilityVectorMismatch);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_profile_default_is_balanced() {
        assert_eq!(PerformanceProfile::default(), PerformanceProfile::Balanced);
    }
}
