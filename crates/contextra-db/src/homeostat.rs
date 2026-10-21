// FILE-CONTEXT
// ZWECK: Homeostat Wrapper in contextra-db für Rerank Pool Regulation und Deadline Management.
// INVARIANTEN: Re-exportiert `PoolController`, `RerankPidController` (deprecated) und `RerankDeadline` aus `contextra-adapt`.

pub use contextra_adapt::homeostat::RerankDeadline;
#[allow(deprecated)]
pub use contextra_adapt::homeostat::RerankPidController;
pub use contextra_adapt::pool_controller::{PoolController, PoolControllerConfig};

#[cfg(test)]
#[allow(deprecated, clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_homeostat_reexports_reachability_and_functionality() {
        let deadline = RerankDeadline::new(Duration::from_millis(500));
        let start = std::time::Instant::now();
        assert!(!deadline.deadline_exceeded(start));

        let cfg = PoolControllerConfig::default();
        let mut controller = PoolController::new(cfg).expect("valid pool controller config");
        let pool = controller.update_step(0.1, 120.0);
        assert!(pool > 0);

        let mut deprecated_pid = RerankPidController::new(150.0, 50, 200, 100);
        let pid_pool = deprecated_pid.update(120.0);
        assert!(pid_pool >= 50 && pid_pool <= 200);
    }
}
