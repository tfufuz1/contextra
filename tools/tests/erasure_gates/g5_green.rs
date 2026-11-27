use std::collections::{BTreeMap, BTreeSet};

/// Kein HashMap, kein Instant::now.
fn a(clock: &dyn Clock) -> u64 {
    let quote = '"';
    clock.monotonic_nanos()
}
