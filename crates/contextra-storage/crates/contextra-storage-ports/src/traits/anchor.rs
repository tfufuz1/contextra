//! Anker, Uhr, Zufall (SP-ENV §3.8, SP-PORTS §4). Keine SystemTime in Signaturen.
use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AnchorValue {
    pub erase_epoch: u64,
    pub ledger_head: u64,
    pub catalog_epoch: u64,
}

/// Strength is explicit because a local file prevents neither rollback of a
/// copied volume nor two restored hosts from writing concurrently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorStrength {
    Weak,
    Strong,
}

/// Injected time sample. `durable_utc_millis` is the persisted UTC high-water
/// mark; adapters must never move it backwards. `skew` is fail-closed when
/// the observed wall clock is implausibly far from that mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockReading {
    /// Monotonic process time used for TTLs and elapsed durations.
    pub monotonic_nanos: u64,
    /// Current UTC time in milliseconds since the Unix epoch.
    pub utc_millis: u64,
    /// Durably persisted UTC high-water mark in milliseconds.
    pub durable_utc_millis: u64,
    /// Whether the adapter considers wall-clock movement safe for deadlines.
    pub skew: ClockSkew,
}

impl ClockReading {
    /// Returns UTC usable for durable deadlines, rejecting clock rollback and
    /// adapter-reported skew. Monotonic process time remains usable for
    /// elapsed-duration measurement while this check fails.
    pub fn deadline_utc_millis(self) -> Result<u64, Error> {
        if self.skew == ClockSkew::Uncertain || self.utc_millis < self.durable_utc_millis {
            return Err(Error::ClockUncertain);
        }
        Ok(self.utc_millis)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockSkew {
    Certain,
    Uncertain,
}

pub trait Anchor: Send + Sync {
    fn read(&self) -> Result<AnchorValue, Error>;
    /// Monoton; bei verlorenem Wettlauf Error::CasLost.
    fn advance(&self, expect: AnchorValue, next: AnchorValue) -> Result<(), Error>;
    fn strength(&self) -> AnchorStrength;
}

pub trait Clock: Send + Sync {
    /// Returns monotone and UTC time plus the durable clock floor.
    ///
    /// Storage uses monotonic time for process-local expiry. Durable deadlines
    /// must fail closed when `skew` is `Uncertain`.
    fn read(&self) -> Result<ClockReading, Error>;
    /// Durably advances the UTC high-water mark with compare-and-swap
    /// semantics. A lower value is rejected; a lost race returns `CasLost`.
    fn advance_utc_floor(&self, expect_millis: u64, next_millis: u64) -> Result<(), Error>;
}

pub trait Entropy: Send + Sync {
    fn fill(&self, buf: &mut [u8]) -> Result<(), Error>;
}
