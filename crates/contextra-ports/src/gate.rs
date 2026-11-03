//! ReadGate port definitions and contracts (SPEC §4.2, §11.3, PO-04, PO-07).
//!
//! Provides the synchronous gate interface [`ReadGate`] used on the hot read and commit paths.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Monotonic sequence number for transactions and gate markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Seq(pub u64);

impl Seq {
    /// Creates a new sequence number.
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    /// Returns the underlying numeric sequence.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Fixed or bounded key buffer for storage operations (SPEC §8.1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct KeyBuf(pub [u8; 16]);

impl KeyBuf {
    /// Creates a new `KeyBuf` from raw bytes.
    pub const fn new(raw: [u8; 16]) -> Self {
        Self(raw)
    }

    /// Returns raw bytes slice.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for KeyBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Redact user content (PO-08)
        write!(f, "KeyBuf([REDACTED])")
    }
}

/// Boundary key used for compaction or range scanning.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum KeyBound {
    /// Exact key match boundary.
    Exact(KeyBuf),
    /// Unbounded lower boundary.
    UnboundedLower,
    /// Unbounded upper boundary.
    UnboundedUpper,
}

impl KeyBound {
    /// Creates an exact key boundary.
    pub const fn exact(key: KeyBuf) -> Self {
        Self::Exact(key)
    }
}

impl fmt::Debug for KeyBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact(_) => write!(f, "KeyBound::Exact([REDACTED])"),
            Self::UnboundedLower => write!(f, "KeyBound::UnboundedLower"),
            Self::UnboundedUpper => write!(f, "KeyBound::UnboundedUpper"),
        }
    }
}

/// Half-open key range `[start, end)` for scan and compaction boundaries.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyRange {
    /// Lower boundary inclusive.
    pub start: KeyBuf,
    /// Upper boundary exclusive.
    pub end: KeyBuf,
}

impl KeyRange {
    /// Creates a new key range.
    pub const fn new(start: KeyBuf, end: KeyBuf) -> Self {
        Self { start, end }
    }

    /// Returns `true` if `key` falls within `[start, end)`.
    pub fn contains(&self, key: &KeyBuf) -> bool {
        key >= &self.start && key < &self.end
    }
}

impl fmt::Debug for KeyRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyRange {{ [REDACTED] }}")
    }
}

/// Gate decision for point queries or file boundaries (SPEC §4.2, §11.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateDecision {
    /// Key or boundary is open for reading.
    Open,
    /// Key or boundary has been erased. Access must be blocked immediately.
    Erased,
    /// Key or boundary is temporarily restricted (e.g. pending gate sequence).
    Restricted,
}

/// Decision variant for range operations during compaction (SPEC §4.2, I-18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RangeGate {
    /// Range contains erased data: records may be safely discarded.
    ContainsErased,
    /// Range is restricted: records must never be discarded until release.
    RestrictedNeverDiscard,
    /// Entire range is completely open.
    EntirelyOpen,
}

/// Synchronous, memory-only read gate port interface (SPEC §4.2, §11.3, PO-07).
///
/// # Invariants
/// - **PO-07**: `ReadGate` is synchronous, performs zero I/O, and executes in $O(\log n)$ time.
pub trait ReadGate: Send + Sync {
    /// Checks point status of an individual key.
    ///
    /// # Fehler
    /// Synchron, keine I/O, panikfrei; liefert GateDecision.
    ///
    /// # Crash
    /// Reiner RAM-Lookup in O(log n); keine Zustandsänderung.
    fn check(&self, k: &KeyBuf) -> GateDecision;

    /// Checks boundary status of a file or partition block for compaction or scans.
    ///
    /// # Fehler
    /// Synchron, keine I/O; liefert GateDecision.
    ///
    /// # Crash
    /// Reiner RAM-Lookup; keine Seiteneffekte.
    fn check_bound(&self, b: &KeyBound) -> GateDecision;

    /// Evaluates range policy for compaction (I-18: Erased may be dropped, Restricted never).
    ///
    /// # Fehler
    /// Synchron, keine I/O; liefert RangeGate.
    ///
    /// # Crash
    /// Reiner RAM-Lookup für Kompaktions-Entscheidungen.
    fn variant_for_range(&self, r: &KeyRange) -> RangeGate;

    /// Retrieves the sequence number at which the key was gated, if any (Restore-Guard, commit rejection).
    ///
    /// # Fehler
    /// Synchron, liefert Option<Seq>.
    ///
    /// # Crash
    /// Reiner RAM-Lookup; keine Mutation.
    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq>;
}
