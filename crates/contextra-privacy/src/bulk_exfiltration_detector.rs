//! `bulk_exfiltration_detector` — Schutz vor Massendatenabfluss (Layer-4 Bulk-Exfiltration-Detektor).
//!
//! # Zweck
//! Überwacht das ausgehende Datenvolumen in gleitenden Zeitfenstern (Sliding Window) pro Sitzung ([`SessionId`]).
//! Verhindert, dass böswillige Akteure oder kompromittierte Agenten große Mengen sensibler Daten schrittweise
//! über Cloud-Abfragen exfiltrieren (Data Loss Prevention, Schutz vor Datenverlust).
//!
//! # Ring-Zugehörigkeit
//! Ring 3 (Domain-Sicherheit).
//!
//! # Invarianten
//! - **Zero-Panic-Doctrine**: Kein `.unwrap()` oder `.expect()` im Produktionscode.
//! - **Fail-Closed**: Bei Zeitanomalien (z. B. nicht-monotonen Zeitsprüngen in die Vergangenheit) wird sofort blockiert.
//! - **Thread-Sicherheit**: Nebenläufiger, lock-armer Zugriff über `scc::HashMap` mit feingranularen Mutexen pro Sitzung.
//! - **Port-Entkopplung**: Zeitstempel ausschließlich über den `Clock`-Port aus `contextra-ports` (INV-EGRESS-AUDIT-1, Regel 3).

use contextra_ports::{Clock, SystemClock};
use parking_lot::Mutex;
use scc::HashMap;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

// ANCHOR[contextra-privacy:BED-001]
/// Eindeutige Sitzungs-Kennung (SessionId) zur Gruppierung von Egress-Rate-Limiting-Buckets.
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

// ANCHOR[contextra-privacy:BED-002]
/// Ergebnis der Auswertung des ausgehenden Datenvolumens gegen die konfigurierten Sitzungslimits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulkExfiltrationOutcome {
    /// Das kumulierte Datenvolumen liegt innerhalb der zulässigen Grenzen des aktiven Zeitfensters.
    Allow,
    /// Das kumulierte Datenvolumen überschreitet das Limit im aktiven Zeitfenster; Abfrage wird blockiert.
    Block {
        /// Gesamte im Zeitfenster registrierte Bytes (inklusive des angefragten Payloads).
        window_bytes: usize,
        /// Konfiguriertes maximales Byte-Limit pro Zeitfenster.
        limit: usize,
    },
}

// ANCHOR[contextra-privacy:BED-003]
// Interner Zähler mit gleitendem Zeitfenster (Sliding Window) für eine einzelne Sitzung.
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

    // ANCHOR[contextra-privacy:BED-004]
    // Entfernt abgelaufene Einträge außerhalb von `window_nanos` relativ zu `now_nanos`.
    // Behandelt Zeitanomalien (`now_nanos < timestamp_nanos`) und liefert `false`, falls die Zeit rückwärts lief.
    fn prune_and_sum(&mut self, now_nanos: u64, window_nanos: u64) -> bool {
        while let Some(&(timestamp_nanos, bytes)) = self.records.front() {
            if now_nanos < timestamp_nanos {
                // Fail-Closed: Monotone Zeitanomalie erkannt (Zeitreise)
                return false;
            }
            let elapsed = now_nanos.saturating_sub(timestamp_nanos);
            if elapsed > window_nanos {
                self.records.pop_front();
                self.current_total = self.current_total.saturating_sub(bytes);
            } else {
                break;
            }
        }
        true
    }

    // ANCHOR[contextra-privacy:BED-005]
    // Registriert den Kandidaten-Payload und prüft, ob das Limit überschritten wird.
    fn record_and_check(
        &mut self,
        payload_len: usize,
        max_bytes_per_window: usize,
        window_nanos: u64,
        now_nanos: u64,
    ) -> BulkExfiltrationOutcome {
        if !self.prune_and_sum(now_nanos, window_nanos) {
            // Bei Zeitanomalie sofort blockieren (Fail-Closed)
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
            self.records.push_back((now_nanos, payload_len));
            self.current_total = prospective_total;
            BulkExfiltrationOutcome::Allow
        }
    }
}

// ANCHOR[contextra-privacy:BED-006]
/// Volumen- und Zeitfenster-basierter Rate-Limiter für Cloud-Egress-Anfragen (Layer 4).
pub struct BulkExfiltrationDetector {
    /// Maximal zulässige Bytes pro Zeitfenster.
    pub max_bytes_per_window: usize,
    /// Dauer des gleitenden Zeitfensters.
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
    // ANCHOR[contextra-privacy:BED-007]
    /// Erstellt einen neuen `BulkExfiltrationDetector` mit Standard-Systemzeit-Clock.
    ///
    /// # Parameter
    /// * `max_bytes_per_window`: Maximale Byteanzahl im Fenster.
    /// * `window`: Dauer des Fensters als [`Duration`].
    pub fn new(max_bytes_per_window: usize, window: Duration) -> Self {
        Self::with_clock(max_bytes_per_window, window, Arc::new(SystemClock::new()))
    }

