//! Minimalverträge und Umweltabstraktionen für `contextra-rank` (R4/R3 Baustein-Architektur).
//!
//! Definiert die minimalen Traits, die der Kern von `contextra-rank` an seine Laufzeitumgebung
//! stellt (z. B. deterministische Zeitstempel für Kalibrierung und Driftmessung).

/// Minimaler Clock-Port für deterministische Zeitmessung im Kern.
pub trait Clock: Send + Sync {
    /// Gibt den aktuellen UNIX-Zeitstempel in Nanosekunden zurück.
    fn now_unix_nanos(&self) -> u64;

    /// Gibt eine monotone Zeit in Nanosekunden zurück.
    fn monotonic_nanos(&self) -> u64 {
        self.now_unix_nanos()
    }
}
