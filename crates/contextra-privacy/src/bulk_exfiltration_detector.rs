// FILE-CONTEXT
// ZWECK:       Layer-4 Bulk-Exfiltration-Detektor (Volumen/Zeitfenster pro Session)
// INVARIANTEN: Zero-Panic Doctrine: Keinem `.unwrap()` oder `.expect()` in Production Code.
//              Fail-Closed Semantik bei Uhrzeitanomalien oder Fehlerzuständen.
//              Thread-safe Sliding-Window-Zähler pro Session ohne Lock-Kontention via `scc::HashMap`.

use contextra_ports::{Clock, SystemClock};
use parking_lot::Mutex;
use scc::HashMap;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

/// Session identifier for grouping egress rate-limiting buckets.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Outcome of evaluating outbound bulk payload volume against session limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulkExfiltrationOutcome {
    /// Request payload byte volume is within allowed limits for the active time window.
    Allow,
    /// Request cumulative payload byte volume exceeds maximum allowed limit in active window.
    Block {
        /// Total bytes recorded in active sliding time window (including candidate payload).
        window_bytes: usize,
        /// Configured maximum allowed bytes per window.
        limit: usize,
    },
}

#[derive(Debug)]
struct SlidingWindowCounter {
    records: VecDeque<(u64, usize)>,
    current_total: usize,
}

impl SlidingWindowCounter {
    fn new() -> Self {
        Self {
            records: VecDeque::new(),
            current_total: 0,
        }
    }

    /// Prunes expired records outside `window` relative to `now_ticks`.
    ///
    /// Handles timestamp clock anomalies by checking `checked_sub`.
    /// Returns `false` if a clock anomaly (e.g. non-monotonic time regression) is detected.
    fn prune_and_sum(&mut self, now_ticks: u64, window: Duration) -> bool {
        let window_nanos = u64::try_from(window.as_nanos()).unwrap_or(u64::MAX);
        while let Some(&(timestamp, bytes)) = self.records.front() {
            match now_ticks.checked_sub(timestamp) {
                Some(elapsed_nanos) => {
                    if elapsed_nanos > window_nanos {
                        self.records.pop_front();
                        self.current_total = self.current_total.saturating_sub(bytes);
                    } else {
                        break;
                    }
                }
                None => {
                    // Fail-Closed: Monotonic clock anomaly detected (now_ticks < timestamp)
                    return false;
                }
            }
        }
        true
    }

    /// Records candidate payload and checks whether total exceeds `max_bytes_per_window`.
    fn record_and_check(
        &mut self,
        payload_len: usize,
        max_bytes_per_window: usize,
        window: Duration,
        now_ticks: u64,
    ) -> BulkExfiltrationOutcome {
        if !self.prune_and_sum(now_ticks, window) {
            // Clock anomaly fail-closed block
            return BulkExfiltrationOutcome::Block {
                window_bytes: usize::MAX, // UNBOUNDED-OK: sentinel value representing clock anomaly block
                limit: max_bytes_per_window,
            };
        }

        let prospective_total = self.current_total.saturating_add(payload_len);
        if prospective_total > max_bytes_per_window {
            BulkExfiltrationOutcome::Block {
                window_bytes: prospective_total,
                limit: max_bytes_per_window,
            }
        } else {
            self.records.push_back((now_ticks, payload_len));
            self.current_total = prospective_total;
            BulkExfiltrationOutcome::Allow
        }
    }
}

/// Volume and time-window based rate limiter for cloud egress requests (Layer 4).
pub struct BulkExfiltrationDetector {
    pub max_bytes_per_window: usize,
    pub window: Duration,
    clock: Arc<dyn Clock>,
    sessions: HashMap<SessionId, Arc<Mutex<SlidingWindowCounter>>>,
}

impl std::fmt::Debug for BulkExfiltrationDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BulkExfiltrationDetector")
            .field("max_bytes_per_window", &self.max_bytes_per_window)
            .field("window", &self.window)
            .finish_non_exhaustive()
    }
}

impl BulkExfiltrationDetector {
    /// Creates a new `BulkExfiltrationDetector` configured with `max_bytes_per_window` and `window` duration
    /// using the default [`SystemClock`].
    pub fn new(max_bytes_per_window: usize, window: Duration) -> Self {
        Self::new_with_clock(max_bytes_per_window, window, Arc::new(SystemClock::new()))
    }

