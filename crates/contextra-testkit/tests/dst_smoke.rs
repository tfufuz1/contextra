// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: DST Smoke test with seed reproducibility.
// INVARIANTEN: Must pass with fixed seed or custom DST_SEED env var.

use contextra_testkit::dst::DstScenario;
use std::env;

#[test]
fn test_dst_smoke() {
    let seed_str = env::var("DST_SEED").unwrap_or_else(|_| "12345".to_string());
    let seed: u64 = seed_str
        .parse()
        .unwrap_or_else(|_| panic!("Invalid DST_SEED value: {}", seed_str));

    println!("Executing DST Smoke Scenario with SEED: {}", seed);

    let result = DstScenario::smoke(seed);

    if !result.success {
        panic!(
            "DST Smoke scenario failed for SEED: {}. Error: {:?}",
            seed, result.error_message
        );
    }

    assert!(
        result.success,
        "DST Smoke test failed for seed {}: {:?}",
        seed, result.error_message
    );
    println!(
        "DST Smoke Scenario PASSED for SEED: {} (Acked commits: {}, Recovered seq: {})",
        seed, result.acked_commits, result.recovered_seq
    );
}
