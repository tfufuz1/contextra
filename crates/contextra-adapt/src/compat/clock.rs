//! Clock-Adapter und Blanket-Implementierung für `contextra_ports::Clock`.

use crate::clock::Clock;
use std::sync::Arc;

/// Adapter für einen dynamischen `contextra_ports::Clock`-Referenzzeiger auf das lokale `Clock`-Trait.
#[derive(Clone)]
pub struct PortsClockAdapter(pub Arc<dyn contextra_ports::Clock>);

impl Clock for PortsClockAdapter {
    fn now_unix_nanos(&self) -> u64 {
        self.0.now_unix_nanos()
    }

    fn monotonic_nanos(&self) -> u64 {
        self.0.monotonic_nanos()
    }
}
