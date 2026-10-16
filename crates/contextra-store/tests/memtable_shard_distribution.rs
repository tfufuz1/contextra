// FILE-CONTEXT
// STAND: 2026-09-28T00:00:00Z
// ZWECK: Integrationstests für MemTable Systempräfix-Hash-Sharding und Scans.
// INVARIANTEN: Systempräfix-Schlüssel verteilen sich gleichmäßig über Shards; Scans stimmen 100% mit BTreeMap überein.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use bytes::Bytes;
use contextra_core::{TxId, TOMBSTONE_BIT};
use contextra_store::memtable::{MemTable, RESERVED_PREFIXES};
use proptest::prelude::*;
use std::collections::BTreeMap;
use std::ops::Bound;

#[test]
fn test_system_keys_distribution_across_shards() {
    let mt = MemTable::new();
    let num_keys = 10_000;

    for i in 0..num_keys {
        let key = Bytes::from(format!("__col:{i}"));
        let val = Bytes::from(format!("val_{i}"));
        mt.put(key, val, i as u64 + 1, 1);
    }

    let shard_counts = mt.shard_entry_counts();
    let active_shards = shard_counts.iter().filter(|&&count| count > 0).count();

    assert!(
        active_shards >= 8,
        "Expected 10,000 __col:<n> keys to distribute across at least 8 shards, got {active_shards} active shards: {:?}",
        shard_counts
    );
    assert_eq!(
        shard_counts.iter().sum::<usize>(),
        num_keys,
        "Total keys across shards must match inserted count"
    );

    // Also verify for other system prefixes
    for &prefix in RESERVED_PREFIXES {
        let prefix_str = std::str::from_utf8(prefix).unwrap();
        let mt_sys = MemTable::new();
        for i in 0..1_000 {
            let key = Bytes::from(format!("{prefix_str}item_{i}"));
            let val = Bytes::from(format!("v_{i}"));
            mt_sys.put(key, val, i as u64 + 1, 1);
        }
        let counts = mt_sys.shard_entry_counts();
        let active = counts.iter().filter(|&&c| c > 0).count();
        assert!(
            active >= 8,
            "Prefix {prefix_str} should distribute across at least 8 shards, got {active}: {:?}",
            counts
        );
    }
}

#[test]
fn test_tombstones_and_multiversion_correctness() {
    let mt = MemTable::new();
    let key = Bytes::from("__col:my_doc");

    // Version 1: Value at seq 10, tx 1
    mt.put(key.clone(), Bytes::from("v1"), 10, 1);
    // Version 2: Tombstone at seq 20, tx 2
    mt.put(key.clone(), Bytes::new(), 20 | TOMBSTONE_BIT, 2);
    // Version 3: Value at seq 30, tx 3
    mt.put(key.clone(), Bytes::from("v3"), 30, 3);

    // Read at seq 15 -> should get v1
    let (val, seq, tx) = mt.get_at_seq(&key, 15, u64::MAX).unwrap();
    assert_eq!(val.as_ref(), b"v1");
    assert_eq!(seq, 10);
    assert_eq!(tx, 1);

    // Read at seq 25 -> should get tombstone
    let (val, seq, tx) = mt.get_at_seq(&key, 25, u64::MAX).unwrap();
    assert!(val.is_empty());
    assert_eq!(seq, 20 | TOMBSTONE_BIT);
    assert_eq!(tx, 2);

    // Read at seq 35 -> should get v3
    let (val, seq, tx) = mt.get_at_seq(&key, 35, u64::MAX).unwrap();
    assert_eq!(val.as_ref(), b"v3");
    assert_eq!(seq, 30);
    assert_eq!(tx, 3);

    // Rollback tx 3
    mt.rollback(3);

    // Read at seq 35 now -> should fall back to tombstone at seq 20
    let (val, seq, tx) = mt.get_at_seq(&key, 35, u64::MAX).unwrap();
    assert!(val.is_empty());
    assert_eq!(seq, 20 | TOMBSTONE_BIT);
    assert_eq!(tx, 2);
}

// Strategy for generating mixed system and normal keys
fn key_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // System keys
        (0..1000i32).prop_map(|i| format!("__col:{i}")),
        (0..1000i32).prop_map(|i| format!("__meta:{i}")),
        (0..1000i32).prop_map(|i| format!("__rel:{i}")),
        (0..1000i32).prop_map(|i| format!("__graph:{i}")),
        (0..1000i32).prop_map(|i| format!("__txt:{i}")),
        // Normal keys
        (0..1000i32).prop_map(|i| format!("user:{i}")),
        (0..1000i32).prop_map(|i| format!("doc:{i}")),
        (0..1000i32).prop_map(|i| format!("a:{i}")),
        (0..1000i32).prop_map(|i| format!("z:{i}")),
    ]
}

