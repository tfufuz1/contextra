#![allow(clippy::unwrap_used)]
use super::*;

#[test]
fn test_put_get() {
    let mt = MemTable::new();
    mt.put(Bytes::from("key1"), Bytes::from("val1"), 1, 1);
    mt.put(Bytes::from("key2"), Bytes::from("val2"), 2, 2);

    let (val, seq) = mt.get(b"key1").expect("key1 should exist"); // #[cfg(test)]
    assert_eq!(val.as_ref(), b"val1");
    assert_eq!(seq, 1);

    let (val, seq) = mt.get(b"key2").expect("key2 should exist"); // #[cfg(test)]
    assert_eq!(val.as_ref(), b"val2");
    assert_eq!(seq, 2);

    assert!(mt.get(b"key3").is_none());
}

#[test]
fn test_mvcc_get_at_seq() {
    let mt = MemTable::new();
    mt.put(Bytes::from("key1"), Bytes::from("v1"), 10, 1);
    mt.put(Bytes::from("key1"), Bytes::from("v2"), 20, 2);
    mt.put(Bytes::from("key1"), Bytes::from("v3"), 30, 3);

    // Before any version
    assert!(mt.get_at_seq(b"key1", 5, u64::MAX).is_none());

    // Exact match
    let (val, seq, tx) = mt.get_at_seq(b"key1", 20, u64::MAX).unwrap(); // #[cfg(test)]
    assert_eq!(val.as_ref(), b"v2");
    assert_eq!(seq, 20);
    assert_eq!(tx, 2);

    // Between versions
    let (val, seq, tx) = mt.get_at_seq(b"key1", 25, u64::MAX).unwrap(); // #[cfg(test)]
    assert_eq!(val.as_ref(), b"v2");
    assert_eq!(seq, 20);
    assert_eq!(tx, 2);

    // Filtered by max_tx: seq 20 has tx=2, max_tx=1 should fallback to seq 10 tx 1
    let (val, seq, tx) = mt.get_at_seq(b"key1", 25, 1).unwrap(); // #[cfg(test)]
    assert_eq!(val.as_ref(), b"v1");
    assert_eq!(seq, 10);
    assert_eq!(tx, 1);

    // Latest version
    let (val, seq, tx) = mt.get_at_seq(b"key1", 100, u64::MAX).unwrap(); // #[cfg(test)]
    assert_eq!(val.as_ref(), b"v3");
    assert_eq!(seq, 30);
    assert_eq!(tx, 3);
}

#[test]
fn test_iter_all_versions() {
    let mt = MemTable::new();
    mt.put(Bytes::from("a"), Bytes::from("v1"), 1, 1);
    mt.put(Bytes::from("a"), Bytes::from("v2"), 2, 2);
    mt.put(Bytes::from("b"), Bytes::from("v3"), 3, 3);

    let entries = mt.iter();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].2, 1);
    assert_eq!(entries[1].2, 2);
    assert_eq!(entries[2].2, 3);
}

#[test]
fn test_mvcc_tombstone_binary_search() {
    use contextra_ids::TOMBSTONE_BIT;
    let mt = MemTable::new();
    let key = Bytes::from("key1");

    // Insert a value then a tombstone at a higher sequence number
    mt.put(key.clone(), Bytes::from("val1"), 10, 1);
    mt.put(key.clone(), Bytes::new(), 20 | TOMBSTONE_BIT, 2);

    // Read at seq 15 -> should get val1
    let (val, seq, tx) = mt.get_at_seq(&key, 15, u64::MAX).expect("Should find v1"); // #[cfg(test)]
    assert_eq!(val.as_ref(), b"val1");
    assert_eq!(seq, 10);
    assert_eq!(tx, 1);

    // Read at seq 25 -> should get tombstone
    let (val, seq, tx) = mt
        .get_at_seq(&key, 25, u64::MAX)
        .expect("Should find tombstone"); // #[cfg(test)]
    assert_eq!(val.len(), 0);
    assert_eq!(seq, 20 | TOMBSTONE_BIT);
    assert_eq!(tx, 2);
}

#[test]
fn test_iter_latest() {
    let mt = MemTable::new();
    mt.put(Bytes::from("a"), Bytes::from("v1"), 1, 1);
    mt.put(Bytes::from("a"), Bytes::from("v2"), 2, 2);
    mt.put(Bytes::from("b"), Bytes::from("v3"), 3, 3);

    let entries = mt.iter_latest();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].0.as_ref(), b"a");
    assert_eq!(entries[0].2, 2);
    assert_eq!(entries[1].0.as_ref(), b"b");
    assert_eq!(entries[1].2, 3);
}

