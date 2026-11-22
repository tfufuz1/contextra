//! # Real Clock
//!
//! Produktive Implementierung des `Clock`-Ports über `std::time`.

use contextra_storage_ports::Clock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Standard-Uhr basierend auf `std::time`.
#[derive(Debug, Clone)]
pub struct RealClock {
    epoch_start: Instant,
}

impl Default for RealClock {
    fn default() -> Self {
        Self {
            epoch_start: Instant::now(),
        }
    }
}

impl RealClock {
    /// Erstellt eine neue `RealClock`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Clock for RealClock {
    fn now_ms(&self) -> u64 {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => u64::try_from(d.as_millis()).unwrap_or(0),
            Err(_) => 0,
        }
    }

    fn monotonic_ns(&self) -> u64 {
        let elapsed = self.epoch_start.elapsed();
        u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_monotonicity() {
        let clock = RealClock::new();
        let t1 = clock.monotonic_ns();
        let t2 = clock.monotonic_ns();
        assert!(t2 >= t1);
        assert!(clock.now_ms() > 1_700_000_000_000);
    }
}
