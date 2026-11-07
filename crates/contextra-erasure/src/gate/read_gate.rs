//! Implementierung des ReadGate-Port-Traits für Lesesperren (SPEC §11.3, D-12, ER-11, D-120, I-21).
//!
//! `GateGuard` ist die Arc-kompatible Implementierung von `ReadGate`,
//! die auf einem `ErasedSet` aufbaut und Lese-Linearisierung unterstützt.

use super::erased_set::ErasedSet;
use crate::types_local::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, ReadGate, Seq};
use parking_lot::RwLock;
use std::sync::Arc;

/// Thread-sichere Implementierung des `ReadGate`-Ports (ER-11, D-120).
pub struct GateGuard {
    inner: Arc<RwLock<ErasedSet>>,
}

impl GateGuard {
    /// Erstellt einen neuen Gate-Wächter.
    pub fn new(set: Arc<RwLock<ErasedSet>>) -> Self {
        Self { inner: set }
    }

    /// Führt einen linearisierten Lesevorgang aus (SPEC D-120, I-21).
    ///
    /// Prüft das Gate vor und nach der übergebenen Lese-/Entschlüsselungs-Operation `f`.
    /// Hat sich `gate_epoch` während der Ausführung verändert, wird die Operation
    /// erneut ausgeführt, um sicherzustellen, dass keine Daten eines Ziels ausgeliefert
    /// werden, dessen Gate währenddessen aktiv wurde.
    pub fn linearized_read<T, F>(&self, k: &KeyBuf, mut f: F) -> Result<T, GateDecision>
    where
        F: FnMut() -> Result<T, GateDecision>,
    {
        loop {
            let epoch_before = self.gate_epoch();
            let decision_before = self.check(k);
            if decision_before != GateDecision::Open {
                return Err(decision_before);
            }

            let result = f()?;

            let epoch_after = self.gate_epoch();
            if epoch_after != epoch_before {
                // Epoche geändert: erneute Prüfung (I-21)
                let decision_after = self.check(k);
                if decision_after != GateDecision::Open {
                    return Err(decision_after);
                }
                // Bei unverändertem GateDecision::Open aber geänderter Epoche wiederholen
                continue;
            }

            return Ok(result);
        }
    }

    /// Führt einen linearisierten Lesevorgang auf einem 78-Byte-Schlüssel aus (SPEC D-120, D-123, I-21).
    pub fn linearized_read_raw<T, F>(&self, raw: &[u8; 78], mut f: F) -> Result<T, GateDecision>
    where
        F: FnMut() -> Result<T, GateDecision>,
    {
        loop {
            let epoch_before = self.gate_epoch();
            let decision_before = self.check_raw(raw);
            if decision_before != GateDecision::Open {
                return Err(decision_before);
            }

            let result = f()?;

            let epoch_after = self.gate_epoch();
            if epoch_after != epoch_before {
                let decision_after = self.check_raw(raw);
                if decision_after != GateDecision::Open {
                    return Err(decision_after);
                }
                continue;
            }

            return Ok(result);
        }
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

    fn gate_epoch(&self) -> u64 {
        self.inner.read().gate_epoch()
    }

    fn covers(&self, k: &KeyBuf) -> bool {
        self.inner.read().covers(k)
    }

    fn check_raw(&self, raw: &[u8; 78]) -> GateDecision {
        self.inner.read().check_raw(raw)
    }

    fn covers_raw(&self, raw: &[u8; 78]) -> bool {
        self.inner.read().covers_raw(raw)
    }

    fn gated_seq_raw(&self, raw: &[u8; 78]) -> Option<Seq> {
        self.inner.read().gated_seq_raw(raw)
    }
}
