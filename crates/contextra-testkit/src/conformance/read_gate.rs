//! # Konformitätsprüfsuite für den `ReadGate`-Port (`conformance/read_gate`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen (Contract Invariants)
//! für den Lese- und Suchfilter [`ReadGate`] gemäß SPEC §11.3, PO-07, PO-04, D-12, I-18 und D-57.
//!
//! ## Getestete Verträge (Pflichtfälle):
//! 1. `po_07_check_without_io`: Synchrones Prüfen ohne I/O in O(log n).
//! 2. `po_04_erased_restricted_open`: Vollständige Abdeckung der Gate-Zustände (`Open`, `Erased`, `Restricted`).
//! 3. `po_04_variant_for_range_restricted_never_dropped`: Invariant I-18: `Restricted` wird bei Bereichsprüfung nie verworfen.
//! 4. `po_04_gated_seq`: Liefert `gated_seq` für Restore-Guard und Commit-Abweisung (D-57).

use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Key representation in ReadGate (78 bytes canonical buffer).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyBuf(pub [u8; 78]);

impl KeyBuf {
    /// Creates a key buffer filled with a prefix.
    #[must_use]
    pub fn from_prefix(prefix: &[u8]) -> Self {
        let mut buf = [0u8; 78];
        let copy_len = prefix.len().min(78);
        if let (Some(dst), Some(src)) = (buf.get_mut(..copy_len), prefix.get(..copy_len)) {
            dst.copy_from_slice(src);
        }
        Self(buf)
    }
}

/// Boundary key for file and compaction boundaries (65 bytes canonical buffer).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyBound(pub [u8; 65]);

impl KeyBound {
    /// Creates a key boundary filled with a prefix.
    #[must_use]
    pub fn from_prefix(prefix: &[u8]) -> Self {
        let mut buf = [0u8; 65];
        let copy_len = prefix.len().min(65);
        if let (Some(dst), Some(src)) = (buf.get_mut(..copy_len), prefix.get(..copy_len)) {
            dst.copy_from_slice(src);
        }
        Self(buf)
    }
}

/// Key range between lower and upper bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRange {
    /// Lower boundary (inclusive).
    pub lo: KeyBound,
    /// Upper boundary (inclusive).
    pub hi: KeyBound,
}

/// Monotonic commit sequence number.
pub type Seq = u64;

/// Decision for individual keys or bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateDecision {
    /// Key is open for reading.
    Open,
    /// Key is permanently erased.
    Erased,
    /// Key is restricted (legal hold, retention).
    Restricted,
}

/// Range gate variant for compaction / range scans (I-18).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeGate {
    /// Range contains only open keys.
    Open,
    /// Range is erased and can be discarded during compaction.
    Erased,
    /// Range is restricted and must NEVER be discarded during compaction (I-18).
    Restricted,
}

/// ReadGate Port Trait according to SPEC §11.3 & PO-07.
pub trait ReadGate: Send + Sync {
    /// Checks decision for an exact key buffer synchronously without I/O.
    fn check(&self, k: &KeyBuf) -> GateDecision;
    /// Checks decision for a boundary key synchronously.
    fn check_bound(&self, b: &KeyBound) -> GateDecision;
    /// Resolves range gate behavior for compaction: Restricted is never dropped (I-18).
    fn variant_for_range(&self, r: &KeyRange) -> RangeGate;
    /// Returns the sequence number from which the gate became effective.
    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq>;
}

/// Conformance failure descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceFailure {
    /// Suite identifier.
    pub suite_name: &'static str,
    /// Test case.
    pub test_case: &'static str,
    /// Failure reason.
    pub message: String,
}

impl ConformanceFailure {
    /// Creates a new failure.
    #[must_use]
    pub fn new(
        suite_name: &'static str,
        test_case: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            suite_name,
            test_case,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ConformanceFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] Conformance failure in test '{}': {}",
            self.suite_name, self.test_case, self.message
        )
    }
}

impl std::error::Error for ConformanceFailure {}

/// Result type.
pub type ConformanceResult = std::result::Result<(), ConformanceFailure>;

/// Führt die vollständige Konformitätsprüfung für eine [`ReadGate`]-Implementierung über eine Factory durch.
pub fn run<F, R>(factory: F) -> ConformanceResult
where
    F: Fn() -> R,
    R: ReadGate,
{
    let gate = factory();
    run_on_gate(&gate)
}

