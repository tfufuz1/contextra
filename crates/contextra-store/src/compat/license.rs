//! FeatureRing and license validation bridge for contextra-store (SPEC S-50).

use crate::lsm::config::{DurabilityConfigError, DurabilityMode};
pub use contextra_ports::license::FeatureRing;

impl DurabilityMode {
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
