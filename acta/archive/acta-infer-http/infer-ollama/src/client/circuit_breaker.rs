//! Circuit-Breaker-Implementierung mit Dreizustandsautomat (Closed / Open / Half-Open)
//! und injizierter Clock (B11-Determinismus).

use crate::error::LocalClock;
use parking_lot::Mutex;
use std::time::Duration;

/// Expliziter Zustand des Circuit-Breakers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerStatus {
    /// Normalbetrieb: Anfragen werden ungehindert durchgeleitet.
    Closed,
    /// Schutzschaltung aktiv: Anfragen werden fail-closed abgewiesen.
    Open,
    /// Probebetrieb: Eine begrenzte Anzahl an Probe-Anfragen wird zugelassen.
    HalfOpen,
}

/// Interner Zustand des Circuit-Breakers.
#[derive(Debug, Clone, Copy)]
struct BreakerState {
    status: BreakerStatus,
    consecutive_failures: u32,
    open_until_nanos: u64,
}

/// Thread-safe circuit breaker backed by the `Clock` port.
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: u32,
    reset_timeout_nanos: u64,
    state: Mutex<BreakerState>,
}

impl CircuitBreaker {
    /// Erstellt eine neue Instanz des CircuitBreakers mit konfigurierbarem Schwellwert und Reset-Timeout.
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_threshold: failure_threshold.max(1),
            reset_timeout_nanos: u64::try_from(reset_timeout.as_nanos()).unwrap_or(u64::MAX),
            state: Mutex::new(BreakerState {
                status: BreakerStatus::Closed,
                consecutive_failures: 0,
                open_until_nanos: 0,
            }),
        }
    }

    /// Liefert den aktuellen Status des Circuit-Breakers unter Auswertung der Zeit.
    pub fn status(&self, clock: &dyn LocalClock) -> BreakerStatus {
        let mut state = self.state.lock();
        self.update_status_locked(&mut state, clock);
        state.status
    }

    /// Checks if the circuit breaker allows a request.
    /// Returns `Ok(())` if closed/half-open, or `Err(msg)` if open.
    pub fn check(&self, clock: &dyn LocalClock) -> Result<(), String> {
        let mut state = self.state.lock();
        self.update_status_locked(&mut state, clock);

        match state.status {
            BreakerStatus::Closed | BreakerStatus::HalfOpen => Ok(()),
            BreakerStatus::Open => Err(format!(
                "Circuit breaker is OPEN due to {} consecutive failures",
                state.consecutive_failures
            )),
        }
    }

    /// Hilfsfunktion zur Zustandstransition unter Lock.
    fn update_status_locked(&self, state: &mut BreakerState, clock: &dyn LocalClock) {
        if state.status == BreakerStatus::Open {
            let now = clock.monotonic_nanos();
            if now >= state.open_until_nanos {
                state.status = BreakerStatus::HalfOpen;
            }
        }
    }

    /// Records a successful request, resetting consecutive failure counts and transitioning to Closed.
    pub fn record_success(&self) {
        let mut state = self.state.lock();
        state.status = BreakerStatus::Closed;
        state.consecutive_failures = 0;
        state.open_until_nanos = 0;
    }

    /// Records a failed request (transient/network/5xx).
    pub fn record_failure(&self, clock: &dyn LocalClock) {
        let mut state = self.state.lock();
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);

        let now = clock.monotonic_nanos();
        match state.status {
            BreakerStatus::HalfOpen => {
                // Ein einzelner Fehlschlag im HalfOpen Zustand öffnet sofort wieder
                state.status = BreakerStatus::Open;
                state.open_until_nanos = now.saturating_add(self.reset_timeout_nanos);
            }
            BreakerStatus::Closed => {
                if state.consecutive_failures >= self.failure_threshold {
                    state.status = BreakerStatus::Open;
                    state.open_until_nanos = now.saturating_add(self.reset_timeout_nanos);
                }
            }
            BreakerStatus::Open => {
                // Bereits offen; Timeout aktualisieren
                state.open_until_nanos = now.saturating_add(self.reset_timeout_nanos);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct MockClock {
        nanos: AtomicU64,
    }

    impl MockClock {
        fn new(start_nanos: u64) -> Self {
            Self {
                nanos: AtomicU64::new(start_nanos),
            }
        }

        fn advance(&self, duration: Duration) {
            self.nanos
                .fetch_add(duration.as_nanos() as u64, Ordering::SeqCst);
        }
    }

    impl LocalClock for MockClock {
        fn now(&self) -> std::time::SystemTime {
            std::time::UNIX_EPOCH + Duration::from_nanos(self.nanos.load(Ordering::SeqCst))
        }

        fn monotonic_nanos(&self) -> u64 {
            self.nanos.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn test_circuit_breaker_transitions() {
        let clock = MockClock::new(1_000_000_000);
        let timeout = Duration::from_secs(10);
        let cb = CircuitBreaker::new(3, timeout);

        // 1. Initialzustand: Closed
        assert_eq!(cb.status(&clock), BreakerStatus::Closed);
        assert!(cb.check(&clock).is_ok());

        // 2. Zwei Fehler unterhalb des Schwellwerts -> immer noch Closed
        cb.record_failure(&clock);
        cb.record_failure(&clock);
        assert_eq!(cb.status(&clock), BreakerStatus::Closed);
        assert!(cb.check(&clock).is_ok());

        // 3. Dritter Fehler -> Wechsel in Open
        cb.record_failure(&clock);
        assert_eq!(cb.status(&clock), BreakerStatus::Open);
        assert!(cb.check(&clock).is_err());

        // 4. Vor Ablauf des Timeouts -> bleibt Open
        clock.advance(Duration::from_secs(5));
        assert_eq!(cb.status(&clock), BreakerStatus::Open);
        assert!(cb.check(&clock).is_err());

        // 5. Nach Ablauf des Timeouts -> Übergang in HalfOpen, Probe-Aufruf erlaubt
        clock.advance(Duration::from_secs(6));
        assert_eq!(cb.status(&clock), BreakerStatus::HalfOpen);
        assert!(cb.check(&clock).is_ok());

        // 6. Fehlschlag in HalfOpen -> sofort wieder Open
        cb.record_failure(&clock);
        assert_eq!(cb.status(&clock), BreakerStatus::Open);
        assert!(cb.check(&clock).is_err());

        // 7. Erneut Timeout abwarten -> HalfOpen
        clock.advance(Duration::from_secs(10));
        assert_eq!(cb.status(&clock), BreakerStatus::HalfOpen);
        assert!(cb.check(&clock).is_ok());

        // 8. Erfolg in HalfOpen -> Übergang in Closed
        cb.record_success();
        assert_eq!(cb.status(&clock), BreakerStatus::Closed);
        assert!(cb.check(&clock).is_ok());
    }
}
