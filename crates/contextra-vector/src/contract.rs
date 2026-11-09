#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("ungültige Eingabedaten: {0}")] InvalidInput(&'static str),
    #[error("Kapazitätsgrenze überschritten: gefordert {requested}, maximal {maximum}")]
    CapacityExceeded { requested: usize, maximum: usize },
    #[error("E/A-Fehler: {0}")] Io(#[from] std::io::Error),
    #[error("interner Zustandskonflikt: {0}")] StateConflict(&'static str),
    #[error("beschädigte Daten: {0}")] Corrupt(&'static str),
}
pub type Result<T> = core::result::Result<T, Error>;
pub trait Clock: Send + Sync { fn now_unix_ms(&self) -> u64; }
pub trait Rng: Send { fn next_u64(&mut self) -> u64; }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExternalId(pub u128);            // 128 Bit erhält vorhandene Fähigkeit (E-05); From<u64> vorhanden
impl From<u64> for ExternalId { fn from(v: u64) -> Self { Self(v as u128) } }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metric { Cosine, Euclidean, DotProduct }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScoredPoint { pub id: ExternalId, pub score: f32 }   // höher = besser, total geordnet
pub trait DistanceFn: Send + Sync { fn distance(&self, a: &[f32], b: &[f32]) -> f32; }  // injiziert, kein simd-Import