#[test]
fn test_iter_ordering_across_shards() {
    let mt = MemTable::new();
    // Insert keys that map to various shards in reverse order
    let keys = ["z", "m", "a", "k", "b", "c", "x", "p"];
    for (i, &k) in keys.iter().enumerate() {
        mt.put(
            Bytes::from(k),
            Bytes::from(format!("val_{}", k)),
            (i + 1) as u64,
            (i + 1) as u64,
        );
    }

    let latest = mt.iter_latest();
    let fetched_keys: Vec<&[u8]> = latest.iter().map(|(k, _, _, _)| k.as_ref()).collect();
    assert_eq!(
        fetched_keys,
        vec![b"a", b"b", b"c", b"k", b"m", b"p", b"x", b"z"]
    );

    let all = mt.iter();
    let fetched_all_keys: Vec<&[u8]> = all.iter().map(|(k, _, _, _)| k.as_ref()).collect();
    assert_eq!(
        fetched_all_keys,
        vec![b"a", b"b", b"c", b"k", b"m", b"p", b"x", b"z"]
    );
}

#[test]
fn test_size_estimation_accuracy() {
    let mt = MemTable::new();
    assert_eq!(mt.size(), 0);

    // Put key (len 4), val (len 4), overhead STAGING_ENTRY_OVERHEAD_BYTES (32) => size = 40
    let key1 = Bytes::from("key1");
    let val1 = Bytes::from("val1");
    mt.put(key1, val1, 1, 1);
    assert_eq!(mt.size(), 40);

    // Put delete tombstone: key (len 4), val (len 0), overhead STAGING_ENTRY_OVERHEAD_BYTES (32) => size += 36
    let key1_del = Bytes::from("key1");
    mt.put(key1_del, Bytes::new(), 2 | TOMBSTONE_BIT, 1);
    assert_eq!(mt.size(), 76);
}

#[test]
fn test_transaction_rollback() {
    let mt = MemTable::new();

    // Begin tx A (tx_id = 100), put 3 keys
    mt.put(Bytes::from("keyA1"), Bytes::from("valA1"), 1, 100);
    mt.put(Bytes::from("keyA2"), Bytes::from("valA2"), 2, 100);
    mt.put(Bytes::from("keyA3"), Bytes::from("valA3"), 3, 100);

    // Begin tx B (tx_id = 200), put 2 keys
    mt.put(Bytes::from("keyB1"), Bytes::from("valB1"), 4, 200);
    mt.put(Bytes::from("keyB2"), Bytes::from("valB2"), 5, 200);

    assert_eq!(mt.iter_latest().len(), 5);

    // Rollback tx A
    mt.rollback(100);

    // Scan all keys — only B's keys should appear
    let latest = mt.iter_latest();
    assert_eq!(latest.len(), 2);
    assert_eq!(latest[0].0.as_ref(), b"keyB1");
    assert_eq!(latest[0].3, 200);
    assert_eq!(latest[1].0.as_ref(), b"keyB2");
    assert_eq!(latest[1].3, 200);

    assert!(mt.get(b"keyA1").is_none());
    assert!(mt.get(b"keyA2").is_none());
    assert!(mt.get(b"keyA3").is_none());
    assert!(mt.get(b"keyB1").is_some());
    assert!(mt.get(b"keyB2").is_some());

    // Verify size tracking updated after rollback
    // Originally: tx A = 3 * (5 + 5 + 32) = 126; tx B = 2 * (5 + 5 + 32) = 84. Total = 210.
    // After rollback tx A: remaining size should be 84.
    assert_eq!(mt.size(), 84);
}

#[test]
fn test_range_sharding_monotonicity() {
    let keys = [
        b"a".as_slice(),
        b"apple".as_slice(),
        b"b".as_slice(),
        b"banana".as_slice(),
        b"m".as_slice(),
        b"mango".as_slice(),
        b"z".as_slice(),
        b"zebra".as_slice(),
    ];
    let mut prev_shard = 0;
    for &k in &keys {
        let shard = MemTable::shard_for(k);
        assert!(
            shard >= prev_shard,
            "Range sharding monotonicity violated for key {:?}: shard {} < prev_shard {}",
            String::from_utf8_lossy(k),
            shard,
            prev_shard
        );
        prev_shard = shard;
    }
}

#[test]
fn test_scan_prefix_prunes_shards() {
    let range = MemTable::shard_range_for_prefix(b"user:");
    assert_eq!(
        range.clone().count(),
        1,
        "2-byte prefix 'user:' should target exactly 1 shard, got {:?}",
        range
    );
}