proptest! {
    #[test]
    fn test_prefix_and_range_scan_equivalence_with_btreemap(
        mut ops in prop::collection::vec(
            (key_strategy(), any::<String>(), 1u64..100u64, 1u64..10u64, any::<bool>()),
            1..100
        )
    ) {
        let mt = MemTable::new();
        // Reference BTreeMap storing for each key a Vec of (seq, val, tx)
        let mut reference: BTreeMap<Bytes, Vec<(u64, Bytes, u64)>> = BTreeMap::new();

        // Sort ops by raw_seq ascending so sequence numbers per key are monotonic
        ops.sort_by_key(|op| op.2);

        for (k_str, v_str, raw_seq, tx, is_del) in ops {
            let k = Bytes::from(k_str);
            let v = if is_del { Bytes::new() } else { Bytes::from(v_str) };
            let seq = if is_del { raw_seq | TOMBSTONE_BIT } else { raw_seq };

            mt.put(k.clone(), v.clone(), seq, tx);
            reference.entry(k).or_default().push((seq, v, tx));
        }

        let max_seq = 200u64;
        let max_tx = TxId(100);

        // Test prefix scans on various prefixes
        let test_prefixes = [
            "__col:",
            "__meta:",
            "__rel:",
            "__graph:",
            "user:",
            "doc:",
            "a:",
            "z:",
            "__",
            "",
        ];

        for &p_str in &test_prefixes {
            let prefix = p_str.as_bytes();
            let mut mt_target = BTreeMap::new();
            mt.scan_prefix_into(prefix, max_seq, max_tx, &mut mt_target);

            // Compute expected target from reference BTreeMap
            let mut expected_target: BTreeMap<Bytes, (Bytes, u64)> = BTreeMap::new();
            for (k, versions) in reference.range::<[u8], _>((Bound::Included(prefix), Bound::Unbounded)) {
                if !k.starts_with(prefix) {
                    break;
                }
                let mut best: Option<(&Bytes, u64)> = None;
                for (seq, val, tx) in versions {
                    let raw_seq = seq & !TOMBSTONE_BIT;
                    if raw_seq <= (max_seq & !TOMBSTONE_BIT) && (*tx <= max_tx.inner() || *tx >= TxId::INTERNAL_BASE) {
                        match best {
                            Some((_, best_seq)) => {
                                if raw_seq > (best_seq & !TOMBSTONE_BIT) {
                                    best = Some((val, *seq));
                                }
                            }
                            None => {
                                best = Some((val, *seq));
                            }
                        }
                    }
                }
                if let Some((val, seq)) = best {
                    expected_target.insert(k.clone(), (val.clone(), seq));
                }
            }

            prop_assert_eq!(
                mt_target,
                expected_target,
                "Prefix scan for {:?} mismatched between MemTable and reference BTreeMap",
                p_str
            );
        }

        // Test range scans
        let range_bounds = [
            (Bound::Included(b"__col:0".as_slice()), Bound::Included(b"__col:999".as_slice())),
            (Bound::Included(b"a:0".as_slice()), Bound::Included(b"z:999".as_slice())),
            (Bound::Included(b"user:0".as_slice()), Bound::Included(b"user:999".as_slice())),
            (Bound::Unbounded, Bound::Unbounded),
        ];

        for (start, end) in range_bounds {
            let mut mt_range_target = BTreeMap::new();
            mt.scan_range_into(start, end, max_seq, max_tx, &mut mt_range_target);

            let mut expected_range_target: BTreeMap<Bytes, (Bytes, u64)> = BTreeMap::new();
            for (k, versions) in reference.range::<[u8], _>((start, end)) {
                let mut best: Option<(&Bytes, u64)> = None;
                for (seq, val, tx) in versions {
                    let raw_seq = seq & !TOMBSTONE_BIT;
                    if raw_seq <= (max_seq & !TOMBSTONE_BIT) && (*tx <= max_tx.inner() || *tx >= TxId::INTERNAL_BASE) {
                        match best {
                            Some((_, best_seq)) => {
                                if raw_seq > (best_seq & !TOMBSTONE_BIT) {
                                    best = Some((val, *seq));
                                }
                            }
                            None => {
                                best = Some((val, *seq));
                            }
                        }
                    }
                }
                if let Some((val, seq)) = best {
                    expected_range_target.insert(k.clone(), (val.clone(), seq));
                }
            }

            prop_assert_eq!(
                mt_range_target,
                expected_range_target,
                "Range scan mismatched between MemTable and reference BTreeMap"
            );
        }

        // Test iter_latest sorting and completeness
        let latest_entries = mt.iter_latest();
        let latest_keys: Vec<&[u8]> = latest_entries.iter().map(|(k, _, _, _)| k.as_ref()).collect();

        // Check latest_keys are strictly sorted ascending
        for i in 1..latest_keys.len() {
            prop_assert!(
                latest_keys[i - 1] < latest_keys[i],
                "iter_latest keys must be strictly sorted ascending: {:?} >= {:?}",
                String::from_utf8_lossy(latest_keys[i - 1]),
                String::from_utf8_lossy(latest_keys[i])
            );
        }
    }
}
