//! # System Pressure Signals & Levels (Ring 1)
//!
//! Definition der Druckpegel, Konstanten und des Signal-Traits für externen Speicherdruck.

/// Kritischer Schwellenwert für die WAL-Warteschlangentiefe.
pub const WAL_QUEUE_CRITICAL_THRESHOLD: usize = 500;
/// Erhöhter Schwellenwert für die WAL-Warteschlangentiefe.
pub const WAL_QUEUE_ELEVATED_THRESHOLD: usize = 100;
/// Kritischer Schwellenwert für die Tokio-Scheduler-Warteschlange (Verhältnis).
pub const SCHEDULER_QUEUE_CRITICAL: f32 = 0.85;
/// Erhöhter Schwellenwert für die Tokio-Scheduler-Warteschlange (Verhältnis).
pub const SCHEDULER_QUEUE_ELEVATED: f32 = 0.60;
/// Kritischer Schwellenwert für Blocking-Pool Auslastung.
pub const BLOCKING_UTIL_CRITICAL: f32 = SCHEDULER_QUEUE_CRITICAL;
/// Erhöhter Schwellenwert für Blocking-Pool Auslastung.
pub const BLOCKING_UTIL_ELEVATED: f32 = SCHEDULER_QUEUE_ELEVATED;

/// Injektions-Trait für externe Speicherdruck-Signale (ohne globale Zustände).
pub trait SystemPressureSignal: Send + Sync {
    /// Liefert den aktuellen Druckpegel.
    fn current_pressure(&self) -> PressureLevel;

    /// Gibt eine empfohlene Durchsatz-Reduktionsrate (0.0 bis 1.0) zurück.
    fn advise_reduction_ratio(&self) -> f32;
}

/// Diskreter Systemdruck-Pegel für Backpressure-Steuerung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureLevel {
    /// Normaler Betriebszustand ohne Drosselung.
    Normal,
    /// System unter Last; leichte Verzögerungen empfohlen.
    Elevated,
    /// Backpressure aktiv; neue Schreibvorgänge müssen verlangsamt werden.
    Critical,
}
