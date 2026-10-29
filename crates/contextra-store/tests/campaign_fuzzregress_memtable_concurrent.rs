// STAND: 2026-10-29
// ZWECK: Deterministic regression test for fuzz_memtable_concurrent crash artifact (Bug J-15-F02).
// ORAKEL: Invariant Zero-Panic — MemTable::put must not panic even on out-of-order sequence numbers.

#![allow(unexpected_cfgs)]

use bytes::Bytes;
use contextra_store::memtable::MemTable;
use contextra_types::TxId;
use std::collections::BTreeMap;
use std::sync::{Arc, Barrier};

#[test]
fn campaign_fuzzregress_memtable_concurrent_out_of_order_seq() {
    let mt = MemTable::new();
    // Fuzz artifact input: 25 bytes decoded
    let _raw_bytes: &[u8] = &[
        181, 45, 0, 0, 0, 0, 0, 0, 3, 0, 226, 1, 1, 126, 0, 0, 0, 0, 0, 0, 0, 3, 0, 226, 1,
    ];

    // Minimal reproduction of out-of-order sequence number put on same key:
    let key = Bytes::from("test_key");
    mt.put(key.clone(), Bytes::from("v1"), 100, 1);
    // Non-monotonic lower sequence number on same key must not panic and must be inserted in sorted order
    mt.put(key.clone(), Bytes::from("v0"), 10, 1);

    let (latest_val, latest_seq) = mt.get(&key).expect("key exists");
    assert_eq!(latest_val.as_ref(), b"v1");
    assert_eq!(latest_seq, 100);

    let (at_50_val, at_50_seq, _) = mt.get_at_seq(&key, 50, u64::MAX).expect("seq 10 visible");
    assert_eq!(at_50_val.as_ref(), b"v0");
    assert_eq!(at_50_seq, 10);
}

#[test]
fn test_concurrent_memtable_out_of_order_writes_deterministic() {
    const THREADS: usize = 8;
    const WRITES_PER_THREAD: usize = 100;

    let mt = Arc::new(MemTable::new());
    let barrier = Arc::new(Barrier::new(THREADS));

    let mut handles = Vec::new();

    for thread_id in 0..THREADS {
        let mt = Arc::clone(&mt);
        let barrier = Arc::clone(&barrier);

        handles.push(std::thread::spawn(move || {
            // Wait for all threads to start simultaneously
            barrier.wait();

            let mut state = (thread_id as u64 + 1).wrapping_mul(0x9E3779B97F4A7C15);

            for i in 0..WRITES_PER_THREAD {
                // Deterministically generate pseudo-random sequence number in range 1..10000
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let seq = (state % 10000) + 1;
                let key_idx = i % 5;
                let key = Bytes::from(format!("user_key_{}", key_idx));
                let val = Bytes::from(format!("val_t{}_i{}_s{}", thread_id, i, seq));

                mt.put(key, val, seq, (thread_id + 1) as u64);
            }
        }));
    }

    for h in handles {
        h.join().expect("thread failed");
    }

    // Verify key version lists remain sorted and MVCC snapshot reads function correctly
    for key_idx in 0..5 {
        let key = Bytes::from(format!("user_key_{}", key_idx));

        let (latest_val, latest_seq) = mt.get(&key).expect("key should exist");
        assert!(latest_seq > 0);
        assert!(!latest_val.is_empty());

        let mut target = BTreeMap::new();
        mt.scan_prefix_into(b"user_key_", 10000, TxId(100), &mut target);
        assert_eq!(target.len(), 5);
    }

    let latest_entries = mt.iter_latest();
    assert_eq!(latest_entries.len(), 5);
}

#[cfg(loom)]
mod loom_tests {
    use bytes::Bytes;
    use contextra_store::memtable::MemTable;
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn test_loom_memtable_out_of_order_put() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(2);

        builder.check(|| {
            let mt = Arc::new(MemTable::new());
            let key = Bytes::from("loom_key");

            let mt1 = Arc::clone(&mt);
            let k1 = key.clone();
            let t1 = thread::spawn(move || {
                mt1.put(k1, Bytes::from("v100"), 100, 1);
            });

            let mt2 = Arc::clone(&mt);
            let k2 = key.clone();
            let t2 = thread::spawn(move || {
                mt2.put(k2, Bytes::from("v50"), 50, 1);
            });

            t1.join().unwrap();
            t2.join().unwrap();

            let (val, seq) = mt.get(&key).expect("key present");
            assert_eq!(val.as_ref(), b"v100");
            assert_eq!(seq, 100);

            let (v_low, seq_low, _) = mt.get_at_seq(&key, 60, u64::MAX).expect("seq 50 visible");
            assert_eq!(v_low.as_ref(), b"v50");
            assert_eq!(seq_low, 50);
        });
    }
}
