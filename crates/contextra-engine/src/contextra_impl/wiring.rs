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

    pub fn set_router(&self, router: std::sync::Weak<dyn DriftStatusProvider>) {
        *self.router.write() = Some(router);
    }

    pub fn set_calibrator(
        &self,
        calibrator: std::sync::Weak<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>,
    ) {
        *self.calibrator.write() = Some(calibrator);
    }

    pub fn set_pid_controller(
        &self,
        pid_controller: std::sync::Weak<parking_lot::Mutex<contextra_adapt::PidController>>,
    ) {
        *self.pid_controller.write() = Some(pid_controller);
    }
}
