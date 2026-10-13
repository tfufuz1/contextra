use crate::*;
use contextra_ports::DriftStatusProvider;
use std::sync::Weak;

/// Engine wiring container holding weak references to external controllers (§20.2 / W3-01).
#[derive(Default)]
pub struct EngineWiring {
    pub router: Option<Weak<dyn DriftStatusProvider>>,
    pub calibrator: Option<Weak<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>>,
    pub pid_controller: Option<Weak<parking_lot::Mutex<contextra_adapt::PidController>>>,
}

impl EngineWiring {
    /// Builds an [`EngineWiring`] instance with configured components.
    pub fn build(
        router: Option<Weak<dyn DriftStatusProvider>>,
        calibrator: Option<Weak<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>>,
        pid_controller: Option<Weak<parking_lot::Mutex<contextra_adapt::PidController>>>,
    ) -> Result<Self, ContextraError> {
        Ok(Self {
            router,
            calibrator,
            pid_controller,
        })
    }
}

/// Extension trait for applying [`EngineWiring`] to a [`Contextra`] instance.
pub trait EngineWiringExt {
    /// Applies component wiring to the engine instance.
    fn apply(&self, wiring: EngineWiring);
}

impl EngineWiringExt for Contextra {
    fn apply(&self, wiring: EngineWiring) {
        if let Some(r) = wiring.router {
            self.set_router(r);
        }
        if let Some(c) = wiring.calibrator {
            self.set_calibrator(c);
        }
        if let Some(p) = wiring.pid_controller {
            self.set_pid_controller(p);
        }
    }
}

impl Contextra {
    /// Configures or replaces the active [`MetricsSink`].
    pub fn set_metrics_sink(&self, sink: Arc<dyn contextra_ports::MetricsSink>) {
        *self.metrics_sink.write() = sink;
    }

    /// Returns the active [`MetricsSink`].
    pub fn metrics_sink(&self) -> Arc<dyn contextra_ports::MetricsSink> {
        self.metrics_sink.read().clone()
    }

    pub fn set_router(&self, router: Weak<dyn DriftStatusProvider>) {
        *self.router.write() = Some(router);
    }

    pub fn set_calibrator(
        &self,
        calibrator: Weak<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>,
    ) {
        *self.calibrator.write() = Some(calibrator);
    }

    pub fn set_pid_controller(
        &self,
        pid_controller: Weak<parking_lot::Mutex<contextra_adapt::PidController>>,
    ) {
        *self.pid_controller.write() = Some(pid_controller);
    }
}
