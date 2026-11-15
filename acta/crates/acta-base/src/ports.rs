//! Kernports Zeit und Zufall (Spec 3.7, 4.6). `SystemTime::now` und `thread_rng` sind im Kern verboten.

/// Zeitquelle. Zeit ist nicht vertrauenswuerdig (A-CLOCK).
pub trait Clock: Send + Sync {
    /// Millisekunden seit Unix-Epoche (UTC).
    fn now_ms(&self) -> u64;
}

/// Fehler der Entropiequelle. Ein Entropiefehler erzeugt nie Schluessel oder Umschlag (Z10).
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("entropy source failed")]
pub struct EntropyError;

/// Fehlbarer kryptografischer Zufall.
pub trait CryptoRng: Send + Sync {
    /// Fuellt `dst` vollstaendig oder liefert einen Fehler (kein Teilergebnis, kein Ersatzwert).
    ///
    /// # Errors
    /// `EntropyError`, wenn die Quelle nicht liefern kann.
    fn try_fill(&self, dst: &mut [u8]) -> Result<(), EntropyError>;
}
