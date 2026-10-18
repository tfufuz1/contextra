// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: DST scenario integration tests covering bitidentical determinism, Torn Write, fsync error, and ENOSPC.
// INVARIANTEN: Bit-identical sequence for identical seed; recovery must preserve invariants across all fault modes.


use contextra_testkit::dst::{DstRunner, DstScenario};

#[test]
fn test_dst_bitidentical_op_sequence() {
    let seed = 12345;
    let ops1 = DstRunner::generate_ops(seed, 100);
    let ops2 = DstRunner::generate_ops(seed, 100);

    assert_eq!(
        ops1, ops2,
        "Same seed MUST yield bitidentical operation sequence"
    );

    let different_seed_ops = DstRunner::generate_ops(54321, 100);
    assert_ne!(
        ops1, different_seed_ops,
        "Different seeds MUST yield different operation sequences"
    );
}

#[test]
fn test_dst_scenario_torn_write() {
    let seed = 9999;
    println!("Executing DST Torn Write Scenario with SEED: {}", seed);

    let result = DstScenario::torn_write(seed, 10);

    if !result.success {
        panic!(
            "DST Torn Write scenario failed for SEED: {}. Error: {:?}",
            seed, result.error_message
        );
    }

    assert!(result.success);
    println!(
        "DST Torn Write Scenario PASSED for SEED: {} (Acked commits: {}, Recovered seq: {})",
        seed, result.acked_commits, result.recovered_seq
    );
}

#[test]
fn test_dst_scenario_fsync_error() {
    let seed = 8888;
    println!("Executing DST fsync Error Scenario with SEED: {}", seed);

    let result = DstScenario::fsync_error(seed, 5);

    if !result.success {
        panic!(
            "DST fsync Error scenario failed for SEED: {}. Error: {:?}",
            seed, result.error_message
        );
    }

    assert!(result.success);
    println!(
        "DST fsync Error Scenario PASSED for SEED: {} (Acked commits: {}, Recovered seq: {})",
        seed, result.acked_commits, result.recovered_seq
    );
}

#[test]
fn test_dst_scenario_enospc() {
    let seed = 7777;
    println!("Executing DST ENOSPC Scenario with SEED: {}", seed);

    let result = DstScenario::enospc(seed, 8);

    if !result.success {
        panic!(
            "DST ENOSPC scenario failed for SEED: {}. Error: {:?}",
            seed, result.error_message
        );
    }

    assert!(result.success);
    println!(
        "DST ENOSPC Scenario PASSED for SEED: {} (Acked commits: {}, Recovered seq: {})",
        seed, result.acked_commits, result.recovered_seq
    );
}
