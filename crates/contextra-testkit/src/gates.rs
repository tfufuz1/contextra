//! # Test-Gates (`gates`)
//!
//! [`ReadGate`]-Implementierungen für Tests. `ports` liefert bewusst keinen Default
//! (Spezifikation 02 §4.1); jede Implementierung hier definiert **alle** Methoden (G-6).

use contextra_ports::gate::{DiscardRule, GateDecision, RangeGate, ReadGate};
use contextra_types::{DocId, Seq, TenantId};
use parking_lot::Mutex;
use std::sync::Arc;

/// Gate ohne aktive Sperren.
#[derive(Debug, Clone, Copy, Default)]
pub struct AlwaysOpenGate;

impl ReadGate for AlwaysOpenGate {
    fn is_active(&self) -> bool {
        false
    }
    fn epoch(&self) -> u64 {
        0
    }
    fn check(&self, _raw_key: &[u8]) -> GateDecision {
        GateDecision::Open
    }
    fn check_version(&self, _raw_key: &[u8], _version_seq: Seq) -> GateDecision {
        GateDecision::Open
    }
    fn check_prefix(&self, _prefix: &[u8]) -> RangeGate {
        RangeGate::EntirelyOpen
    }
    fn discard_rule(&self, _raw_key: &[u8], _version_seq: Seq) -> DiscardRule {
        DiscardRule::Keep
    }
    fn check_doc(&self, _tenant: TenantId, _collection: &str, _doc: DocId) -> GateDecision {
        GateDecision::Open
    }
    fn gated_seq(&self, _raw_key: &[u8]) -> Option<Seq> {
        None
    }
}

/// Fail-closed-Gate: alles `Restricted`, nichts darf verworfen werden.
#[derive(Debug, Clone, Copy, Default)]
pub struct AlwaysRestrictedGate;

impl ReadGate for AlwaysRestrictedGate {
    fn is_active(&self) -> bool {
        true
    }
    fn epoch(&self) -> u64 {
        1
    }
    fn check(&self, _raw_key: &[u8]) -> GateDecision {
        GateDecision::Restricted
    }
    fn check_version(&self, _raw_key: &[u8], _version_seq: Seq) -> GateDecision {
        GateDecision::Restricted
    }
    fn check_prefix(&self, _prefix: &[u8]) -> RangeGate {
        RangeGate::EntirelyBlocked
    }
    fn discard_rule(&self, _raw_key: &[u8], _version_seq: Seq) -> DiscardRule {
        // Restricted ist reversibel: niemals physisch verwerfen.
        DiscardRule::Keep
    }
    fn check_doc(&self, _tenant: TenantId, _collection: &str, _doc: DocId) -> GateDecision {
        GateDecision::Restricted
    }
    fn gated_seq(&self, _raw_key: &[u8]) -> Option<Seq> {
        Some(Seq(u64::MAX))
    }
}

/// Protokollierter Gate-Aufruf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateCall {
    /// `is_active`.
    IsActive,
    /// `epoch`.
    Epoch,
    /// `check`.
    Check(Vec<u8>),
    /// `check_version`.
    CheckVersion(Vec<u8>, Seq),
    /// `check_prefix`.
    CheckPrefix(Vec<u8>),
    /// `discard_rule`.
    DiscardRule(Vec<u8>, Seq),
    /// `check_doc`.
    CheckDoc(TenantId, String, DocId),
    /// `gated_seq`.
    GatedSeq(Vec<u8>),
}

/// Zeichnet jeden Aufruf auf und delegiert an ein inneres Gate. `Clone` teilt das Protokoll.
#[derive(Clone)]
pub struct RecordingGate {
    inner: Arc<dyn ReadGate>,
    calls: Arc<Mutex<Vec<GateCall>>>,
}

impl RecordingGate {
    /// Umhüllt `inner`.
    pub fn new(inner: Arc<dyn ReadGate>) -> Self {
        Self {
            inner,
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Bisherige Aufrufe in Reihenfolge.
    pub fn calls(&self) -> Vec<GateCall> {
        self.calls.lock().clone()
    }

    fn record(&self, call: GateCall) {
        self.calls.lock().push(call);
    }
}

impl ReadGate for RecordingGate {
    fn is_active(&self) -> bool {
        self.record(GateCall::IsActive);
        self.inner.is_active()
    }
    fn epoch(&self) -> u64 {
        self.record(GateCall::Epoch);
        self.inner.epoch()
    }
    fn check(&self, raw_key: &[u8]) -> GateDecision {
        self.record(GateCall::Check(raw_key.to_vec()));
        self.inner.check(raw_key)
    }
    fn check_version(&self, raw_key: &[u8], version_seq: Seq) -> GateDecision {
        self.record(GateCall::CheckVersion(raw_key.to_vec(), version_seq));
        self.inner.check_version(raw_key, version_seq)
    }
    fn check_prefix(&self, prefix: &[u8]) -> RangeGate {
        self.record(GateCall::CheckPrefix(prefix.to_vec()));
        self.inner.check_prefix(prefix)
    }
    fn discard_rule(&self, raw_key: &[u8], version_seq: Seq) -> DiscardRule {
        self.record(GateCall::DiscardRule(raw_key.to_vec(), version_seq));
        self.inner.discard_rule(raw_key, version_seq)
    }
    fn check_doc(&self, tenant: TenantId, collection: &str, doc: DocId) -> GateDecision {
        self.record(GateCall::CheckDoc(tenant, collection.to_string(), doc));
        self.inner.check_doc(tenant, collection, doc)
    }
    fn gated_seq(&self, raw_key: &[u8]) -> Option<Seq> {
        self.record(GateCall::GatedSeq(raw_key.to_vec()));
        self.inner.gated_seq(raw_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_gate_delegates_and_records() {
        let g = RecordingGate::new(Arc::new(AlwaysRestrictedGate));
        assert!(g.check(b"k").is_blocked());
        assert_eq!(g.discard_rule(b"k", Seq(1)), DiscardRule::Keep);
        assert_eq!(
            g.calls(),
            vec![
                GateCall::Check(b"k".to_vec()),
                GateCall::DiscardRule(b"k".to_vec(), Seq(1))
            ]
        );
        assert!(!AlwaysOpenGate.is_active());
        assert_eq!(AlwaysOpenGate.check(b"k"), GateDecision::Open);
    }
}
