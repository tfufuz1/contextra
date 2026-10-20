// STAND: 2026-10-21
// ZWECK: Deterministic regression test for fuzz_memtable_concurrent crash artifact (Bug J-15-F02).
// ORAKEL: Invariant Zero-Panic — MemTable::put must not panic even on out-of-order sequence numbers, and version queries must return correct ordered entries.

use bytes::Bytes;
use contextra_store::memtable::MemTable;

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
    // Non-monotonic lower sequence number on same key must insert in sorted order without panicking
    mt.put(key.clone(), Bytes::from("v0"), 10, 1);

    // Assert get_at_seq at seq 5 sees no version
    assert!(mt.get_at_seq(&key, 5, u64::MAX).is_none());

    // Assert get_at_seq at seq 15 sees v0 at seq 10
    let (val_10, seq_10, tx_10) = mt
        .get_at_seq(&key, 15, u64::MAX)
        .expect("seq 10 version should be present");
    assert_eq!(val_10.as_ref(), b"v0");
    assert_eq!(seq_10, 10);
    assert_eq!(tx_10, 1);

    // Assert get_at_seq at seq 150 sees v1 at seq 100
    let (val_100, seq_100, tx_100) = mt
        .get_at_seq(&key, 150, u64::MAX)
        .expect("seq 100 version should be present");
    assert_eq!(val_100.as_ref(), b"v1");
    assert_eq!(seq_100, 100);
    assert_eq!(tx_100, 1);

    // Assert get() returns latest sequence version (v1 at seq 100)
    let (latest_val, latest_seq) = mt.get(&key).expect("latest version should exist");
    assert_eq!(latest_val.as_ref(), b"v1");
    assert_eq!(latest_seq, 100);
}
