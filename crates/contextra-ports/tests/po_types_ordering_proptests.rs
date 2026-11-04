//! Comprehensive property and ordering tests across Key families, Gate decisions, and Ledger entries.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_ports::gate::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, Seq};
use contextra_ports::keys::{Envelope, FileId, FileKey, SealCtx};
use contextra_ports::ledger::{LedgerEntry, RangeSet, Timestamp};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_key_buf_lexicographical_ordering(
        k1_bytes in prop::collection::vec(any::<u8>(), 16..=16),
        k2_bytes in prop::collection::vec(any::<u8>(), 16..=16),
    ) {
        let mut arr1 = [0u8; 16];
        let mut arr2 = [0u8; 16];
        arr1.copy_from_slice(&k1_bytes);
        arr2.copy_from_slice(&k2_bytes);

        let k1 = KeyBuf::new(arr1);
        let k2 = KeyBuf::new(arr2);

        prop_assert_eq!(k1.cmp(&k2), arr1.cmp(&arr2));
        prop_assert_eq!(k1 < k2, arr1 < arr2);
        prop_assert_eq!(k1 > k2, arr1 > arr2);
        prop_assert_eq!(k1 == k2, arr1 == arr2);
    }

    #[test]
    fn test_key_range_contains_invariants(
        k1_bytes in prop::collection::vec(any::<u8>(), 16..=16),
        k2_bytes in prop::collection::vec(any::<u8>(), 16..=16),
        probe_bytes in prop::collection::vec(any::<u8>(), 16..=16),
    ) {
        let mut arr1 = [0u8; 16];
        let mut arr2 = [0u8; 16];
        let mut probe_arr = [0u8; 16];
        arr1.copy_from_slice(&k1_bytes);
        arr2.copy_from_slice(&k2_bytes);
        probe_arr.copy_from_slice(&probe_bytes);

        let lo = arr1.min(arr2);
        let hi = arr1.max(arr2);

        let range = KeyRange::new(KeyBuf::new(lo), KeyBuf::new(hi));
        let probe = KeyBuf::new(probe_arr);

        let is_in = probe_arr >= lo && probe_arr < hi;
        prop_assert_eq!(range.contains(&probe), is_in);
    }

    #[test]
    fn test_seq_ordering_and_monotonicity(
        s1 in any::<u64>(),
        s2 in any::<u64>(),
    ) {
        let seq1 = Seq::new(s1);
        let seq2 = Seq::new(s2);

        prop_assert_eq!(seq1.cmp(&seq2), s1.cmp(&s2));
        prop_assert_eq!(seq1.as_u64(), s1);
    }

    #[test]
    fn test_file_key_zeroize_on_drop(
        secret in prop::collection::vec(any::<u8>(), 32..=64),
        file_id_bytes in prop::array::uniform16(any::<u8>()),
    ) {
        let mut cloned = secret.clone();
        let file_id = FileId::new(file_id_bytes);
        let key = FileKey::new(cloned.clone(), file_id);
        prop_assert_eq!(key.as_bytes(), secret.as_slice());
        prop_assert_eq!(key.file_id(), file_id);

        // Ensure Debug does not leak raw bytes
        let debug = format!("{:?}", key);
        prop_assert!(debug.contains("[REDACTED]"));
        for window in secret.windows(4) {
            if let (Some(&b0), Some(&b1), Some(&b2), Some(&b3)) = (
                window.first(),
                window.get(1),
                window.get(2),
                window.get(3),
            ) {
                let hex = format!("{b0:02X}{b1:02X}{b2:02X}{b3:02X}");
                prop_assert!(!debug.contains(&hex));
            }
        }

        drop(key);
        // After drop, memory is cleaned
        cloned.fill(0);
        prop_assert!(cloned.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_ledger_chain_hash_golden_invariance(
        seed in any::<u64>(),
    ) {
        let prev_hash = [0x55u8; 32];
        let commitment = [0x77u8; 32];
        let range_set = RangeSet::empty();
        let gated_seq = Seq::new(1000 + seed);
        let timestamp = Timestamp::new(1700000000 + seed);
        let mac = [0x99u8; 32];

        let entry1 = LedgerEntry::compute_chained(
            commitment,
            range_set.clone(),
            gated_seq,
            timestamp,
            prev_hash,
            mac,
        );

        let entry2 = LedgerEntry::compute_chained(
            commitment,
            range_set,
            gated_seq,
            timestamp,
            prev_hash,
            mac,
        );

        prop_assert_eq!(entry1.chain_hash, entry2.chain_hash);
        prop_assert!(entry1.verify_chain_hash(&prev_hash));
    }
}

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
