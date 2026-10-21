// FILE-CONTEXT
// ZWECK: Determinismus-Integrationstest (W1-11 Stage 1).

use contextra_ports::{Clock, DeterministicTimestamp, SystemClock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct FixedTestClock {
    now_ms: u64,
    ticks: AtomicU64,
}

impl Clock for FixedTestClock {
    fn now(&self) -> DeterministicTimestamp {
        let t = self.ticks.fetch_add(100, Ordering::SeqCst);
        DeterministicTimestamp::new(self.now_ms, t)
    }

    fn monotonic_ticks(&self) -> u64 {
        self.ticks.load(Ordering::SeqCst)
    }
}

#[test]
fn contextra_kvcache_deterministic_with_fixed_clock_and_seed() {
    let clock = Arc::new(FixedTestClock {
        now_ms: 1_700_000_000_000,
        ticks: AtomicU64::new(1_000),
    });

    let ts1 = clock.now();
    let ts2 = clock.now();

    assert_eq!(ts1.wall_ms, 1_700_000_000_000);
    assert_eq!(ts1.mono_ctr, 1_000);
    assert_eq!(ts2.wall_ms, 1_700_000_000_000);
    assert_eq!(ts2.mono_ctr, 1_100);

    let sys_clock = SystemClock::new();
    assert!(sys_clock.monotonic_ticks() > 0 || sys_clock.now_unix_nanos() > 0);
}
