//! Engine-Verträge (Platzhalter, Signaturen folgen SP-MEMVER, SP-EXT).
use crate::error::Error;

pub trait Memtable: Send + Sync {
    fn approx_bytes(&self) -> u64;
}

pub trait ConsumerFeed: Send + Sync {
    fn poll(&self, max: usize) -> Result<usize, Error>;
}
