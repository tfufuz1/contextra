// STAND: 2026-10-21
// ZWECK: Deterministic edge-case, permutation, and concurrent test suite for out-of-order MemTable sequence insertions (Card J04).
// ORAKEL: Invariant Zero-Panic and Snapshot Isolation — MemTable::put preserves sorted version order and correct get_at_seq visibility under all insertion permutations and concurrent access.

use bytes::Bytes;
use contextra_ports::Rng;
use contextra_testkit::TestkitSeededRng as SeededRng;
use contextra_store::memtable::{is_tombstone, MemTable};
use contextra_types::{TxId, TOMBSTONE_BIT};
use std::collections::BTreeMap;
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn test_all_permutations_of_4_puts_seq_1_5_5_10() {
    let items = [
        (1u64, Bytes::from("v1"), 100u64),
        (5u64, Bytes::from("v5a"), 200u64),
        (5u64, Bytes::from("v5b"), 300u64),
        (10u64, Bytes::from("v10"), 400u64),
    ];

    // Generate all 24 permutations of indices [0, 1, 2, 3]
    fn permutations(arr: Vec<usize>) -> Vec<Vec<usize>> {
        if arr.len() <= 1 {
            return vec![arr];
        }
        let mut res = Vec::new();
        for i in 0..arr.len() {
            let mut remaining = arr.clone();
            let current = remaining.remove(i);
            for sub in permutations(remaining) {
                let mut p = vec![current];
                p.extend(sub);
                res.push(p);
            }
        }
        res
    }

    let perm_indices = permutations(vec![0, 1, 2, 3]);
    assert_eq!(perm_indices.len(), 24);

    let key = Bytes::from("perm_key");

    for (perm_idx, p) in perm_indices.iter().enumerate() {
        let mt = MemTable::new();
        for &item_idx in p {
            let (seq, val, tx) = &items[item_idx];
            mt.put(key.clone(), val.clone(), *seq, *tx);
        }

        // 1. Verify latest version via get()
        let (latest_val, latest_seq) = mt
            .get(&key)
            .unwrap_or_else(|| panic!("Permutation {perm_idx} failed: latest key missing"));
        assert_eq!(
            latest_val.as_ref(),
            b"v10",
            "Permutation {perm_idx} latest val mismatch"
        );
        assert_eq!(
            latest_seq, 10,
            "Permutation {perm_idx} latest seq mismatch"
        );

        // 2. Verify get_at_seq across checkpoints
        // seq < 1 => None
        assert!(
            mt.get_at_seq(&key, 0, u64::MAX).is_none(),
            "Permutation {perm_idx} at seq 0"
        );

        // seq = 1 => v1
        let (v, s, _) = mt
            .get_at_seq(&key, 1, u64::MAX)
            .unwrap_or_else(|| panic!("Permutation {perm_idx} at seq 1"));
        assert_eq!(v.as_ref(), b"v1");
        assert_eq!(s, 1);

        // seq = 3 => v1
        let (v, s, _) = mt
            .get_at_seq(&key, 3, u64::MAX)
            .unwrap_or_else(|| panic!("Permutation {perm_idx} at seq 3"));
        assert_eq!(v.as_ref(), b"v1");
        assert_eq!(s, 1);

        // seq = 5 => latest inserted version at seq 5 (v5a or v5b depending on order)
        let (v, s, _) = mt
            .get_at_seq(&key, 5, u64::MAX)
            .unwrap_or_else(|| panic!("Permutation {perm_idx} at seq 5"));
        assert_eq!(s, 5);
        assert!(
            v.as_ref() == b"v5a" || v.as_ref() == b"v5b",
            "Permutation {perm_idx} at seq 5 value error: {:?}",
            v
        );

        // seq = 7 => same as seq 5
        let (v_7, s_7, _) = mt
            .get_at_seq(&key, 7, u64::MAX)
            .unwrap_or_else(|| panic!("Permutation {perm_idx} at seq 7"));
        assert_eq!(s_7, 5);
        assert_eq!(v_7, v);

        // seq = 10 => v10
        let (v, s, _) = mt
            .get_at_seq(&key, 10, u64::MAX)
            .unwrap_or_else(|| panic!("Permutation {perm_idx} at seq 10"));
        assert_eq!(v.as_ref(), b"v10");
        assert_eq!(s, 10);

        // seq = 15 => v10
        let (v, s, _) = mt
            .get_at_seq(&key, 15, u64::MAX)
            .unwrap_or_else(|| panic!("Permutation {perm_idx} at seq 15"));
        assert_eq!(v.as_ref(), b"v10");
        assert_eq!(s, 10);
    }
}

