// FILE-CONTEXT
// ZWECK: Test matrix of PerformanceProfile x KvDeleteMode combinations (Spec B.1.8)

use contextra::collection_profile::{CollectionProfile, CollectionProfileError, LsmTuning};
use contextra::performance_profile::PerformanceProfile;
use contextra_store::kv::delete_mode::KvDeleteMode;

#[test]
fn test_collection_profile_matrix() {
    let dummy_tuning = LsmTuning {
        memtable_size_limit: 64 << 20,
        max_ram_mb: 512,
        group_commit_window_micros: 2_000,
        block_cache_shards: 8,
    };

    let profiles = [
        PerformanceProfile::Compliance,
        PerformanceProfile::Balanced,
        PerformanceProfile::BareMetal,
    ];

    let delete_modes = [KvDeleteMode::TombstoneOnly, KvDeleteMode::CryptoShred];

    for perf in profiles {
        for kv in delete_modes {
            let profile = CollectionProfile {
                performance: perf,
                kv_delete_mode: kv,
                lsm_tuning: dummy_tuning,
            };

            let res = profile.validate();
            match (perf, kv) {
                (PerformanceProfile::Compliance, KvDeleteMode::TombstoneOnly) => {
                    assert_eq!(
                        res,
                        Err(CollectionProfileError::KvDeleteMismatch),
                        "Expected KvDeleteMismatch for Compliance + TombstoneOnly"
                    );
                }
                _ => {
                    assert!(
                        res.is_ok(),
                        "Expected Ok for {:?} x {:?}, got {:?}",
                        perf,
                        kv,
                        res
                    );
                }
            }
        }
    }
}