    // ANCHOR[contextra-privacy:BED-008]
    /// Erstellt einen neuen `BulkExfiltrationDetector` mit einem spezifischen [`Clock`]-Port.
    ///
    /// # Parameter
    /// * `max_bytes_per_window`: Maximale Byteanzahl im Fenster.
    /// * `window`: Dauer des Fensters.
    /// * `clock`: Trait-Objekt für monotone Zeitmessung.
    pub fn with_clock(
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

    // ANCHOR[contextra-privacy:BED-009]
    /// Registriert `payload_len` Bytes für `session` und bewertet, ob das Limit im gleitenden Fenster überschritten wurde.
    ///
    /// Thread-sicher für nebenläufige Anfragen über mehrere Sitzungen hinweg.
    ///
    /// # Nebenwirkungen
    /// Liest monotone Zeit über den internen `Clock`-Port.
    #[allow(clippy::disallowed_methods)] // ADR-098: Monotonic sliding window timestamp in BulkExfiltrationDetector
    pub fn record_and_check(
        &self,
        session: SessionId,
        payload_len: usize,
    ) -> BulkExfiltrationOutcome {
        self.record_and_check_with_clock(session, payload_len, self.clock.as_ref())
    }

    // ANCHOR[contextra-privacy:BED-010]
    /// Registriert Bytes für eine Sitzung unter Verwendung einer explizit übergebenen [`Clock`].
    pub fn record_and_check_with_clock(
        &self,
        session: SessionId,
        payload_len: usize,
        clock: &dyn Clock,
    ) -> BulkExfiltrationOutcome {
        self.record_and_check_at_nanos(session, payload_len, clock.monotonic_nanos())
    }

    // ANCHOR[contextra-privacy:BED-011]
    /// Interner Hilfsablauf zur Auswertung an einem expliziten Nanosekunden-Zeitstempel.
    ///
    /// # Ablauf
    /// 1. Sucht den Sitzungs-Eintrag in der `scc::HashMap` oder erzeugt ihn atomar.
    /// 2. Konvertiert die Fensterdauer in Nanosekunden.
    /// 3. Sperrt den Sitzungs-Mutex kurzzeitig.
    /// 4. Führt `record_and_check` auf dem `SlidingWindowCounter` aus.
    pub fn record_and_check_at_nanos(
        &self,
        session: SessionId,
        payload_len: usize,
        now_nanos: u64,
    ) -> BulkExfiltrationOutcome {
        let entry_arc = self
            .sessions
            .entry(session)
            .or_insert_with(|| Arc::new(Mutex::new(SlidingWindowCounter::new())))
            .get()
            .clone();

        let window_nanos = u64::try_from(self.window.as_nanos()).unwrap_or(u64::MAX);
        let mut counter = entry_arc.lock();
        counter.record_and_check(
            payload_len,
            self.max_bytes_per_window,
            window_nanos,
            now_nanos,
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use contextra_testkit::ManualClock;
    use std::sync::Arc;

    // ANCHOR[contextra-privacy:TEST-BED-001]
    // Prüft, dass eine einzelne Anfrage unterhalb des Limits erlaubt wird.
    #[test]
    fn test_single_request_under_limit_allowed() {
        let detector = BulkExfiltrationDetector::new(1000, Duration::from_secs(60));
        let session = SessionId::from("session_1");

        let outcome = detector.record_and_check(session, 500);
        assert_eq!(outcome, BulkExfiltrationOutcome::Allow);
    }

    // ANCHOR[contextra-privacy:TEST-BED-002]
    // Prüft, dass kumulierte Anfragen oberhalb des Limits blockiert werden.
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

    // ANCHOR[contextra-privacy:TEST-BED-003]
    // Prüft, dass das abgelaufene Zeitfenster das verbrauchte Datenvolumen wieder freigibt.
    #[test]
    fn test_sliding_window_expiry_resets_volume() {
        let clock = Arc::new(ManualClock::new(100_000_000));
        let detector =
            BulkExfiltrationDetector::with_clock(1000, Duration::from_secs(10), clock.clone());
        let session = SessionId::from("session_1");

        let out1 = detector.record_and_check(session.clone(), 800);
        assert_eq!(out1, BulkExfiltrationOutcome::Allow);

        clock.advance(Duration::from_secs(5));
        let out2 = detector.record_and_check(session.clone(), 400);
        assert_eq!(
            out2,
            BulkExfiltrationOutcome::Block {
                window_bytes: 1200,
                limit: 1000
            }
        );

        clock.advance(Duration::from_secs(6));
        let out3 = detector.record_and_check(session.clone(), 500);
        assert_eq!(out3, BulkExfiltrationOutcome::Allow);
    }

    // ANCHOR[contextra-privacy:TEST-BED-004]
    // Prüft die strikte Trennung und Isolation verschiedener Sitzungs-Buckets.
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

    // ANCHOR[contextra-privacy:TEST-BED-005]
    // Prüft, dass nebenläufige parallele Anfragen kein Umgehen des Limits durch Race Conditions erlauben.
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

    // ANCHOR[contextra-privacy:TEST-BED-006]
    // Prüft das Fail-Closed-Verhalten bei einer Zeitregression (nicht-monotone Zeitänderung).
    #[test]
    fn test_time_regression_fails_closed() {
        let detector = BulkExfiltrationDetector::new(1000, Duration::from_secs(10));
        let session = SessionId::from("time_travel_session");

        detector.record_and_check_at_nanos(session.clone(), 100, 100_000);

        let outcome = detector.record_and_check_at_nanos(session, 100, 50_000);

        assert!(matches!(outcome, BulkExfiltrationOutcome::Block { .. }));
    }
}
