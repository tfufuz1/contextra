//! Spawner-Port. Keine Sperre über Rückrufen (INDEX §4 Regel 5).
use crate::error::Error;

pub trait Spawner: Send + Sync {
    fn spawn(&self, name: &'static str, job: Box<dyn FnOnce() + Send>) -> Result<(), Error>;
}
