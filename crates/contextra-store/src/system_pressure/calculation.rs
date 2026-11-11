//! # System Pressure Calculation (Ring 1)
//!
//! Schwellenwert-Logik zur Berechnung des aktuellen Systemdrucks.

use super::signal::{
    PressureLevel, BLOCKING_UTIL_CRITICAL, BLOCKING_UTIL_ELEVATED, SCHEDULER_QUEUE_CRITICAL,
    SCHEDULER_QUEUE_ELEVATED, WAL_QUEUE_CRITICAL_THRESHOLD, WAL_QUEUE_ELEVATED_THRESHOLD,
};

/// Aggregierter Systemdruck-Zustand.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemPressure {
    /// Warteschlangentiefe anstehender WAL-Schreiboperationen.
    pub wal_queue_depth: usize,
    /// Tokio Scheduler Queue-Verhältnis (0.0 bis 1.0).
    pub scheduler_queue_depth: f32,
    /// Blocking-Pool-Auslastung (0.0 bis 1.0).
    pub blocking_util: f32,
    /// Warteschlangentiefe für Embeddings.
    pub embedding_queue_depth: usize,
    /// Berechneter Druckpegel.
    pub pressure_level: PressureLevel,
}

/// Berechnet den Systemdruck anhand der Einzelsignale.
pub fn compute_pressure(
    wal_depth: usize,
    scheduler_queue_depth: f32,
    blocking_util: f32,
    embedding_queue: usize,
    max_permits: usize,
) -> SystemPressure {
    let level = if scheduler_queue_depth > SCHEDULER_QUEUE_CRITICAL
        || blocking_util > BLOCKING_UTIL_CRITICAL
        || wal_depth > WAL_QUEUE_CRITICAL_THRESHOLD
        || (max_permits > 0 && embedding_queue == 0)
    {
        PressureLevel::Critical
    } else if scheduler_queue_depth > SCHEDULER_QUEUE_ELEVATED
        || blocking_util > BLOCKING_UTIL_ELEVATED
        || wal_depth > WAL_QUEUE_ELEVATED_THRESHOLD
    {
        PressureLevel::Elevated
    } else {
        PressureLevel::Normal
    };

    SystemPressure {
        wal_queue_depth: wal_depth,
        scheduler_queue_depth,
        blocking_util,
        embedding_queue_depth: embedding_queue,
        pressure_level: level,
    }
}
