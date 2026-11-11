// FILE-CONTEXT
// ZWECK: Backoff- und Cooldown-Zustandsautomat für HNSW-Rebuilds ohne Wanduhr.
// INVARIANTEN: Zeitabfragen erfolgen ausschließlich über Clock (INV-INDEX-APPLY-1); saturierende Arithmetik.
// HOTSPOTS: BackoffTracker::check_and_update
// STAND: TS:2026-11-11T02:00:00Z

use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::error::{Result, VectorError};
use crate::hnsw::types::HnswIndexCore;

/// Holt den letzten Rebuild-Zeitstempel in Nanosekunden (oder None bei 0).
pub(crate) fn get_last_rebuild_nanos(core: &HnswIndexCore) -> Option<u64> {
    let val = core.cold.last_rebuild_nanos.load(Ordering::Acquire);
    if val == 0 {
        None
    } else {
        Some(val)
    }
}

/// Setzt den letzten Rebuild-Zeitstempel in Nanosekunden.
pub(crate) fn set_last_rebuild_nanos(core: &HnswIndexCore, nanos: u64) {
    core.cold.last_rebuild_nanos.store(nanos, Ordering::Release);
}

/// Autarker Backoff-Zustandsautomat zur Ratenbegrenzung von Rebuild-Vorgängen.
#[derive(Debug, Clone)]
pub struct BackoffTracker {
    base_delay: Duration,
    max_delay: Duration,
    alert_threshold: u64,
    last_rebuild_nanos: Option<u64>,
    consecutive_rapid_rebuilds: u64,
}

impl BackoffTracker {
    /// Erstellt einen neuen BackoffTracker mit den konfigurierten Parametern.
    pub fn new(base_delay: Duration, max_delay: Duration, alert_threshold: u64) -> Self {
        Self {
            base_delay,
            max_delay,
            alert_threshold,
            last_rebuild_nanos: None,
            consecutive_rapid_rebuilds: 0,
        }
    }

    /// Berechnet die aktuelle Cooldown-Dauer anhand der Anzahl aufeinanderfolgender Rebuilds.
    pub fn current_cooldown(&self) -> Duration {
        if self.consecutive_rapid_rebuilds == 0 {
            self.base_delay
        } else {
            let shift = self.consecutive_rapid_rebuilds.saturating_sub(1).min(30);
            let multiplier = 1u64.checked_shl(shift as u32).unwrap_or(u64::MAX);
            let calc = self.base_delay.saturating_mul(multiplier as u32);
            calc.min(self.max_delay)
        }
    }

    /// Prüft, ob ein Rebuild zum Zeitpunkt `now_nanos` erlaubt ist, und aktualisiert den internen Zustand.
    pub fn check_and_update(&mut self, now_nanos: u64) -> Result<()> {
        if let Some(last_nanos) = self.last_rebuild_nanos {
            let elapsed_nanos = now_nanos.saturating_sub(last_nanos);
            let elapsed = Duration::from_nanos(elapsed_nanos);
            let cooldown = self.current_cooldown();

            if elapsed < cooldown {
                self.consecutive_rapid_rebuilds = self.consecutive_rapid_rebuilds.saturating_add(1);
                if self.consecutive_rapid_rebuilds >= self.alert_threshold {
                    tracing::warn!(
                        consecutive_rebuilds = self.consecutive_rapid_rebuilds,
                        cooldown_seconds = cooldown.as_secs_f64(),
                        "Excessive HNSW rebuilds triggered in rapid succession; suppressing rebuild due to backoff"
                    );
                } else {
                    tracing::warn!(
                        cooldown_seconds = cooldown.as_secs_f64(),
                        "HNSW rebuild suppressed by backoff cooldown"
                    );
                }
                return Err(VectorError::Index(format!(
                    "Rebuild rate limited by backoff cooldown ({:?})",
                    cooldown
                )));
            } else if elapsed >= self.max_delay.saturating_mul(2) {
                self.consecutive_rapid_rebuilds = 0;
            }
        }
        Ok(())
    }

    /// Markiert einen erfolgreichen Rebuild zum Zeitpunkt `now_nanos` und setzt den Zähler zurück.
    pub fn record_success(&mut self, now_nanos: u64) {
        self.last_rebuild_nanos = Some(now_nanos);
        self.consecutive_rapid_rebuilds = 0;
    }

    /// Liefert die Anzahl der aufeinanderfolgenden schnellen Rebuild-Versuche.
    pub fn consecutive_rapid_rebuilds(&self) -> u64 {
        self.consecutive_rapid_rebuilds
    }

    /// Liefert den Zeitstempel des letzten erfolgreichen Rebuilds.
    pub fn last_rebuild_nanos(&self) -> Option<u64> {
        self.last_rebuild_nanos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standalone_backoff_tracker() {
        let mut tracker =
            BackoffTracker::new(Duration::from_millis(50), Duration::from_millis(400), 3);

        assert!(tracker.check_and_update(1_000_000).is_ok());
        tracker.record_success(1_000_000);
        assert_eq!(tracker.last_rebuild_nanos(), Some(1_000_000));
        assert_eq!(tracker.consecutive_rapid_rebuilds(), 0);

        // Immediate attempt fails
        assert!(tracker.check_and_update(1_010_000).is_err());
        assert_eq!(tracker.consecutive_rapid_rebuilds(), 1);

        // After cooldown succeeds
        assert!(tracker.check_and_update(60_000_000).is_ok());
    }
}
