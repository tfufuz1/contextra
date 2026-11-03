//! Ausführungskontext für Purge-Operationen über Schichtgrenzen (SPEC §8.2, §11.4).
//!
//! `PurgeCx` trägt Deadline und Plattenreserve-Budget. Zeitmessung ausschließlich
//! über monotonen Zeitstempel oder Clock-Port (P7, P28).

use contextra_ports::clock::Clock;
use std::time::Duration;

/// Ausführungskontext eines Purge-Fensters (ER-20, ER-22).
#[derive(Debug, Clone)]
pub struct PurgeCx {
    /// Monotone Deadline in Nanosekunden (0 = keine Deadline).
    pub deadline_monotonic_nanos: u64,
    /// Mindest-freier Speicherplatz (Bytes) vor dem Rewrite (ER-22).
    pub min_free_bytes: u64,
    /// Batch-Fensterbreite.
    pub batch_window: Duration,
}

impl PurgeCx {
    /// Erstellt einen neuen Purge-Kontext.
    pub fn new(min_free_bytes: u64, batch_window: Duration) -> Self {
        Self {
            deadline_monotonic_nanos: 0,
            min_free_bytes,
            batch_window,
        }
    }

    /// Erstellt einen Kontext mit monotoner Deadline.
    pub fn with_deadline_nanos(mut self, deadline_monotonic_nanos: u64) -> Self {
        self.deadline_monotonic_nanos = deadline_monotonic_nanos;
        self
    }

    /// Prüft ob die Deadline bezogen auf die gegebene Clock überschritten ist.
    pub fn is_expired(&self, clock: &dyn Clock) -> bool {
        if self.deadline_monotonic_nanos == 0 {
            false
        } else {
            clock.monotonic_nanos() > self.deadline_monotonic_nanos
        }
    }
}
