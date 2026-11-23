//! ColdTier-Port für Ledger-Archivierung (E-39).
use crate::error::Error;

pub trait ColdTier: Send + Sync {
    fn archive(&self, key: &str, bytes: &[u8]) -> Result<(), Error>;
    fn fetch(&self, key: &str, out: &mut Vec<u8>) -> Result<(), Error>;
}
