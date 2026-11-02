//! Ausführungskontext für Purge-Operationen über Schichtgrenzen (SPEC §8.2, §11.4).
//!
//! `PurgeCx` trägt Deadline und Plattenreserve-Budget.

use std::time::{Duration, Instant};

/// Ausführungskontext eines Purge-Fensters (ER-20, ER-22).
#[derive(Debug, Clone)]
pub struct PurgeCx {
    /// Deadline für das Purge-Fenster.
    pub deadline: Option<Instant>,
    /// Mindest-freier Speicherplatz (Bytes) vor dem Rewrite (ER-22).
    pub min_free_bytes: u64,
    /// Batch-Fensterbreite.
    pub batch_window: Duration,
}

impl PurgeCx {
    /// Erstellt einen neuen Purge-Kontext.
    pub fn new(min_free_bytes: u64, batch_window: Duration) -> Self {
        Self {
            deadline: None,
            min_free_bytes,
            batch_window,
        }
    }

    /// Erstellt einen Kontext mit Deadline.
    pub fn with_deadline(mut self, deadline: Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Prüft ob die Deadline überschritten ist.
    pub fn is_expired(&self) -> bool {
        self.deadline.map(|d| Instant::now() > d).unwrap_or(false)
    }
}
