//! Eingefrorener Crate-Vertrag `text` (SPEC v4.0 §4 / Klasse A).

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("ungültige Eingabedaten: {0}")]
    InvalidInput(&'static str),
    #[error("Kapazitätsgrenze überschritten: gefordert {requested}, maximal {maximum}")]
    CapacityExceeded { requested: usize, maximum: usize },
    #[error("E/A-Fehler: {0}")]
    Io(#[from] std::io::Error),
    #[error("interner Zustandskonflikt: {0}")]
    StateConflict(&'static str),
    #[error("beschädigte Daten: {0}")]
    Corrupt(&'static str),
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait Clock: Send + Sync {
    fn now_unix_ms(&self) -> u64;
}

pub trait Rng: Send {
    fn next_u64(&mut self) -> u64;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocKey(pub u128); // From<u64> vorhanden, erhält 128-Bit-Fähigkeit

impl From<u64> for DocKey {
    #[inline]
    fn from(val: u64) -> Self {
        Self(val as u128)
    }
}

impl From<u128> for DocKey {
    #[inline]
    fn from(val: u128) -> Self {
        Self(val)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Epoch(pub u64); // opake, monotone Sichtbarkeitsmarke; ersetzt TxId (Konvertierung nur im Adapter)