#[test]
fn test_concurrent_put_no_data_loss() {
    // Write 1000 keys from 8 threads, verify all are readable.
    use std::sync::Arc;
    let mt = Arc::new(MemTable::new());
    let handles: Vec<_> = (0..8u64)
        .map(|t| {
            let mt = Arc::clone(&mt);
            std::thread::spawn(move || {
                for i in 0..125u64 {
                    let key = Bytes::from(format!("key-{}-{}", t, i));
                    mt.put(key, Bytes::from("val"), t * 125 + i + 1, t);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread panicked"); // #[cfg(test)]
    }
    assert_eq!(mt.iter_latest().len(), 1000);
}

#[test]
fn test_empty_memtable_edge_cases() {
    let mt = MemTable::new();

    // 1. is_empty on new memtable
    assert!(mt.is_empty());
    assert_eq!(mt.size(), 0);

    // 2. tx_range on empty memtable -> (u64::MAX, 0)
    assert_eq!(mt.tx_range(), (u64::MAX, 0));

    // 3. Rollback non-existent transaction on empty or populated memtable
    mt.rollback(999);
    assert!(mt.is_empty());
    assert_eq!(mt.tx_range(), (u64::MAX, 0));

    // Insert and then rollback non-existent tx
    mt.put(Bytes::from("k"), Bytes::from("v"), 1, 10);
    assert!(!mt.is_empty());
    assert_eq!(mt.tx_range(), (10, 10));

    mt.rollback(999); // should do nothing
    assert!(!mt.is_empty());
    assert_eq!(mt.get(b"k").expect("should exist").0.as_ref(), b"v"); // #[cfg(test)]
    assert_eq!(mt.tx_range(), (10, 10));

    // Rollback existing tx
    mt.rollback(10);
    assert!(mt.is_empty());
    assert_eq!(mt.size(), 0);
    assert_eq!(mt.tx_range(), (u64::MAX, 0));
}

#[test]
fn test_saturating_sub_size_underflow_saturates_to_zero() {
    let mt = MemTable::new();
    mt.put(Bytes::from("key1"), Bytes::from("val1"), 1, 1);
    assert!(mt.size() > 0);

    // Directly invoke saturating_sub_size with an amount far larger than current size
    let current_size = mt.size();
    mt.saturating_sub_size(current_size + 1000);

    // Verify size saturates at 0 and does not wrap around to usize::MAX
    assert_eq!(mt.size(), 0);
}

#[test]
fn test_scan_prefix_into_and_scan_range_into() {
    let mt = MemTable::new();

    // Insert versions for keys across prefix and range
    mt.put(Bytes::from("prefix:a"), Bytes::from("val_a_v1"), 10, 1);
    mt.put(Bytes::from("prefix:a"), Bytes::from("val_a_v2"), 20, 2);
    mt.put(Bytes::from("prefix:b"), Bytes::from("val_b_v1"), 15, 1);
    mt.put(Bytes::from("other:c"), Bytes::from("val_c_v1"), 30, 3);

    // Test scan_prefix_into at max_seq=25, max_tx=TxId(2)
    let mut target = BTreeMap::new();
    mt.scan_prefix_into(b"prefix:", 25, TxId(2), &mut target);
    assert_eq!(target.len(), 2);
    assert_eq!(
        target.get(b"prefix:a".as_slice()),
        Some(&(Bytes::from("val_a_v2"), 20))
    );
    assert_eq!(
        target.get(b"prefix:b".as_slice()),
        Some(&(Bytes::from("val_b_v1"), 15))
    );

    // Test scan_prefix_into at max_seq=12, max_tx=TxId(1)
    let mut target2 = BTreeMap::new();
    mt.scan_prefix_into(b"prefix:", 12, TxId(1), &mut target2);
    assert_eq!(target2.len(), 1);
    assert_eq!(
        target2.get(b"prefix:a".as_slice()),
        Some(&(Bytes::from("val_a_v1"), 10))
    );

    // Test scan_range_into bounded range [prefix:a, prefix:z]
    let mut target3 = BTreeMap::new();
    mt.scan_range_into(
        Bound::Included(b"prefix:a".as_slice()),
        Bound::Included(b"prefix:z".as_slice()),
        100,
        TxId(10),
        &mut target3,
    );
    assert_eq!(target3.len(), 2);
    assert_eq!(
        target3.get(b"prefix:a".as_slice()),
        Some(&(Bytes::from("val_a_v2"), 20))
    );
    assert_eq!(
        target3.get(b"prefix:b".as_slice()),
        Some(&(Bytes::from("val_b_v1"), 15))
    );
}

#[test]
fn test_scan_prefix_into_bit_identity_with_iter() {
    let mt = MemTable::new();
    let prefix = b"user:";

    for i in 0..100u64 {
        let k = Bytes::from(format!("user:{:04}", i));
        let v = Bytes::from(format!("val_{}", i));
        mt.put(k.clone(), v.clone(), i + 1, (i % 5) + 1);
        // Put a second version for even keys
        if i % 2 == 0 {
            mt.put(
                k,
                Bytes::from(format!("val_v2_{}", i)),
                i + 101,
                (i % 5) + 1,
            );
        }
    }

    let max_seq = 150u64;
    let max_tx = TxId(4);

    // 1. Target via scan_prefix_into
    let mut fast_target = BTreeMap::new();
    mt.scan_prefix_into(prefix, max_seq, max_tx, &mut fast_target);

    // 2. Target via legacy mt.iter() filter logic
    let mut legacy_target: BTreeMap<Bytes, (Bytes, u64)> = BTreeMap::new();
    for (k, v, seq, tx) in mt.iter() {
        if k.starts_with(prefix) {
            let raw_seq = seq & !TOMBSTONE_BIT;
            if raw_seq <= (max_seq & !TOMBSTONE_BIT)
                && (tx <= max_tx.inner() || tx >= TxId::INTERNAL_BASE)
            {
                let entry = legacy_target.entry(k).or_insert_with(|| (v.clone(), seq));
                if (seq & !TOMBSTONE_BIT) > (entry.1 & !TOMBSTONE_BIT) {
                    *entry = (v, seq);
                }
            }
        }
    }

    assert_eq!(
        fast_target, legacy_target,
        "Fast range/prefix scan results must be bit-identical to legacy iter-based filtering"
    );
}

#[test]
fn test_concurrent_writer_and_range_scanner() {
    use std::sync::Arc;
    let mt = Arc::new(MemTable::new());

    // Pre-populate 50 keys
    for i in 0..50u64 {
        mt.put(
            Bytes::from(format!("doc:{:03}", i)),
            Bytes::from(format!("val_{}", i)),
            i + 1,
            1,
        );
    }

    let mt_writer = Arc::clone(&mt);
    let mt_scanner = Arc::clone(&mt);

    let writer_handle = std::thread::spawn(move || {
        for i in 50..150u64 {
            mt_writer.put(
                Bytes::from(format!("doc:{:03}", i)),
                Bytes::from(format!("val_{}", i)),
                i + 1,
                2,
            );
        }
    });

    let scanner_handle = std::thread::spawn(move || {
        for _ in 0..100 {
            let mut target = BTreeMap::new();
            mt_scanner.scan_prefix_into(b"doc:", 200, TxId(2), &mut target);
            assert!(target.len() >= 50 && target.len() <= 150);
        }
    });

    writer_handle.join().expect("writer finished"); // #[cfg(test)]
    scanner_handle.join().expect("scanner finished"); // #[cfg(test)]
}

#[test]
fn test_out_of_order_seq_put() {
    let mt = MemTable::new();
    let key = Bytes::from("key_ooo");

    // Insert seq 20 first, then seq 10 (out-of-order), then seq 30, then seq 15
    mt.put(key.clone(), Bytes::from("v20"), 20, 2);
    mt.put(key.clone(), Bytes::from("v10"), 10, 1);
    mt.put(key.clone(), Bytes::from("v30"), 30, 3);
    mt.put(key.clone(), Bytes::from("v15"), 15, 2);

    // get() should return the highest sequence version (v30 at seq 30)
    let (val, seq) = mt.get(&key).expect("key should exist");
    assert_eq!(val.as_ref(), b"v30");
    assert_eq!(seq, 30);

    // get_at_seq at seq 12 -> should find v10 at seq 10
    let (val, seq, tx) = mt.get_at_seq(&key, 12, u64::MAX).expect("seq 10 visible");
    assert_eq!(val.as_ref(), b"v10");
    assert_eq!(seq, 10);
    assert_eq!(tx, 1);

    // get_at_seq at seq 18 -> should find v15 at seq 15
    let (val, seq, tx) = mt.get_at_seq(&key, 18, u64::MAX).expect("seq 15 visible");
    assert_eq!(val.as_ref(), b"v15");
    assert_eq!(seq, 15);
    assert_eq!(tx, 2);

    // iter_latest should return key_ooo with seq 30
    let latest = mt.iter_latest();
    assert_eq!(latest.len(), 1);
    assert_eq!(latest[0].2, 30);
}
