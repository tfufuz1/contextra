//! ErasedSet nach SPEC §11.3, PO-07: Intervallbaum, O(log n), synchron, ohne I/O.

use contextra_ports::gate::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, ReadGate, Seq};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Art der Torsperre für ein Intervall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateKind {
    /// Dauerhaft gelöscht: Lesen blockiert, Kompaktion darf verwerfen
    Erased,
    /// Vorübergehend restringiert: Lesen blockiert, Kompaktion darf NIE verwerfen (I-18)
    Restricted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IntervalEntry {
    end: KeyBuf,
    kind: GateKind,
    seq: Seq,
}

/// Speicherbasierte Bereichsmenge als geordneter Baum (O(log n) Suchzeit, keine I/O).
#[derive(Debug, Clone, Default)]
pub struct ErasedSet {
    /// BTreeMap sortiert nach Intervall-Start
    intervals: BTreeMap<KeyBuf, IntervalEntry>,
}

impl ErasedSet {
    /// Erstellt eine leere Tormenge.
    pub fn new() -> Self {
        Self {
            intervals: BTreeMap::new(),
        }
    }

    /// Fügt einen gelöschten Bereich [start, end) mit Sequenznummer hinzu.
    pub fn add_erased(&mut self, start: KeyBuf, end: KeyBuf, seq: Seq) {
        self.intervals.insert(
            start,
            IntervalEntry {
                end,
                kind: GateKind::Erased,
                seq,
            },
        );
    }

    /// Fügt einen restringierten Bereich [start, end) mit Sequenznummer hinzu.
    pub fn add_restricted(&mut self, start: KeyBuf, end: KeyBuf, seq: Seq) {
        self.intervals.insert(
            start,
            IntervalEntry {
                end,
                kind: GateKind::Restricted,
                seq,
            },
        );
    }

    /// Hebt eine Restriktion auf einem Intervall auf (ER-15).
    pub fn lift_restriction(&mut self, start: &KeyBuf, end: &KeyBuf) {
        if let Some(entry) = self.intervals.get(start) {
            if entry.end == *end && entry.kind == GateKind::Restricted {
                self.intervals.remove(start);
            }
        }
    }

    fn find_matching_entry(&self, k: &KeyBuf) -> Option<&IntervalEntry> {
        let mut range = self.intervals.range(..=k);
        if let Some((_start, entry)) = range.next_back() {
            if k.as_bytes() < entry.end.as_bytes() {
                return Some(entry);
            }
        }
        None
    }
}

impl ReadGate for ErasedSet {
    fn check(&self, k: &KeyBuf) -> GateDecision {
        match self.find_matching_entry(k) {
            Some(entry) => match entry.kind {
                GateKind::Erased => GateDecision::Erased,
                GateKind::Restricted => GateDecision::Restricted,
            },
            None => GateDecision::Open,
        }
    }

    fn check_bound(&self, b: &KeyBound) -> GateDecision {
        match b {
            KeyBound::Exact(k) => self.check(k),
            KeyBound::UnboundedLower | KeyBound::UnboundedUpper => GateDecision::Open,
        }
    }

    fn variant_for_range(&self, r: &KeyRange) -> RangeGate {
        let mut has_erased = false;
        let mut has_restricted = false;

        for (start, entry) in &self.intervals {
            let overlaps =
                start.as_bytes() < r.end.as_bytes() && entry.end.as_bytes() > r.start.as_bytes();
            if overlaps {
                match entry.kind {
                    GateKind::Restricted => has_restricted = true,
                    GateKind::Erased => has_erased = true,
                }
            }
        }

        if has_restricted {
            RangeGate::RestrictedNeverDiscard
        } else if has_erased {
            RangeGate::ContainsErased
        } else {
            RangeGate::EntirelyOpen
        }
    }

    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq> {
        self.find_matching_entry(k).map(|e| e.seq)
    }
}

/// Thread-sicherer Shared-Wrapper für `ReadGate`.
#[derive(Clone, Default)]
pub struct SharedReadGate {
    inner: Arc<RwLock<ErasedSet>>,
}

impl SharedReadGate {
    /// Erstellt ein neues SharedReadGate.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ErasedSet::new())),
        }
    }

    /// Gibt eine Referenz auf das innere ErasedSet zum Schreiben.
    pub fn write(&self) -> parking_lot::RwLockWriteGuard<'_, ErasedSet> {
        self.inner.write()
    }
}

impl ReadGate for SharedReadGate {
    fn check(&self, k: &KeyBuf) -> GateDecision {
        self.inner.read().check(k)
    }

    fn check_bound(&self, b: &KeyBound) -> GateDecision {
        self.inner.read().check_bound(b)
    }

    fn variant_for_range(&self, r: &KeyRange) -> RangeGate {
        self.inner.read().variant_for_range(r)
    }

    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq> {
        self.inner.read().gated_seq(k)
    }
}
