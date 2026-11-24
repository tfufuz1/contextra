//! # Throttling & Rate-Limiting (Ring 1)
//!
//! Begrenzung der I/O-Bandbreite und Berücksichtigung des Systemdrucks bei der Kompaktierung.

use crate::system_pressure::{PressureLevel, SystemPressure};
use std::time::Duration;
use tokio::sync::watch::Receiver;

/// Berechnet eine Drossel-Verzögerung basierend auf dem aktuellen Systemdruck.
pub fn calculate_throttle_delay(
    pressure_rx: Option<&Receiver<SystemPressure>>,
) -> Option<Duration> {
    if let Some(rx) = pressure_rx {
        let level = rx.borrow().pressure_level;
        match level {
            PressureLevel::Critical => {
                tracing::warn!("Compaction merge delayed due to Critical system pressure.");
                Some(Duration::from_millis(50))
            }
            PressureLevel::Elevated => {
                tracing::debug!("Compaction merge active under Elevated system pressure.");
                None
            }
            PressureLevel::Normal => None,
        }
    } else {
        None
    }
}