/// Führt die Konformitätsprüfung direkt auf einer Referenz durch.
#[allow(clippy::disallowed_methods)]
pub fn run_on_gate<R: ReadGate>(gate: &R) -> ConformanceResult {
    let suite = "ReadGate";

    let k_open = KeyBuf::from_prefix(b"tenant1:open_key");
    let k_erased = KeyBuf::from_prefix(b"tenant1:erased_key");
    let k_restricted = KeyBuf::from_prefix(b"tenant1:restricted_key");

    // Fall 1: PO-07 check ohne I/O (synchron, schnell)
    {
        let t0 = std::time::Instant::now();
        for _ in 0..10_000 {
            let _ = gate.check(&k_open);
        }
        let elapsed = t0.elapsed();
        // 10.000 checks in memory must be sub-second without disk I/O
        if elapsed.as_millis() > 500 {
            return Err(ConformanceFailure::new(
                suite,
                "po_07_check_without_io",
                format!("check took too long ({elapsed:?}), likely performing I/O"),
            ));
        }
    }

    // Fall 5: PO-04 check_bound
    {
        let bound_open = KeyBound::from_prefix(b"tenant1:open_key");
        let bound_erased = KeyBound::from_prefix(b"tenant1:erased_key");
        if gate.check_bound(&bound_open) != GateDecision::Open {
            return Err(ConformanceFailure::new(
                suite,
                "po_04_erased_restricted_open",
                "bound_open did not return GateDecision::Open",
            ));
        }
        if gate.check_bound(&bound_erased) != GateDecision::Erased {
            return Err(ConformanceFailure::new(
                suite,
                "po_04_erased_restricted_open",
                "bound_erased did not return GateDecision::Erased",
            ));
        }
    }

    // Fall 2: PO-04 / D-12 Erased/Restricted/Open
    {
        if gate.check(&k_open) != GateDecision::Open {
            return Err(ConformanceFailure::new(
                suite,
                "po_04_erased_restricted_open",
                "k_open did not return GateDecision::Open",
            ));
        }
        if gate.check(&k_erased) != GateDecision::Erased {
            return Err(ConformanceFailure::new(
                suite,
                "po_04_erased_restricted_open",
                "k_erased did not return GateDecision::Erased",
            ));
        }
        if gate.check(&k_restricted) != GateDecision::Restricted {
            return Err(ConformanceFailure::new(
                suite,
                "po_04_erased_restricted_open",
                "k_restricted did not return GateDecision::Restricted",
            ));
        }
    }

    // Fall 3: PO-04 / I-18 variant_for_range: Restricted nie verworfen
    {
        let range_restricted = KeyRange {
            lo: KeyBound::from_prefix(b"tenant1:restricted_key"),
            hi: KeyBound::from_prefix(b"tenant1:restricted_key_end"),
        };
        match gate.variant_for_range(&range_restricted) {
            RangeGate::Restricted => {}
            other => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_04_variant_for_range_restricted_never_dropped",
                    format!("Restricted range returned {other:?}, violating invariant I-18"),
                ));
            }
        }

        let range_erased = KeyRange {
            lo: KeyBound::from_prefix(b"tenant1:erased_key"),
            hi: KeyBound::from_prefix(b"tenant1:erased_key_end"),
        };
        if gate.variant_for_range(&range_erased) != RangeGate::Erased {
            return Err(ConformanceFailure::new(
                suite,
                "po_04_variant_for_range_restricted_never_dropped",
                "Erased range did not return RangeGate::Erased",
            ));
        }
    }

    // Fall 4: PO-04 / D-57 gated_seq
    {
        let seq_erased = gate.gated_seq(&k_erased);
        if seq_erased.is_none() {
            return Err(ConformanceFailure::new(
                suite,
                "po_04_gated_seq",
                "gated_seq for erased key returned None",
            ));
        }
        let seq_open = gate.gated_seq(&k_open);
        if seq_open.is_some() {
            return Err(ConformanceFailure::new(
                suite,
                "po_04_gated_seq",
                format!("gated_seq for open key returned {seq_open:?}, expected None"),
            ));
        }
    }

    Ok(())
}

// ============================================================================
// REFERENCE IMPLEMENTATION (IN-MEMORY)
// ============================================================================

/// In-Memory Reference Implementation of [`ReadGate`].
#[derive(Clone, Default)]
pub struct ReferenceReadGate {
    entries: Arc<RwLock<BTreeMap<KeyBuf, (GateDecision, Seq)>>>,
    ranges: Arc<RwLock<Vec<(KeyRange, RangeGate)>>>,
}

