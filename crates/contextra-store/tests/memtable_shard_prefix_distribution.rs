// ZWECK: MemTable Shard Prefix Distribution and Monotonicity Verification (S-04 Goal D).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_store::memtable::MemTable;
use std::collections::HashMap;

#[test]
fn test_memtable_shard_for_monotonicity_and_prefix_distribution() {
    let prefixes = [
        "__col:", "__meta:", "__sys:", "t1:", "t2:", "usr_1:", "usr_2:", "doc:", "vec:", "graph:",
    ];

    let mut shard_distribution: HashMap<usize, Vec<&'static str>> = HashMap::new();

    for &prefix in &prefixes {
        let shard = MemTable::shard_for(prefix.as_bytes());
        shard_distribution.entry(shard).or_default().push(prefix);

        let shard_computed = MemTable::shard_for(prefix.as_bytes());
        assert_eq!(shard, shard_computed);
    }

    println!("MemTable Prefix Shard Distribution (16 Shards):");
    for shard in 0..16 {
        if let Some(list) = shard_distribution.get(&shard) {
            println!("  Shard {:2}: {:?}", shard, list);
        } else {
            println!("  Shard {:2}: [empty]", shard);
        }
    }

    // Verify monotonicity across ordered keys
    let ordered_keys = [
        b"__col:0001".as_slice(),
        b"__col:0002".as_slice(),
        b"__meta:0001".as_slice(),
        b"__sys:0001".as_slice(),
        b"doc:0001".as_slice(),
        b"graph:0001".as_slice(),
        b"t1:0001".as_slice(),
        b"t2:0001".as_slice(),
        b"usr_1:0001".as_slice(),
        b"usr_2:0001".as_slice(),
        b"vec:0001".as_slice(),
    ];

    let mut prev_shard = 0;
    for &k in &ordered_keys {
        let shard = MemTable::shard_for(k);
        assert!(
            shard >= prev_shard,
            "Monotonicity rule keyA <= keyB => shard_for(keyA) <= shard_for(keyB) failed: key {:?} shard {} < prev_shard {}",
            String::from_utf8_lossy(k),
            shard,
            prev_shard
        );
        prev_shard = shard;
    }
}
