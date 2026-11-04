//! Type ordering, formatting, and debug redaction checks across ports types.

use contextra_ports::gate::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, Seq};
use contextra_ports::keys::{Envelope, FileId, FileKey, SealCtx};
use contextra_ports::ledger::{LedgerEntry, RangeSet, Timestamp};

/// SPEC: T-ARCH-6
#[test]
fn test_key_buf_lexicographical_ordering() {
    let arr1 = [1u8; 16];
    let arr2 = [2u8; 16];

    let k1 = KeyBuf::new(arr1);
    let k2 = KeyBuf::new(arr2);

    assert_eq!(k1.cmp(&k2), arr1.cmp(&arr2));
    assert!(k1 < k2);
    assert!(k2 > k1);
    assert_ne!(k1, k2);
}

/// SPEC: T-ARCH-6
#[test]
fn test_key_range_contains_invariants() {
    let lo = [1u8; 16];
    let hi = [10u8; 16];
    let probe_in = [5u8; 16];
    let probe_out = [20u8; 16];

    let range = KeyRange::new(KeyBuf::new(lo), KeyBuf::new(hi));
    assert!(range.contains(&KeyBuf::new(probe_in)));
    assert!(!range.contains(&KeyBuf::new(probe_out)));
}

/// SPEC: T-ARCH-6
#[test]
fn test_seq_ordering_and_monotonicity() {
    let seq1 = Seq::new(10);
    let seq2 = Seq::new(20);

    assert_eq!(seq1.cmp(&seq2), 10.cmp(&20));
    assert_eq!(seq1.as_u64(), 10);
}

/// SPEC: T-ARCH-6
#[test]
fn test_file_key_zeroize_on_drop() {
    let secret = vec![0x42u8; 32];
    let key = FileKey::new(secret.clone(), FileId::new(7));
    assert_eq!(key.as_bytes(), secret.as_slice());
    assert_eq!(key.file_id(), FileId::new(7));

    // Ensure Debug does not leak raw bytes
    let debug = format!("{:?}", key);
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("42424242"));
}

/// SPEC: T-ARCH-6
#[test]
fn test_ledger_chain_hash_golden_invariance() {
    let prev_hash = [0x55u8; 32];
    let commitment = [0x77u8; 32];
    let range_set = RangeSet::empty();
    let gated_seq = Seq::new(1000);
    let timestamp = Timestamp::new(1700000000);
    let mac = [0x99u8; 32];

    let entry1 = LedgerEntry::compute_chained(
        commitment,
        range_set.clone(),
        gated_seq,
        timestamp,
        prev_hash,
        mac,
    );

    let entry2 =
        LedgerEntry::compute_chained(commitment, range_set, gated_seq, timestamp, prev_hash, mac);

    assert_eq!(entry1.chain_hash, entry2.chain_hash);
    assert!(entry1.verify_chain_hash(&prev_hash));
}

/// SPEC: T-ARCH-6
#[test]
fn test_debug_redaction_across_ports_types() {
    let k = KeyBuf::new([0x42; 16]);
    assert_eq!(format!("{:?}", k), "KeyBuf([REDACTED])");

    let bound = KeyBound::exact(k);
    assert_eq!(format!("{:?}", bound), "KeyBound::Exact([REDACTED])");

    let range = KeyRange::new(k, KeyBuf::new([0x99; 16]));
    assert_eq!(format!("{:?}", range), "KeyRange { [REDACTED] }");

    let env = Envelope::new(vec![1, 2, 3, 4, 5]);
    assert!(format!("{:?}", env).contains("[REDACTED, len=5]"));

    let ctx = SealCtx::new(1234);
    assert_eq!(format!("{:?}", ctx), "SealCtx { epoch: 1234 }");
}

/// SPEC: T-ARCH-6
#[test]
fn test_gate_decision_variants() {
    assert_eq!(GateDecision::Open, GateDecision::Open);
    assert_ne!(GateDecision::Open, GateDecision::Erased);
    assert_ne!(GateDecision::Open, GateDecision::Restricted);

    assert_eq!(RangeGate::EntirelyOpen, RangeGate::EntirelyOpen);
    assert_eq!(RangeGate::ContainsErased, RangeGate::ContainsErased);
    assert_eq!(
        RangeGate::RestrictedNeverDiscard,
        RangeGate::RestrictedNeverDiscard
    );
}
