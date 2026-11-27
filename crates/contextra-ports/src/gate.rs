//! ReadGate v2 port definitions and fail-closed contracts (WP-02 §4.1, G-6).
//!
//! Provides the synchronous, allocation-free (on the `Open` fast path) gate interface
//! [`ReadGate`] used on the hot read, scan, write, and compaction paths.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Re-export canonical monotonic sequence number from `contextra-types`.
pub use contextra_types::{DocId, Seq, TenantId};

/// Gate decision for point queries and document lookups.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum GateDecision {
    /// Key or document is open for reading.
    Open,
    /// Key or document is temporarily restricted (e.g. pending gate sequence); never discard.
    Restricted,
    /// Key or document has been erased. Access must be blocked immediately.
    Erased,
}

impl GateDecision {
    /// Returns `true` if access is blocked (`Restricted` or `Erased`).
    pub const fn is_blocked(self) -> bool {
        !matches!(self, Self::Open)
    }
}

/// Gate decision for a prefix or range scan.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum RangeGate {
    /// Entire prefix range is open for reading.
    EntirelyOpen,
    /// Prefix range contains both open and blocked keys; per-key checks required.
    Partial,
    /// Entire prefix range is blocked; scan can short-circuit.
    EntirelyBlocked,
}

/// Discard rule for compaction and flush operations.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DiscardRule {
    /// Version must be retained (e.g. `Open` or `Restricted`).
    Keep,
    /// Version is erased within the active scope and may be physically dropped.
    MayDrop,
}

/// Synchronous, memory-only read gate port interface (WP-02 §4.1, G-6).
///
/// # Invariants
/// - Synchronous, zero I/O, panic-free, and allocation-free on the `Open` fast path.
/// - **Fail-closed (G-6):** This trait intentionally defines **no default methods**.
///   Every implementer must explicitly implement every method.
/// - Unloadable or unknown gate state must return `Restricted` for all queries (I-7).
///
/// # Doctests (Compile-Fail Checks)
///
/// `TST-PORTS-01`: Implementing `ReadGate` without all methods must fail to compile:
/// ```compile_fail
/// use contextra_ports::gate::ReadGate;
/// struct IncompleteGate;
/// impl ReadGate for IncompleteGate {}
/// ```
pub trait ReadGate: Send + Sync {
    /// Returns `false` if no gates are active (fast path: all checks return `Open`).
    /// Implementations should use a single `Relaxed` atomic load.
    ///
    /// # Fehler
    /// Keine; synchroner atomarer Zustandscheck.
    /// # Crash
    /// Reiner RAM-Zugriff ohne Mutation.
    fn is_active(&self) -> bool;

    /// Returns the strictly monotonic gate epoch counter (`Acquire` load).
    ///
    /// # Fehler
    /// Keine; synchroner atomarer Abruf.
    /// # Crash
    /// Reiner RAM-Zugriff ohne Mutation.
    fn epoch(&self) -> u64;

    /// Evaluates the gate decision for a raw key as seen by `LsmStorage`
    /// (`t:{tenant}:` + collection key).
    ///
    /// # Fehler
    /// Keine; synchron, panikfrei, ohne I/O.
    /// # Crash
    /// Reiner RAM-Lookup ohne Seiteneffekte.
    fn check(&self, raw_key: &[u8]) -> GateDecision;

    /// Version-aware gate check: `Erased { up_to }` blocks only `version_seq <= up_to`.
    ///
    /// # Fehler
    /// Keine; synchron, panikfrei, ohne I/O.
    /// # Crash
    /// Reiner RAM-Lookup ohne Seiteneffekte.
    fn check_version(&self, raw_key: &[u8], version_seq: Seq) -> GateDecision;

    /// Evaluates whether an entire prefix range is open, partial, or entirely blocked.
    ///
    /// # Fehler
    /// Keine; synchron, panikfrei, ohne I/O.
    /// # Crash
    /// Reiner RAM-Lookup ohne Seiteneffekte.
    fn check_prefix(&self, prefix: &[u8]) -> RangeGate;

    /// Evaluates whether a key version may be discarded during compaction or flush.
    ///
    /// # Fehler
    /// Keine; synchron, panikfrei, ohne I/O.
    /// # Crash
    /// Reiner RAM-Lookup ohne Seiteneffekte.
    fn discard_rule(&self, raw_key: &[u8], version_seq: Seq) -> DiscardRule;

    /// Document-level gate check for secondary indices.
    /// `collection` is passed without internal prefix (e.g. `"default"`, `"docs"`).
    ///
    /// # Fehler
    /// Keine; synchron, panikfrei, ohne I/O.
    /// # Crash
    /// Reiner RAM-Lookup ohne Seiteneffekte.
    fn check_doc(&self, tenant: TenantId, collection: &str, doc: DocId) -> GateDecision;

    /// Returns the sequence number at which `raw_key` was gated, if any
    /// (used for write rejection and diagnostics).
    ///
    /// # Fehler
    /// Keine; synchron, panikfrei, ohne I/O.
    /// # Crash
    /// Reiner RAM-Lookup ohne Mutation.
    fn gated_seq(&self, raw_key: &[u8]) -> Option<Seq>;
}

impl<T: ReadGate + ?Sized> ReadGate for Arc<T> {
    fn is_active(&self) -> bool {
        (**self).is_active()
    }

    fn epoch(&self) -> u64 {
        (**self).epoch()
    }

    fn check(&self, raw_key: &[u8]) -> GateDecision {
        (**self).check(raw_key)
    }

    fn check_version(&self, raw_key: &[u8], version_seq: Seq) -> GateDecision {
        (**self).check_version(raw_key, version_seq)
    }

    fn check_prefix(&self, prefix: &[u8]) -> RangeGate {
        (**self).check_prefix(prefix)
    }

    fn discard_rule(&self, raw_key: &[u8], version_seq: Seq) -> DiscardRule {
        (**self).discard_rule(raw_key, version_seq)
    }

    fn check_doc(&self, tenant: TenantId, collection: &str, doc: DocId) -> GateDecision {
        (**self).check_doc(tenant, collection, doc)
    }

    fn gated_seq(&self, raw_key: &[u8]) -> Option<Seq> {
        (**self).gated_seq(raw_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_decision_is_blocked() {
        assert!(!GateDecision::Open.is_blocked());
        assert!(GateDecision::Restricted.is_blocked());
        assert!(GateDecision::Erased.is_blocked());
    }
}
