//! Tests für Gate-Subsystem (T-GATE-1, T-RESTR-1).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::gate::erased_set::ErasedSet;
use contextra_erasure::types_local::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, Seq};

#[test]
fn t_gate_1_erased_set_checks_and_compaction_policy() {
    let mut gate = ErasedSet::new();
    let k_open = KeyBuf::new([0x01; 16]);
    let k_erased = KeyBuf::new([0x05; 16]);
    let k_restricted = KeyBuf::new([0x09; 16]);

    assert_eq!(gate.check(&k_open), GateDecision::Open);
    assert_eq!(gate.check(&k_erased), GateDecision::Open);
    assert_eq!(gate.check(&k_restricted), GateDecision::Open);

    gate.add_erased(
        KeyBuf::new([0x04; 16]),
        KeyBuf::new([0x06; 16]),
        Seq::new(100),
    );

    gate.add_restricted(
        KeyBuf::new([0x08; 16]),
        KeyBuf::new([0x0a; 16]),
        Seq::new(105),
    );

    assert_eq!(gate.check(&k_open), GateDecision::Open);
    assert_eq!(gate.check(&k_erased), GateDecision::Erased);
    assert_eq!(gate.check(&k_restricted), GateDecision::Restricted);
    assert_eq!(gate.gated_seq(&k_erased), Some(Seq::new(100)));
    assert_eq!(gate.gated_seq(&k_restricted), Some(Seq::new(105)));

    assert_eq!(
        gate.check_bound(&KeyBound::Exact(k_erased)),
        GateDecision::Erased
    );
    assert_eq!(
        gate.check_bound(&KeyBound::Exact(k_restricted)),
        GateDecision::Restricted
    );
    assert_eq!(
        gate.check_bound(&KeyBound::Exact(k_open)),
        GateDecision::Open
    );

    let r_open = KeyRange::new(KeyBuf::new([0x00; 16]), KeyBuf::new([0x02; 16]));
    assert_eq!(gate.variant_for_range(&r_open), RangeGate::EntirelyOpen);

    let r_erased = KeyRange::new(KeyBuf::new([0x03; 16]), KeyBuf::new([0x07; 16]));
    assert_eq!(gate.variant_for_range(&r_erased), RangeGate::ContainsErased);

    let r_restr = KeyRange::new(KeyBuf::new([0x07; 16]), KeyBuf::new([0x0b; 16]));
    assert_eq!(
        gate.variant_for_range(&r_restr),
        RangeGate::RestrictedNeverDiscard
    );
}

#[test]
fn t_restr_1_lift_restriction() {
    let mut gate = ErasedSet::new();
    let k = KeyBuf::new([0x05; 16]);
    let start = KeyBuf::new([0x04; 16]);
    let end = KeyBuf::new([0x06; 16]);

    gate.add_restricted(start, end, Seq::new(50));
    assert_eq!(gate.check(&k), GateDecision::Restricted);

    gate.lift_restriction(&start, &end);
    assert_eq!(gate.check(&k), GateDecision::Open);
}
