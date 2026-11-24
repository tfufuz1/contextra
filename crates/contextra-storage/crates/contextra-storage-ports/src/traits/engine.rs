//! Engine-Verträge (SP-MEMVER, SP-EXT).
#![forbid(unsafe_code)]

use crate::error::Error;

pub trait Memtable: Send + Sync {
    fn approx_bytes(&self) -> u64;
    fn is_empty(&self) -> bool {
        self.approx_bytes() == 0
    }
}

pub trait ConsumerFeed: Send + Sync {
    fn poll(&self, max: usize) -> Result<usize, Error>;
    fn watermark(&self) -> u64;
}
