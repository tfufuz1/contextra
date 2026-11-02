//! Implementierung des ReadGate-Port-Traits für Lesesperren (SPEC §11.3, D-12, ER-11).
//!
//! `GateGuard` ist die Arc-kompatible Implementierung von `ReadGate`,
//! die auf einem `ErasedSet` aufbaut.

use super::erased_set::ErasedSet;
use contextra_ports::gate::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, ReadGate, Seq};
use parking_lot::RwLock;
use std::sync::Arc;

/// Thread-sichere Implementierung des `ReadGate`-Ports (ER-11).
pub struct GateGuard {
    inner: Arc<RwLock<ErasedSet>>,
}

impl GateGuard {
    /// Erstellt einen neuen Gate-Wächter.
    pub fn new(set: Arc<RwLock<ErasedSet>>) -> Self {
        Self { inner: set }
    }
}

impl ReadGate for GateGuard {
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
