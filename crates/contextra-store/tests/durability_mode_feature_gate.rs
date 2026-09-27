use contextra_ports::license::FeatureRing;
use contextra_store::{DurabilityConfigError, DurabilityMode};

#[test]
fn test_durability_mode_matrix_all_18_combinations() {
    let modes = [
        DurabilityMode::Full,
        DurabilityMode::WalNoHmac,
        DurabilityMode::MemoryOnly,
    ];
    let deletion_proof_states = [false, true];
    let rings = [
        FeatureRing::Fast,
        FeatureRing::Compliance,
        FeatureRing::Sovereign,
    ];

    let mut tested_cases = 0;

    for &mode in &modes {
        for &del_proof in &deletion_proof_states {
            for &ring in &rings {
                tested_cases += 1;
                let res = mode.validate_against_features(del_proof, ring);

                match mode {
                    DurabilityMode::Full => {
                        assert!(
                            res.is_ok(),
                            "DurabilityMode::Full must be valid for del_proof={}, ring={:?}",
                            del_proof,
                            ring
                        );
                    }
                    DurabilityMode::WalNoHmac => {
                        if del_proof {
                            assert_eq!(
                                res,
                                Err(DurabilityConfigError::IncompatibleCombination {
                                    mode: DurabilityMode::WalNoHmac,
                                    feature: "deletion-proof",
                                    reason: "no HMAC integrity chain to anchor the proof",
                                }),
                                "WalNoHmac with deletion-proof must fail"
                            );
                        } else if ring == FeatureRing::Sovereign {
                            assert_eq!(
                                res,
                                Err(DurabilityConfigError::IncompatibleCombination {
                                    mode: DurabilityMode::WalNoHmac,
                                    feature: "FeatureRing::Sovereign",
                                    reason: "sovereign mode requires full integrity chain",
                                }),
                                "WalNoHmac with Sovereign ring must fail"
                            );
                        } else {
                            assert!(
                                res.is_ok(),
                                "WalNoHmac with del_proof=false and ring={:?} must be Ok",
                                ring
                            );
                        }
                    }
                    DurabilityMode::MemoryOnly => {
                        if del_proof || ring == FeatureRing::Sovereign {
                            assert_eq!(
                                res,
                                Err(DurabilityConfigError::IncompatibleCombination {
                                    mode: DurabilityMode::MemoryOnly,
                                    feature: "deletion-proof / FeatureRing::Sovereign",
                                    reason: "no persistence layer exists to prove deletion from",
                                }),
                                "MemoryOnly with del_proof={} ring={:?} must fail",
                                del_proof,
                                ring
                            );
                        } else {
                            assert!(
                                res.is_ok(),
                                "MemoryOnly with del_proof=false and ring={:?} must be Ok",
                                ring
                            );
                        }
                    }
                    _ => unreachable!("All non-exhaustive DurabilityMode variants covered in test"),
                }
            }
        }
    }

    assert_eq!(tested_cases, 18, "Must test exactly 18 combinations");
}
