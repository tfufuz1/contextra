use contextra_ports::license::FeatureRing;
use contextra_store::lsm::config::{DurabilityConfigError, DurabilityMode};

#[test]
fn test_durability_mode_feature_gate_matrix() {
    use DurabilityMode::*;

    let durability_modes = [Full, WalNoHmac, MemoryOnly];
    let deletion_proof_states = [false, true];
    let feature_rings = [FeatureRing::Fast, FeatureRing::Sovereign, FeatureRing::Compliance];

    let mut evaluated_count = 0;

    for &mode in &durability_modes {
        for &dp_active in &deletion_proof_states {
            for &ring in &feature_rings {
                evaluated_count += 1;
                let result = mode.validate_against_features(dp_active, ring);

                match (mode, dp_active, ring) {
                    (Full, _, _) => {
                        assert!(
                            result.is_ok(),
                            "Full durability mode should be valid for dp_active={}, ring={:?}",
                            dp_active,
                            ring
                        );
                    }
                    (WalNoHmac, true, _) => {
                        assert_eq!(
                            result,
                            Err(DurabilityConfigError::IncompatibleCombination {
                                mode: WalNoHmac,
                                feature: "deletion-proof",
                                reason: "no HMAC integrity chain to anchor the proof",
                            }),
                            "WalNoHmac with dp_active=true must return IncompatibleCombination error"
                        );
                    }
                    (WalNoHmac, false, _) => {
                        assert!(
                            result.is_ok(),
                            "WalNoHmac with dp_active=false should be valid"
                        );
                    }
                    (MemoryOnly, true, _) | (MemoryOnly, _, FeatureRing::Sovereign) => {
                        assert_eq!(
                            result,
                            Err(DurabilityConfigError::IncompatibleCombination {
                                mode: MemoryOnly,
                                feature: "deletion-proof / FeatureRing::Sovereign",
                                reason: "no persistence layer exists to prove deletion from",
                            }),
                            "MemoryOnly with dp_active={} ring={:?} must return IncompatibleCombination error",
                            dp_active,
                            ring
                        );
                    }
                    (MemoryOnly, false, ring) => {
                        assert!(
                            result.is_ok(),
                            "MemoryOnly with dp_active=false, ring={:?} should be valid",
                            ring
                        );
                    }
                    _ => unreachable!("Exhaustive match arm for non-exhaustive enum"),
                }
            }
        }
    }

    assert_eq!(evaluated_count, 18, "Must evaluate exactly 3x2x3 = 18 matrix combinations");
}