impl ReferenceReadGate {
    /// Creates a pre-populated reference gate satisfying conformance checks.
    #[must_use]
    pub fn new() -> Self {
        let gate = Self::default();
        gate.add_key(
            KeyBuf::from_prefix(b"tenant1:erased_key"),
            GateDecision::Erased,
            42,
        );
        gate.add_key(
            KeyBuf::from_prefix(b"tenant1:restricted_key"),
            GateDecision::Restricted,
            43,
        );
        gate.add_range(
            KeyRange {
                lo: KeyBound::from_prefix(b"tenant1:restricted_key"),
                hi: KeyBound::from_prefix(b"tenant1:restricted_key_end"),
            },
            RangeGate::Restricted,
        );
        gate.add_range(
            KeyRange {
                lo: KeyBound::from_prefix(b"tenant1:erased_key"),
                hi: KeyBound::from_prefix(b"tenant1:erased_key_end"),
            },
            RangeGate::Erased,
        );
        gate
    }

    /// Adds a key decision.
    pub fn add_key(&self, k: KeyBuf, d: GateDecision, seq: Seq) {
        self.entries.write().insert(k, (d, seq));
    }

    /// Adds a range decision.
    pub fn add_range(&self, r: KeyRange, g: RangeGate) {
        self.ranges.write().push((r, g));
    }
}

impl ReadGate for ReferenceReadGate {
    fn check(&self, k: &KeyBuf) -> GateDecision {
        self.entries
            .read()
            .get(k)
            .map(|(d, _)| *d)
            .unwrap_or(GateDecision::Open)
    }

    fn check_bound(&self, b: &KeyBound) -> GateDecision {
        let guard = self.entries.read();
        for (k, (d, _)) in guard.iter() {
            if k.0[..65] == b.0 {
                return *d;
            }
        }
        GateDecision::Open
    }

    fn variant_for_range(&self, r: &KeyRange) -> RangeGate {
        let guard = self.ranges.read();
        for (range, gate) in guard.iter() {
            if range.lo == r.lo && range.hi == r.hi {
                return *gate;
            }
        }
        RangeGate::Open
    }

    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq> {
        self.entries.read().get(k).map(|(_, s)| *s)
    }
}

// ============================================================================
// MUTANTS (INTENTIONALLY DEFECTIVE IMPLEMENTATIONS)
// ============================================================================

/// Mutant 1: violates I-18 by returning Erased for Restricted ranges.
#[derive(Clone, Default)]
pub struct MutantDropRestrictedReadGate(ReferenceReadGate);

impl ReadGate for MutantDropRestrictedReadGate {
    fn check(&self, k: &KeyBuf) -> GateDecision {
        self.0.check(k)
    }
    fn check_bound(&self, b: &KeyBound) -> GateDecision {
        self.0.check_bound(b)
    }
    fn variant_for_range(&self, _r: &KeyRange) -> RangeGate {
        // DEFECT: Drop Restricted, treat all ranges as Erased (violates I-18)!
        RangeGate::Erased
    }
    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq> {
        self.0.gated_seq(k)
    }
}

/// Mutant 2: never returns a gated_seq.
#[derive(Clone, Default)]
pub struct MutantMissingGatedSeqReadGate(ReferenceReadGate);

impl ReadGate for MutantMissingGatedSeqReadGate {
    fn check(&self, k: &KeyBuf) -> GateDecision {
        self.0.check(k)
    }
    fn check_bound(&self, b: &KeyBound) -> GateDecision {
        self.0.check_bound(b)
    }
    fn variant_for_range(&self, r: &KeyRange) -> RangeGate {
        self.0.variant_for_range(r)
    }
    fn gated_seq(&self, _k: &KeyBuf) -> Option<Seq> {
        // DEFECT: Always returns None!
        None
    }
}

/// Helper executing reference test.
pub fn run_reference_test() -> Result<(), ConformanceFailure> {
    run(ReferenceReadGate::new)
}

/// Helper executing mutant tests, ensuring 100% detection rate.
pub fn run_mutant_tests() -> std::result::Result<(), String> {
    let res1 = run(|| MutantDropRestrictedReadGate(ReferenceReadGate::new()));
    if res1.is_ok() {
        return Err("MutantDropRestrictedReadGate escaped detection!".to_string());
    }

    let res2 = run(|| MutantMissingGatedSeqReadGate(ReferenceReadGate::new()));
    if res2.is_ok() {
        return Err("MutantMissingGatedSeqReadGate escaped detection!".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn po_07_read_gate_reference_passes() {
        let res = run_reference_test();
        assert!(res.is_ok(), "Reference must pass: {res:?}");
    }

    #[test]
    fn po_07_read_gate_mutants_fail() {
        let res = run_mutant_tests();
        assert!(res.is_ok(), "Mutants must be caught: {res:?}");
    }
}
