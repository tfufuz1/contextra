//! Ausführungskontext für Purge-Operationen über Schichtgrenzen (SPEC §8.2, §11.4).
//!
//! `PurgeCx` trägt Deadline und Plattenreserve-Budget.

use contextra_ports::clock::{Clock, SystemClock};
use std::time::Duration;

/// Ausführungskontext eines Purge-Fensters (ER-20, ER-22).
#[derive(Debug, Clone)]
pub struct PurgeCx {
    /// Deadline für das Purge-Fenster in Monotonic Nanos.
    pub deadline_nanos: Option<u64>,
    /// Mindest-freier Speicherplatz (Bytes) vor dem Rewrite (ER-22).
    pub min_free_bytes: u64,
    /// Batch-Fensterbreite.
    pub batch_window: Duration,
}

impl PurgeCx {
    /// Erstellt einen neuen Purge-Kontext.
    pub fn new(min_free_bytes: u64, batch_window: Duration) -> Self {
        Self {
            deadline_nanos: None,
            min_free_bytes,
            batch_window,
        }
    }

    /// Erstellt einen Kontext mit Deadline (Monotonic Nanos).
    pub fn with_deadline_nanos(mut self, deadline_nanos: u64) -> Self {
        self.deadline_nanos = Some(deadline_nanos);
        self
    }

    /// Prüft ob die Deadline überschritten ist bezüglich einer gegebenen Uhr.
    pub fn is_expired_against(&self, clock: &dyn Clock) -> bool {
        self.deadline_nanos
            .map(|d| clock.monotonic_nanos() > d)
            .unwrap_or(false)
    }

    /// Prüft ob die Deadline mit der Systemuhr überschritten ist.
    pub fn is_expired(&self) -> bool {
        let clock = SystemClock::new();
        self.is_expired_against(&clock)
    }
}
