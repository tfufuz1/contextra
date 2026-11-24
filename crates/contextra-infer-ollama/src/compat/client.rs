//! Brückenerweiterungen für OllamaClient (`with_ports_rng`).

use crate::client::OllamaClient;
use crate::error::LocalRng;
use std::sync::Arc;

impl OllamaClient {
    /// Sets a custom random number generator from contextra_ports.
    pub fn with_ports_rng(mut self, rng: Arc<dyn contextra_ports::Rng>) -> Self {
        struct PortsRngAdapter(Arc<dyn contextra_ports::Rng>);
        impl LocalRng for PortsRngAdapter {
            fn next_u64(&self) -> u64 {
                self.0.next_u64()
            }
        }
        self.rng = Arc::new(PortsRngAdapter(rng));
        self
    }
}
