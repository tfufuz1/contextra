//! Composition Root wiring module for `contextra-ports` domain primitives.

use crate::{Clock, IdGen, Rng, SystemClock, SystemIdGen, SystemRng};
use contextra_types::ContextraError;
use std::sync::Arc;

/// Composition Root wiring container for canonical `contextra-ports` implementations.
#[derive(Clone)]
pub struct PortsWiring {
    /// Wall-clock and monotonic clock port.
    pub clock: Arc<dyn Clock>,
    /// Pseudorandom generator port.
    pub rng: Arc<dyn Rng>,
    /// Unique ID generator port.
    pub id_gen: Arc<dyn IdGen>,
}

impl PortsWiring {
    /// Builds a default production [`PortsWiring`] instance with canonical system ports.
    pub fn build() -> Result<Self, ContextraError> {
        Ok(Self {
            clock: Arc::new(SystemClock::new()),
            rng: Arc::new(SystemRng::new()),
            id_gen: Arc::new(SystemIdGen::new()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ports_wiring_build() {
        let wiring = PortsWiring::build().expect("PortsWiring build should succeed");
        assert!(wiring.clock.now_unix_nanos() > 0);
        let _u = wiring.rng.next_u64();
        assert!(wiring.id_gen.next_id() > 0);
    }
}
