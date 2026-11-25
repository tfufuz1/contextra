// STAND: 2026-10-03
// ZWECK: Deterministic regression test for fuzz_memtable_concurrent crash artifact (Bug J-15-F02).
// ORAKEL: Invariant Zero-Panic — MemTable::put must not panic even on out-of-order sequence numbers.

use bytes::Bytes;
use contextra_store::memtable::MemTable;

#[test]
#[ignore = "J-15-F02: debug_assert panic in MemTable::put when raw_seq < last_raw_seq"]
fn campaign_fuzzregress_memtable_concurrent_out_of_order_seq() {
    let mt = MemTable::new();
    // Fuzz artifact input: 25 bytes decoded
    let _raw_bytes: &[u8] = &[
        181, 45, 0, 0, 0, 0, 0, 0, 3, 0, 226, 1, 1, 126, 0, 0, 0, 0, 0, 0, 0, 3, 0, 226, 1,
    ];

    // Minimal reproduction of out-of-order sequence number put on same key:
    let key = Bytes::from("test_key");
    mt.put(key.clone(), Bytes::from("v1"), 100, 1);
    // Non-monotonic lower sequence number on same key causes debug_assert! panic in MemTable::put
    mt.put(key, Bytes::from("v0"), 10, 1);
}
