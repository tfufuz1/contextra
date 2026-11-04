//! SPEC: SPEC §8.2 / D-96
//! Test verifying the visibility and value of MAX_SCAN_MERGE_ACCUMULATOR.

use contextra_ports::MAX_SCAN_MERGE_ACCUMULATOR;

/// SPEC: T-ARCH-6
#[test]
fn test_max_scan_merge_accumulator_visibility() {
    assert_eq!(MAX_SCAN_MERGE_ACCUMULATOR, 100_000);
}
