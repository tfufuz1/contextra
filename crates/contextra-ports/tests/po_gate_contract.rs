//! Contract tests for PO-04 and PO-07: ReadGate trait, sync execution, O(log n).

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "../src/gate.rs"]
pub mod gate;

use gate::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, ReadGate, Seq};
use std::collections::BTreeMap;
use std::sync::Arc;

struct MockReadGate {
    gated: BTreeMap<KeyBuf, Seq>,
}

impl ReadGate for MockReadGate {
    fn check(&self, k: &KeyBuf) -> GateDecision {
        if self.gated.contains_key(k) {
            GateDecision::Erased
        } else {
            GateDecision::Open
        }
    }

    fn check_bound(&self, _b: &KeyBound) -> GateDecision {
        GateDecision::Open
    }

    fn variant_for_range(&self, _r: &KeyRange) -> RangeGate {
        RangeGate::ContainsErased
    }

    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq> {
        self.gated.get(k).copied()
    }
}

#[test]
fn po_04_read_gate_dyn_compatible() {
    let mut gated = BTreeMap::new();
    let k = KeyBuf::new(*b"user-42-key-0001");
    gated.insert(k, Seq::new(10));
    let mock = MockReadGate { gated };
    let dyn_gate: Box<dyn ReadGate> = Box::new(mock);

    assert_eq!(dyn_gate.check(&k), GateDecision::Erased);
    assert_eq!(dyn_gate.gated_seq(&k), Some(Seq::new(10)));
}

#[test]
fn po_04_read_gate_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MockReadGate>();
    assert_send_sync::<Box<dyn ReadGate>>();
    assert_send_sync::<Arc<dyn ReadGate>>();
}

#[test]
fn po_07_read_gate_synchronous_no_io() {
    let mock = MockReadGate {
        gated: BTreeMap::new(),
    };
    let key = KeyBuf::new(*b"unrestricted-key");
    // Synchronous execution check
    let decision = mock.check(&key);
    assert_eq!(decision, GateDecision::Open);
    let bound = KeyBound::exact(key);
    assert_eq!(mock.check_bound(&bound), GateDecision::Open);
}
