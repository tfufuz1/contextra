use crate::*;
use contextra_core::DriftStatusProvider;

impl Contextra {
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
