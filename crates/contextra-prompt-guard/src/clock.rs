//! Neutrale Clock-Abstraktion für Zeitstempel im Sicherheits-Audit.

use std::time::{SystemTime, UNIX_EPOCH};

// MIGRATION(S-70): Clock wurde nach contract::Clock verlegt, bleibt hier re-exportiert.
pub use crate::contract::Clock;

/// Standard-Implementierung basierend auf `std::time::SystemTime`.
#[derive(Debug, Default, Clone)]
pub struct SystemClock;

impl SystemClock {
    /// Erstellt eine neue `SystemClock`.
    pub fn new() -> Self {
        Self
    }
}

impl Clock for SystemClock {
    #[allow(clippy::disallowed_methods)]
    fn now_unix_nanos(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }

    fn monotonic_nanos(&self) -> u64 {
        0
    }
}
