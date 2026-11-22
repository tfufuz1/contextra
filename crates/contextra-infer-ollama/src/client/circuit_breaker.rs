use contextra_ports::Clock;
use parking_lot::Mutex;
use std::time::Duration;

/// Thread-safe circuit breaker backed by the `Clock` port.
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: u32,
    reset_timeout_nanos: u64,
    state: Mutex<BreakerState>,
}

#[derive(Debug, Clone, Copy)]
struct BreakerState {
    consecutive_failures: u32,
    open_until_nanos: u64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            reset_timeout_nanos: u64::try_from(reset_timeout.as_nanos()).unwrap_or(u64::MAX),
            state: Mutex::new(BreakerState {
                consecutive_failures: 0,
                open_until_nanos: 0,
            }),
        }
    }

    /// Checks if the circuit breaker allows a request.
    /// Returns `Ok(())` if closed/half-open, or `Err(msg)` if open.
    pub fn check(&self, clock: &dyn Clock) -> Result<(), String> {
        let mut state = self.state.lock();
        if state.open_until_nanos > 0 {
            let now = clock.monotonic_nanos();
            if now < state.open_until_nanos {
                return Err(format!(
                    "Circuit breaker is OPEN due to {} consecutive failures",
                    state.consecutive_failures
                ));
            } else {
                // Reset window
                state.open_until_nanos = 0;
                state.consecutive_failures = 0;
            }
        }
        Ok(())
    }

    /// Records a successful request, resetting consecutive failure counts.
    pub fn record_success(&self) {
        let mut state = self.state.lock();
        state.consecutive_failures = 0;
        state.open_until_nanos = 0;
    }

    /// Records a failed request (transient/network/5xx).
    pub fn record_failure(&self, clock: &dyn Clock) {
        let mut state = self.state.lock();
        state.consecutive_failures += 1;
        if state.consecutive_failures >= self.failure_threshold {
            let now = clock.monotonic_nanos();
            state.open_until_nanos = now.saturating_add(self.reset_timeout_nanos);
        }
    }
}
