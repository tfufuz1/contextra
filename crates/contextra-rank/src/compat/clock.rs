//! Clock-Adapter für `contextra_ports::Clock`.

use crate::calibration::{Clock, IsotonicCalibrator};
use std::sync::Arc;

/// Adapter für einen dynamischen `contextra_ports::Clock` Referenzzeiger auf das lokale `Clock`-Trait.
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

/// Extension-Trait für `IsotonicCalibrator` zur Ports-Clock-Konfiguration.
pub trait IsotonicPortsClockExt {
    /// Konfiguriert den Kalibrator mit einer externen `contextra_ports::Clock`.
    fn with_ports_clock(self, clock: Arc<dyn contextra_ports::Clock>) -> Self;
}

impl IsotonicPortsClockExt for IsotonicCalibrator {
    fn with_ports_clock(self, clock: Arc<dyn contextra_ports::Clock>) -> Self {
        self.with_clock(Arc::new(PortsClockAdapter(clock)))
    }
}
