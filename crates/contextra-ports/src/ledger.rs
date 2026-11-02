//! Ledger sink port definitions and contracts (SPEC §4.2, §11.8, PO-04, PO-08).
//!
//! Provides durable audit tracking of completed purges outside the primary data directory.

use crate::gate::{KeyBuf, Seq};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Discrete point in time measured in epoch milliseconds or nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Creates a new timestamp.
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    /// Returns the underlying numeric value.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Set of key ranges or tokens included in an erasure ledger entry.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeSet {
    ranges: Vec<(KeyBuf, KeyBuf)>,
}

impl RangeSet {
    /// Creates an empty range set.
    pub const fn empty() -> Self {
        Self { ranges: Vec::new() }
    }

    /// Creates a range set from intervals.
    pub fn new(ranges: Vec<(KeyBuf, KeyBuf)>) -> Self {
        Self { ranges }
    }

    /// Returns the slice of ranges.
    pub fn ranges(&self) -> &[(KeyBuf, KeyBuf)] {
        &self.ranges
    }
}

impl fmt::Debug for RangeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RangeSet(count={})", self.ranges.len())
    }
}

/// A tamper-evident entry in the erasure ledger (SPEC §4.2, §11.8, F.9).
///
/// Contains `(commitment, range_set, gated_seq, completed_at)` along with
/// `chain_hash` and `ledger_mac`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Commitment hash over the target erasure definition.
    pub commitment: [u8; 32],
    /// Key ranges covered by this erasure.
    pub range_set: RangeSet,
    /// Sequence at which the gate was closed.
    pub gated_seq: Seq,
    /// Timestamp when erasure and witness checks were finalized.
    pub completed_at: Timestamp,
    /// Chained BLAKE3 hash incorporating previous entry digest.
    pub chain_hash: [u8; 32],
    /// Authentication code / MAC protecting integrity.
    pub ledger_mac: [u8; 32],
}

impl LedgerEntry {
    /// Creates a new ledger entry with explicit fields.
    pub const fn new(
        commitment: [u8; 32],
        range_set: RangeSet,
        gated_seq: Seq,
        completed_at: Timestamp,
        chain_hash: [u8; 32],
        ledger_mac: [u8; 32],
    ) -> Self {
        Self {
            commitment,
            range_set,
            gated_seq,
            completed_at,
            chain_hash,
            ledger_mac,
        }
    }

    /// Constructs a ledger entry and computes its chained BLAKE3 hash from the prior entry hash.
    pub fn compute_chained(
        commitment: [u8; 32],
        range_set: RangeSet,
        gated_seq: Seq,
        completed_at: Timestamp,
        prev_hash: [u8; 32],
        ledger_mac: [u8; 32],
    ) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&prev_hash);
        hasher.update(&commitment);
        hasher.update(&gated_seq.as_u64().to_le_bytes());
        hasher.update(&completed_at.as_u64().to_le_bytes());
        for (start, end) in range_set.ranges() {
            hasher.update(start.as_bytes());
            hasher.update(end.as_bytes());
        }
        let chain_hash = *hasher.finalize().as_bytes();

        Self {
            commitment,
            range_set,
            gated_seq,
            completed_at,
            chain_hash,
            ledger_mac,
        }
    }

    /// Verifies the entry's chain hash against the supplied previous block hash.
    pub fn verify_chain_hash(&self, prev_hash: &[u8; 32]) -> bool {
        let mut hasher = blake3::Hasher::new();
        hasher.update(prev_hash);
        hasher.update(&self.commitment);
        hasher.update(&self.gated_seq.as_u64().to_le_bytes());
        hasher.update(&self.completed_at.as_u64().to_le_bytes());
        for (start, end) in self.range_set.ranges() {
            hasher.update(start.as_bytes());
            hasher.update(end.as_bytes());
        }
        let computed = hasher.finalize();
        computed.as_bytes() == &self.chain_hash
    }
}

impl fmt::Debug for LedgerEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Redact any possible user content (PO-08)
        write!(
            f,
            "LedgerEntry {{ gated_seq: {:?}, completed_at: {:?}, chain_hash: {:02x?} }}",
            self.gated_seq, self.completed_at, &self.chain_hash
        )
    }
}

/// Durable audit ledger sink located outside the primary data directory (SPEC §4.2, §11.8).
pub trait LedgerSink: Send + Sync {
    /// Appends a new verified erasure entry to the ledger.
    fn append(&self, e: &LedgerEntry) -> Result<(), String>;

    /// Reads all entries recorded in the ledger for verification or auditing.
    fn read_all(&self) -> Result<Vec<LedgerEntry>, String>;

    /// Prunes entries older than the specified retention threshold.
    fn prune(&self, older_than: Timestamp) -> Result<usize, String>;
}
