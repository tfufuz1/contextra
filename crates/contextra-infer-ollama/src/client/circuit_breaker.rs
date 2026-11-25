//! Circuit Breaker für Anfragen an Ollama mit den Zuständen `Closed`, `Open` und `HalfOpen`.
//!
//! Die Zeit wird über [`MonotonicClock`] injiziert. Dadurch sind Zustandsübergänge
//! deterministisch testbar. Produktiv wird [`SystemMonotonicClock`] verwendet.
//!
//! Verhalten:
//! - `Closed`: Anfragen werden durchgelassen. Nach `failure_threshold` aufeinanderfolgenden
//!   Fehlern wechselt der Breaker nach `Open`.
//! - `Open`: Anfragen werden abgewiesen (fail-closed), bis `reset_timeout` abgelaufen ist.
//! - `HalfOpen`: Nach Ablauf des Timeouts werden Anfragen wieder durchgelassen. Ein Erfolg
//!   schließt den Breaker, ein Fehler öffnet ihn sofort erneut.

use parking_lot::Mutex;
use std::fmt;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Monotone Zeitquelle in Nanosekunden. Der Startwert ist beliebig, nur Differenzen zählen.
pub trait MonotonicClock: Send + Sync {
    /// Liefert die seit einem festen Ursprung vergangene Zeit in Nanosekunden.
    fn monotonic_nanos(&self) -> u64;
}

/// Produktive Zeitquelle. Der Ursprung ist der erste Aufruf im Prozess.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemMonotonicClock;

static PROCESS_EPOCH: OnceLock<Instant> = OnceLock::new();

impl MonotonicClock for SystemMonotonicClock {
    fn monotonic_nanos(&self) -> u64 {
        let epoch = *PROCESS_EPOCH.get_or_init(Instant::now);
        u64::try_from(epoch.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }
}

/// Zustand des Circuit Breakers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerStatus {
    /// Normalbetrieb: Anfragen werden durchgelassen.
    Closed,
    /// Schutzschaltung aktiv: Anfragen werden abgewiesen.
    Open,
    /// Erholungsphase nach Ablauf des Timeouts.
    HalfOpen,
}

/// Fehler, wenn der Breaker eine Anfrage abweist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CircuitOpenError {
    /// Anzahl aufeinanderfolgender Fehler, die zur Öffnung geführt haben.
    pub consecutive_failures: u32,
}

impl fmt::Display for CircuitOpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "circuit breaker is open after {} consecutive failures",
            self.consecutive_failures
        )
    }
}

impl std::error::Error for CircuitOpenError {}

#[derive(Debug, Clone, Copy)]
struct BreakerState {
    status: BreakerStatus,
    consecutive_failures: u32,
    open_until_nanos: u64,
}

/// Thread-sicherer Circuit Breaker.
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: u32,
    reset_timeout_nanos: u64,
    state: Mutex<BreakerState>,
}

impl CircuitBreaker {
    /// Erstellt einen Breaker. `failure_threshold` wird auf mindestens 1 angehoben.
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

    /// Liefert den aktuellen Zustand unter Berücksichtigung der Zeit.
    pub fn status(&self, clock: &dyn MonotonicClock) -> BreakerStatus {
        let mut state = self.state.lock();
        Self::advance_locked(&mut state, clock);
        state.status
    }

    /// Prüft, ob eine Anfrage durchgelassen wird.
    ///
    /// # Errors
    ///
    /// Liefert [`CircuitOpenError`], solange der Breaker `Open` ist.
    pub fn check(&self, clock: &dyn MonotonicClock) -> Result<(), CircuitOpenError> {
        let mut state = self.state.lock();
        Self::advance_locked(&mut state, clock);
        match state.status {
            BreakerStatus::Closed | BreakerStatus::HalfOpen => Ok(()),
            BreakerStatus::Open => Err(CircuitOpenError {
                consecutive_failures: state.consecutive_failures,
            }),
        }
    }

    /// Verbucht einen Erfolg. Setzt den Breaker auf `Closed` zurück.
    pub fn record_success(&self) {
        let mut state = self.state.lock();
        state.status = BreakerStatus::Closed;
        state.consecutive_failures = 0;
        state.open_until_nanos = 0;
    }

    /// Verbucht einen Fehler (transient, Netzwerk oder 5xx).
    pub fn record_failure(&self, clock: &dyn MonotonicClock) {
        let mut state = self.state.lock();
        Self::advance_locked(&mut state, clock);
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        let now = clock.monotonic_nanos();
        match state.status {
            BreakerStatus::HalfOpen => {
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
                state.open_until_nanos = now.saturating_add(self.reset_timeout_nanos);
            }
        }
    }

