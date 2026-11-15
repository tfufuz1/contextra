//! Minimale Umwelt- und Abstraktionsverträge für den Prompt-Guard.
//!
//! Gemäß den Entkopplungsrichtlinien definiert der Kern seine minimal benötigten
//! Verträge selbst. Für Zeitstempel in Sicherheits-Audits wird der Trait [`Clock`] definiert.

/// Trait für Zeitgeber (ohne Abhängigkeit von acta-base oder externen Runtimes).
pub trait Clock: Send + Sync {
    /// Liefert die aktuellen Unix-Nanosekunden.
    fn now_unix_nanos(&self) -> u64;

    /// Liefert monotone Nanosekunden (z. B. für Zeitdifferenzen oder Benchmarks).
    fn monotonic_nanos(&self) -> u64;
}
