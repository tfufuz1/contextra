//! # Modul: wiring.rs
//!
//! **Zweck:** Verdrahtung und Initialisierung aller Teilsysteme und Ports.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, ACID/SSI-Transaktionsgarantien.

use crate::*;
use contextra_ports::DriftStatusProvider;

impl Contextra {
    /// Configures or replaces the active [`MetricsSink`].
    pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_ports::MetricsSink>) {
        *self.metrics_sink.write() = sink;
    }

    /// Returns the active [`MetricsSink`].
    pub fn metrics_sink(&self) -> Arc<dyn contextra_ports::MetricsSink> {
        self.metrics_sink.read().clone()
    }

    // ANCHOR[contextra-engine:WIR-001]
    /// Registriert die Router-Komponente.
    pub fn set_router(&self, router: std::sync::Weak<dyn DriftStatusProvider>) {
        *self.router.write() = Some(router);
    }

    // ANCHOR[contextra-engine:WIR-002]
    /// Registriert die Score-Kalibrator-Komponente.
    pub fn set_calibrator(
        &self,
        calibrator: std::sync::Weak<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>,
    ) {
        *self.calibrator.write() = Some(calibrator);
    }
}
