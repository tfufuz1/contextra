//! Anker, Uhr, Zufall (SP-ENV §3.8, SP-PORTS §4). Keine SystemTime in Signaturen.
use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AnchorValue {
    pub erase_epoch: u64,
    pub ledger_head: u64,
    pub catalog_epoch: u64,
}

pub trait Anchor: Send + Sync {
    fn read(&self) -> Result<AnchorValue, Error>;
    /// Monoton; bei verlorenem Wettlauf Error::CasLost.
    fn advance(&self, expect: AnchorValue, next: AnchorValue) -> Result<(), Error>;
}

pub trait Clock: Send + Sync {
    fn now_nanos(&self) -> u64;
}

pub trait Entropy: Send + Sync {
    fn fill(&self, buf: &mut [u8]) -> Result<(), Error>;
}
