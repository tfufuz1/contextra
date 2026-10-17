//! Composition Root wiring module for `contextra-ports` domain primitives.

use crate::{Clock, IdGenerator, SeededRng};
use contextra_types::ContextraError;
use std::sync::Arc;

/// Composition Root wiring container for canonical `contextra-ports` primitives.
#[derive(Clone)]
pub struct PortsWiring {
    /// Wall-clock and monotonic clock port.
    pub clock: Arc<dyn Clock>,
    /// Pseudorandom generator port.
    pub rng: Arc<dyn SeededRng>,
    /// Unique ID generator port.
    pub id_gen: Arc<dyn IdGenerator>,
}

impl PortsWiring {
    /// Creates a new [`PortsWiring`] instance with injected port implementations.
    pub fn new(
        clock: Arc<dyn Clock>,
        rng: Arc<dyn SeededRng>,
        id_gen: Arc<dyn IdGenerator>,
    ) -> Result<Self, ContextraError> {
        Ok(Self { clock, rng, id_gen })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::DeterministicTimestamp;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TestClock;
    impl Clock for TestClock {
        fn now(&self) -> DeterministicTimestamp {
            DeterministicTimestamp::new(1000, 1)
        }
        fn monotonic_ticks(&self) -> u64 {
            1000
        }
    }

    struct TestRng;
    impl SeededRng for TestRng {
        fn next_u64(&self) -> u64 {
            42
        }
        fn fill_bytes(&self, dest: &mut [u8]) {
            dest.fill(7);
        }
        fn next_unit_f64(&self) -> f64 {
            0.5
        }
    }

    struct TestIdGen(AtomicU64);
    impl IdGenerator for TestIdGen {
        fn next_id(&self) -> u64 {
            self.0.fetch_add(1, Ordering::Relaxed)
        }
    }

    #[test]
    fn test_ports_wiring_new() {
        let wiring = PortsWiring::new(
            Arc::new(TestClock),
            Arc::new(TestRng),
            Arc::new(TestIdGen(AtomicU64::new(1))),
        )
        .expect("PortsWiring construction should succeed");

        assert_eq!(wiring.clock.now().wall_ms, 1000);
        assert_eq!(wiring.rng.next_u64(), 42);
        assert_eq!(wiring.id_gen.next_id(), 1);
    }
}
