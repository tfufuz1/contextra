//! Sharding- und Nebenläufigkeitstests für die MemTable (MZ_C)
//!
//! Validiert Shard-Verteilung, Bereichsbeschränkung und MVCC-Snapshot-Isolation.

#![forbid(unsafe_code)]
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use bytes::Bytes;
use contextra_ids::TxId;
use contextra_store::memtable::{MemTable, SHARD_COUNT};
use std::collections::BTreeMap;
use std::ops::Bound;

#[test]
fn test_memtable_shard_bounds_and_coverage() {
    let mt = MemTable::new();
    assert_eq!(mt.size(), 0);
    assert!(mt.is_empty());

    // Fülle Schlüssel in unterschiedliche Shards
    for i in 0..100u32 {
        let key = Bytes::from(format!("key_{:04}", i));
        let val = Bytes::from(format!("val_{:04}", i));
        mt.put(key, val, i as u64, i as u64);
    }

    assert_eq!(mt.size() > 0, true);
    assert!(!mt.is_empty());

    let counts = mt.shard_entry_counts();
    assert_eq!(counts.len(), SHARD_COUNT);
    let total: usize = counts.iter().sum();
    assert_eq!(total, 100);

    // Iteration über alle Einträge muss sortiert sein
    let all = mt.iter();
    assert_eq!(all.len(), 100);
    for window in all.windows(2) {
        assert!(window[0].0 <= window[1].0);
    }
}

#[test]
fn test_memtable_mvcc_snapshot_isolation() {
    let mt = MemTable::new();
    let key = Bytes::from("concurrent_key");

    mt.put(key.clone(), Bytes::from("v1"), 10, 1);
    mt.put(key.clone(), Bytes::from("v2"), 20, 2);
    mt.put(key.clone(), Bytes::from("v3"), 30, 3);

    // Snapshot bei seq 15 sieht v1
    let snap1 = mt.get_at_seq(&key, 15, 10).unwrap();
    assert_eq!(snap1.0, Bytes::from("v1"));
    assert_eq!(snap1.1, 10);

    // Snapshot bei seq 25 sieht v2
    let snap2 = mt.get_at_seq(&key, 25, 10).unwrap();
    assert_eq!(snap2.0, Bytes::from("v2"));
    assert_eq!(snap2.1, 20);

    // Snapshot bei seq 5 sieht nichts
    let snap0 = mt.get_at_seq(&key, 5, 10);
    assert!(snap0.is_none());
}

#[test]
fn test_memtable_prefix_scan_and_range_scan() {
    let mt = MemTable::new();
    for i in 0..50u32 {
        let key = Bytes::from(format!("user:{:03}", i));
        let val = Bytes::from(format!("val:{:03}", i));
        mt.put(key, val, i as u64, i as u64);
    }

    let mut target = BTreeMap::new();
    mt.scan_prefix_into(b"user:", 100, TxId::new(100), &mut target);
    assert_eq!(target.len(), 50);

    let mut range_target = BTreeMap::new();
    let start = Bound::Included(&b"user:010"[..]);
    let end = Bound::Included(&b"user:020"[..]);
    mt.scan_range_into(start, end, 100, TxId::new(100), &mut range_target);
    assert_eq!(range_target.len(), 11);
}
