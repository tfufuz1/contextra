// FILE-CONTEXT
// ZWECK: Test matrix of PerformanceProfile validation (Spec B.1.8)

use contextra::collection_profile::{
    AutoExtractionMode, CollectionProfile, LsmTuning,
};
use contextra::performance_profile::PerformanceProfile;

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

    for perf in profiles {
        let profile = CollectionProfile {
            performance: perf,
            lsm_tuning: dummy_tuning,
            auto_extraction: AutoExtractionMode::Enabled,
        };

        let res = profile.validate();
        assert!(
            res.is_ok(),
            "Expected Ok for {:?}, got {:?}",
            perf,
            res
        );
    }
}
