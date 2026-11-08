//! Zeitabstraktion (Clock) für `contextra-adapt` (SPEC v4.0 §2 B11).
//!
//! Zeit wird ausschließlich als Parameter oder lokaler Trait injiziert.

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Trait für deterministische Zeitgeber.
pub trait Clock: Send + Sync + 'static {
    /// Liefert die aktuelle Unix-Epochenzeit in Nanosekunden.
    fn now_unix_nanos(&self) -> u64;

    /// Liefert einen monoton steigenden Zeitstempel in Nanosekunden.
    fn monotonic_nanos(&self) -> u64;
}

/// Standard-Implementierung von [`Clock`] unter Verwendung des Betriebssystem-Zeitgebers.
#[derive(Debug, Clone)]
pub struct SystemClock {
    epoch_baseline: Instant,
    unix_baseline_nanos: u64,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemClock {
    /// Erstellt eine neue `SystemClock`-Instanz.
    #[allow(clippy::disallowed_methods)]
    pub fn new() -> Self {
        // ALLOW-DETERMINISM: SystemClock encapsulation baseline (P28)
        let now_sys = SystemTime::now(); // ALLOW-DETERMINISM
        let unix_baseline_nanos = now_sys
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_nanos() as u64;
        // ALLOW-DETERMINISM: SystemClock monotonic baseline (P28)
        let epoch_baseline = Instant::now(); // ALLOW-DETERMINISM

        Self {
            epoch_baseline,
            unix_baseline_nanos,
        }
    }
}

impl Clock for SystemClock {
    fn now_unix_nanos(&self) -> u64 {
        let elapsed = self.epoch_baseline.elapsed().as_nanos() as u64;
        self.unix_baseline_nanos.saturating_add(elapsed)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.epoch_baseline.elapsed().as_nanos() as u64
    }
}

impl<T: Clock + ?Sized> Clock for Arc<T> {
    fn now_unix_nanos(&self) -> u64 {
        (**self).now_unix_nanos()
    }

    fn monotonic_nanos(&self) -> u64 {
        (**self).monotonic_nanos()
    }
}

#[cfg(feature = "ports-compat")]
impl Clock for contextra_ports::SystemClock {
    fn now_unix_nanos(&self) -> u64 {
        contextra_ports::Clock::now_unix_nanos(self)
    }

    fn monotonic_nanos(&self) -> u64 {
        contextra_ports::Clock::monotonic_nanos(self)
    }
}