#[test]
fn test_edge_case_sequences_seq0_and_seq_max_and_tombstone() {
    let mt = MemTable::new();
    let key = Bytes::from("edge_key");

    // 1. Insert seq = 0
    mt.put(key.clone(), Bytes::from("v_seq0"), 0, 100);
    let (v0, s0, t0) = mt
        .get_at_seq(&key, 0, u64::MAX)
        .expect("seq 0 should be accessible");
    assert_eq!(v0.as_ref(), b"v_seq0");
    assert_eq!(s0, 0);
    assert_eq!(t0, 100);

    // 2. Insert out-of-order seq = 1000
    mt.put(key.clone(), Bytes::from("v_seq1000"), 1000, 200);

    // 3. Insert seq = u64::MAX & !TOMBSTONE_BIT (highest non-tombstone raw sequence)
    let max_raw_seq = u64::MAX & !TOMBSTONE_BIT;
    mt.put(key.clone(), Bytes::from("v_max_raw"), max_raw_seq, 300);

    let (v_max, s_max, _) = mt
        .get_at_seq(&key, max_raw_seq, u64::MAX)
        .expect("max_raw_seq should be accessible");
    assert_eq!(v_max.as_ref(), b"v_max_raw");
    assert_eq!(s_max, max_raw_seq);

    // 4. Insert tombstone at u64::MAX (has TOMBSTONE_BIT set, raw_seq = max_raw_seq)
    let tombstone_seq = u64::MAX;
    assert!(is_tombstone(tombstone_seq));
    mt.put(key.clone(), Bytes::new(), tombstone_seq, 400);

    let (v_tomb, s_tomb, _) = mt
        .get_at_seq(&key, max_raw_seq, u64::MAX)
        .expect("tombstone version should be accessible at max_raw_seq");
    assert_eq!(v_tomb.len(), 0);
    assert_eq!(s_tomb, tombstone_seq);
}

#[test]
fn test_duplicate_sequence_numbers_semantics() {
    let mt = MemTable::new();
    let key = Bytes::from("dup_key");

    // Insert three updates with identical raw sequence number (seq = 42) out-of-order relative to other sequence numbers
    mt.put(key.clone(), Bytes::from("v_seq_100"), 100, 10);
    mt.put(key.clone(), Bytes::from("v_seq_42_first"), 42, 1);
    mt.put(key.clone(), Bytes::from("v_seq_42_second"), 42, 2);
    mt.put(key.clone(), Bytes::from("v_seq_42_third"), 42, 3);
    mt.put(key.clone(), Bytes::from("v_seq_10"), 10, 1);

    // Query at seq 42: should return the latest inserted version at seq 42 ("v_seq_42_third")
    let (val_42, seq_42, tx_42) = mt
        .get_at_seq(&key, 42, u64::MAX)
        .expect("seq 42 entry must exist");
    assert_eq!(val_42.as_ref(), b"v_seq_42_third");
    assert_eq!(seq_42, 42);
    assert_eq!(tx_42, 3);

    // Filtered by max_tx = 2 at seq 42: should return "v_seq_42_second"
    let (val_tx2, _, tx) = mt
        .get_at_seq(&key, 42, 2)
        .expect("seq 42 max_tx 2 entry must exist");
    assert_eq!(val_tx2.as_ref(), b"v_seq_42_second");
    assert_eq!(tx, 2);
}

