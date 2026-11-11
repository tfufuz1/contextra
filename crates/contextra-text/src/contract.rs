//! Minimaler Umgebungskontrakt für `contextra-text` (Klasse A / SPEC §6).

/// Minimaler Fehlertyp für Umgebungs- und Kontraktoperationen.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Ungültige Eingabedaten.
    #[error("ungültige Eingabedaten: {0}")]
    InvalidInput(&'static str),
    /// Kapazitätsgrenze überschritten.
    #[error("Kapazitätsgrenze überschritten: gefordert {requested}, maximal {maximum}")]
    CapacityExceeded {
        /// Geforderte Kapazität.
        requested: usize,
        /// Maximale Kapazität.
        maximum: usize,
    },
    /// E/A-Fehler.
    #[error("E/A-Fehler: {0}")]
    Io(#[from] std::io::Error),
    /// Interner Zustandskonflikt.
    #[error("interner Zustandskonflikt: {0}")]
    StateConflict(&'static str),
    /// Beschädigte Daten.
    #[error("beschädigte Daten: {0}")]
    Corrupt(&'static str),
}

/// Einheitlicher Result-Typ für Kontraktoperationen.
pub type Result<T> = core::result::Result<T, Error>;

/// Minimale Zeituhr-Schnittstelle zur Bereitstellung monotoner oder Unix-Millisekunden.
pub trait Clock: Send + Sync {
    /// Liefert den aktuellen Unix-Zeitstempel in Millisekunden.
    fn now_unix_ms(&self) -> u64;
}

impl<T: contextra_env::Clock> Clock for T {
    #[inline]
    fn now_unix_ms(&self) -> u64 {
        self.now_unix_nanos() / 1_000_000
    }
}

/// Minimale Zufallsquelle für Baukasten-Operationen.
pub trait Rng: Send {
    /// Liefert die nächste Zufallszahl als `u64`.
    fn next_u64(&mut self) -> u64;
}

/// Autarker Dokumentenschlüssel (128-Bit fähig).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocKey(pub u128);

impl From<u64> for DocKey {
    #[inline]
    fn from(val: u64) -> Self {
        DocKey(val as u128)
    }
}

impl From<u128> for DocKey {
    #[inline]
    fn from(val: u128) -> Self {
        DocKey(val)
    }
}

/// Opake, monotone Sichtbarkeitsmarke zur Abstraktion von Transaktions- und MVCC-Ständen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Epoch(pub u64);
