use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::Clock;

/// Runs the complete conformance test suite for a [`Clock`] implementation.
///
/// # Contract Requirements Tested
/// 1. `monotonic_nanos()` is non-decreasing across consecutive readings ($t_2 \ge t_1$).
/// 2. `now_unix_nanos()` returns a valid non-zero timestamp.
pub fn run_clock_suite<C: Clock>(clock: &C) -> ConformanceResult {
    let suite = "Clock";

    let t1 = clock.monotonic_nanos();
    let t2 = clock.monotonic_nanos();
    if t2 < t1 {
        return Err(ConformanceFailure::new(
            suite,
            "monotonic_non_decreasing",
            format!("monotonic clock decreased from {t1} to {t2}"),
        ));
    }

    let unix = clock.now_unix_nanos();
    if unix == 0 {
        return Err(ConformanceFailure::new(
            suite,
            "valid_unix_nanos",
            "now_unix_nanos returned 0",
        ));
    }

    Ok(())
}