#[test]
fn test_concurrent_out_of_order_puts_and_snapshot_readers() {
    const NUM_WRITERS: usize = 4;
    const NUM_READERS: usize = 4;
    const ITERATIONS_PER_WRITER: usize = 200;
    const SEED: u64 = 0xDEAF_BEEF_42;

    println!("Executing test_concurrent_out_of_order_puts_and_snapshot_readers with Seed: {SEED:#X}");

    let mt = Arc::new(MemTable::new());
    let barrier = Arc::new(Barrier::new(NUM_WRITERS + NUM_READERS));
    let mut handles = Vec::new();

    // Writers inserting out-of-order sequence numbers for shared key "shared_key" and distinct keys
    for thread_id in 0..NUM_WRITERS {
        let mt_clone = Arc::clone(&mt);
        let barrier_clone = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            let rng = SeededRng::new(SEED + thread_id as u64);
            barrier_clone.wait();

            for i in 0..ITERATIONS_PER_WRITER {
                // Generate sequence number non-monotonically
                let raw_seq = (rng.next_u64() % 1000) + 1;
                let tx_id = (thread_id as u64 * 1000) + (i as u64 + 1);

                let key = if i % 2 == 0 {
                    Bytes::from("shared_key")
                } else {
                    Bytes::from(format!("writer_{thread_id}_key_{i}"))
                };

                let val = Bytes::from(format!("val_t{thread_id}_i{i}_seq{raw_seq}"));
                mt_clone.put(key, val, raw_seq, tx_id);
            }
        }));
    }

    // Readers scanning during concurrent writes
    for reader_id in 0..NUM_READERS {
        let mt_clone = Arc::clone(&mt);
        let barrier_clone = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            let rng = SeededRng::new(SEED + 100 + reader_id as u64);
            barrier_clone.wait();

            for _ in 0..ITERATIONS_PER_WRITER {
                let query_seq = (rng.next_u64() % 1000) + 1;

                // get_at_seq on shared_key must never panic or return invalid order
                if let Some((_, found_seq, _)) =
                    mt_clone.get_at_seq(b"shared_key", query_seq, u64::MAX)
                {
                    assert!(
                        (found_seq & !TOMBSTONE_BIT) <= query_seq,
                        "Reader observed sequence {found_seq} exceeding query_seq {query_seq} (Seed: {SEED:#X})"
                    );
                }

                // scan_prefix_into must succeed and produce ordered entries
                let mut scan_target = BTreeMap::new();
                mt_clone.scan_prefix_into(
                    b"writer_",
                    query_seq,
                    TxId(u64::MAX),
                    &mut scan_target,
                );
                for (_k, (_v, seq)) in scan_target {
                    assert!(
                        (seq & !TOMBSTONE_BIT) <= query_seq,
                        "Prefix scan observed sequence {seq} exceeding query_seq {query_seq} (Seed: {SEED:#X})"
                    );
                }
            }
        }));
    }

    for h in handles {
        h.join().expect("Thread panicked during concurrent stress test");
    }

    // Final verification: verify sorted order of all versions in shared_key
    let all_entries = mt.iter();
    let mut prev_seq_by_key: BTreeMap<Bytes, u64> = BTreeMap::new();
    for (k, _v, seq, _tx) in all_entries {
        let raw_seq = seq & !TOMBSTONE_BIT;
        if let Some(prev_raw) = prev_seq_by_key.get(&k) {
            assert!(
                *prev_raw <= raw_seq,
                "Version order in MemTable violated for key {:?}: prev_seq {prev_raw} > current_seq {raw_seq} (Seed: {SEED:#X})",
                k
            );
        }
        prev_seq_by_key.insert(k, raw_seq);
    }
}