    /// Creates a new `BulkExfiltrationDetector` with explicit [`Clock`] port injection for testability (P28).
    pub fn new_with_clock(
        max_bytes_per_window: usize,
        window: Duration,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            max_bytes_per_window,
            window,
            clock,
            sessions: HashMap::new(),
        }
    }

    /// Registers `payload_len` bytes for `session` and evaluates whether the volume limit
    /// in the current sliding time window was exceeded.
    ///
    /// Thread-safe for concurrent MCP requests across multiple sessions or tasks.
    pub fn record_and_check(
        &self,
        session: SessionId,
        payload_len: usize,
    ) -> BulkExfiltrationOutcome {
        self.record_and_check_at(session, payload_len, self.clock.monotonic_ticks())
    }

    /// Internal deterministic helper accepting explicit `now_ticks` monotonic timestamp for testability.
    pub fn record_and_check_at(
        &self,
        session: SessionId,
        payload_len: usize,
        now_ticks: u64,
    ) -> BulkExfiltrationOutcome {
        let entry_arc = self
            .sessions
            .entry(session)
            .or_insert_with(|| Arc::new(Mutex::new(SlidingWindowCounter::new())))
            .get()
            .clone();

        let mut counter = entry_arc.lock();
        counter.record_and_check(payload_len, self.max_bytes_per_window, self.window, now_ticks)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_single_request_under_limit_allowed() {
        let detector = BulkExfiltrationDetector::new(1000, Duration::from_secs(60));
        let session = SessionId::from("session_1");

        let outcome = detector.record_and_check(session, 500);
        assert_eq!(outcome, BulkExfiltrationOutcome::Allow);
    }

    #[test]
    fn test_cumulative_requests_exceeding_limit_blocked() {
        let detector = BulkExfiltrationDetector::new(1000, Duration::from_secs(60));
        let session = SessionId::from("session_1");

        assert_eq!(
            detector.record_and_check(session.clone(), 600),
            BulkExfiltrationOutcome::Allow
        );

        let outcome = detector.record_and_check(session, 500);
        assert_eq!(
            outcome,
            BulkExfiltrationOutcome::Block {
                window_bytes: 1100,
                limit: 1000,
            }
        );
    }

    #[test]
    fn test_sliding_window_expiry_resets_volume() {
        let detector = BulkExfiltrationDetector::new(1000, Duration::from_secs(10));
        let session = SessionId::from("session_1");
        let start_ticks = 100_000_000_000u64;

        let out1 = detector.record_and_check_at(session.clone(), 800, start_ticks);
        assert_eq!(out1, BulkExfiltrationOutcome::Allow);

        let out2 = detector.record_and_check_at(
            session.clone(),
            400,
            start_ticks + Duration::from_secs(5).as_nanos() as u64,
        );
        assert_eq!(
            out2,
            BulkExfiltrationOutcome::Block {
                window_bytes: 1200,
                limit: 1000
            }
        );

        let out3 = detector.record_and_check_at(
            session.clone(),
            500,
            start_ticks + Duration::from_secs(11).as_nanos() as u64,
        );
        assert_eq!(out3, BulkExfiltrationOutcome::Allow);
    }

    #[test]
    fn test_session_isolation() {
        let detector = BulkExfiltrationDetector::new(1000, Duration::from_secs(60));
        let s1 = SessionId::from("user_alice");
        let s2 = SessionId::from("user_bob");

        assert_eq!(
            detector.record_and_check(s1.clone(), 900),
            BulkExfiltrationOutcome::Allow
        );

        assert_eq!(
            detector.record_and_check(s2.clone(), 900),
            BulkExfiltrationOutcome::Allow
        );

        assert_eq!(
            detector.record_and_check(s1, 200),
            BulkExfiltrationOutcome::Block {
                window_bytes: 1100,
                limit: 1000
            }
        );
        assert_eq!(
            detector.record_and_check(s2, 100),
            BulkExfiltrationOutcome::Allow
        );
    }

    #[tokio::test]
    async fn test_concurrent_requests_prevent_limit_bypass_race() {
        let detector = Arc::new(BulkExfiltrationDetector::new(1000, Duration::from_secs(60)));
        let session = SessionId::from("concurrent_session");

        let mut handles = Vec::new();
        for _ in 0..10 {
            let det = detector.clone();
            let sess = session.clone();
            handles.push(tokio::spawn(async move { det.record_and_check(sess, 150) }));
        }

        let mut allowed_count = 0;
        let mut blocked_count = 0;

        for h in handles {
            let res = h.await.expect("task join failed");
            match res {
                BulkExfiltrationOutcome::Allow => allowed_count += 1,
                BulkExfiltrationOutcome::Block { .. } => blocked_count += 1,
            }
        }

        assert_eq!(allowed_count, 6);
        assert_eq!(blocked_count, 4);
    }

    #[test]
    fn test_time_regression_fails_closed() {
        let detector = BulkExfiltrationDetector::new(1000, Duration::from_secs(10));
        let session = SessionId::from("time_travel_session");
        let now_ticks = 10_000_000_000u64;

        detector.record_and_check_at(session.clone(), 100, now_ticks);

        let earlier_ticks = 9_000_000_000u64;
        let outcome = detector.record_and_check_at(session, 100, earlier_ticks);

        assert!(matches!(outcome, BulkExfiltrationOutcome::Block { .. }));
    }
}
