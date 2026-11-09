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
pub type NodeId = u64;
pub type Weight = f32; // endlich, >= 0; Validierung an der API-Grenze