    fn advance_locked(state: &mut BreakerState, clock: &dyn MonotonicClock) {
        if state.status == BreakerStatus::Open && clock.monotonic_nanos() >= state.open_until_nanos {
            state.status = BreakerStatus::HalfOpen;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct ManualClock {
        nanos: AtomicU64,
    }

    impl ManualClock {
        fn new(start_nanos: u64) -> Self {
            Self {
                nanos: AtomicU64::new(start_nanos),
            }
        }

        fn advance(&self, by: Duration) {
            let delta = u64::try_from(by.as_nanos()).unwrap_or(u64::MAX);
            self.nanos.fetch_add(delta, Ordering::SeqCst);
        }
    }

    impl MonotonicClock for ManualClock {
        fn monotonic_nanos(&self) -> u64 {
            self.nanos.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn closed_opens_after_threshold_failures() {
        let clock = ManualClock::new(1_000);
        let cb = CircuitBreaker::new(3, Duration::from_secs(10));

        assert_eq!(cb.status(&clock), BreakerStatus::Closed);
        cb.record_failure(&clock);
        cb.record_failure(&clock);
        assert_eq!(cb.status(&clock), BreakerStatus::Closed);
        assert!(cb.check(&clock).is_ok());

        cb.record_failure(&clock);
        assert_eq!(cb.status(&clock), BreakerStatus::Open);
        assert_eq!(
            cb.check(&clock),
            Err(CircuitOpenError {
                consecutive_failures: 3
            })
        );
    }

    #[test]
    fn open_becomes_half_open_after_timeout() {
        let clock = ManualClock::new(0);
        let cb = CircuitBreaker::new(1, Duration::from_secs(10));
        cb.record_failure(&clock);

        clock.advance(Duration::from_secs(9));
        assert_eq!(cb.status(&clock), BreakerStatus::Open);
        assert!(cb.check(&clock).is_err());

        clock.advance(Duration::from_secs(1));
        assert_eq!(cb.status(&clock), BreakerStatus::HalfOpen);
        assert!(cb.check(&clock).is_ok());
    }

    #[test]
    fn half_open_failure_reopens_immediately() {
        let clock = ManualClock::new(0);
        let cb = CircuitBreaker::new(1, Duration::from_secs(10));
        cb.record_failure(&clock);
        clock.advance(Duration::from_secs(10));
        assert_eq!(cb.status(&clock), BreakerStatus::HalfOpen);

        cb.record_failure(&clock);
        assert_eq!(cb.status(&clock), BreakerStatus::Open);
        assert!(cb.check(&clock).is_err());
    }

    #[test]
    fn half_open_success_closes_and_resets_counter() {
        let clock = ManualClock::new(0);
        let cb = CircuitBreaker::new(2, Duration::from_secs(5));
        cb.record_failure(&clock);
        cb.record_failure(&clock);
        clock.advance(Duration::from_secs(5));
        assert_eq!(cb.status(&clock), BreakerStatus::HalfOpen);

        cb.record_success();
        assert_eq!(cb.status(&clock), BreakerStatus::Closed);

        // Nach dem Schließen braucht es wieder den vollen Schwellwert.
        cb.record_failure(&clock);
        assert_eq!(cb.status(&clock), BreakerStatus::Closed);
    }

    #[test]
    fn failure_after_timeout_without_prior_check_reopens() {
        let clock = ManualClock::new(0);
        let cb = CircuitBreaker::new(1, Duration::from_secs(10));
        cb.record_failure(&clock);
        clock.advance(Duration::from_secs(10));

        // Kein status()/check() vor dem Fehler: der Übergang HalfOpen -> Open muss trotzdem greifen.
        cb.record_failure(&clock);
        assert_eq!(cb.status(&clock), BreakerStatus::Open);
        clock.advance(Duration::from_secs(9));
        assert_eq!(cb.status(&clock), BreakerStatus::Open);
    }

    #[test]
    fn failures_while_open_extend_the_timeout() {
        let clock = ManualClock::new(0);
        let cb = CircuitBreaker::new(1, Duration::from_secs(10));
        cb.record_failure(&clock);

        clock.advance(Duration::from_secs(8));
        cb.record_failure(&clock);
        clock.advance(Duration::from_secs(8));
        assert_eq!(cb.status(&clock), BreakerStatus::Open);

        clock.advance(Duration::from_secs(2));
        assert_eq!(cb.status(&clock), BreakerStatus::HalfOpen);
    }

    #[test]
    fn zero_threshold_is_clamped_to_one() {
        let clock = ManualClock::new(0);
        let cb = CircuitBreaker::new(0, Duration::from_secs(1));
        cb.record_failure(&clock);
        assert_eq!(cb.status(&clock), BreakerStatus::Open);
    }

    #[test]
    fn system_clock_is_monotone() {
        let clock = SystemMonotonicClock;
        let a = clock.monotonic_nanos();
        let b = clock.monotonic_nanos();
        assert!(b >= a);
    }
}
