// FILE-CONTEXT
// ZWECK: AK-21 Collision test for length-prefixed key hash function hash_deleted_keys_length_prefixed.
// INVARIANTEN: Distinct key chunk partitions of identical byte sequences MUST produce distinct length-prefixed hashes.
// STAND: TS:2026-09-17T00:00:00Z

#![allow(dead_code)]

#[path = "../src/deletion_proof.rs"]
mod deletion_proof;

use deletion_proof::hash_deleted_keys_length_prefixed;
use proptest::prelude::*;

fn split_into_chunks(bytes: &[u8], split_indices: &[usize]) -> Vec<Vec<u8>> {
    let mut chunks = Vec::new();
    let mut sorted_splits: Vec<usize> = split_indices
        .iter()
        .copied()
        .filter(|&idx| idx > 0 && idx < bytes.len())
        .collect();
    sorted_splits.sort_unstable();
    sorted_splits.dedup();

    let mut start = 0;
    for split in sorted_splits {
        chunks.push(bytes[start..split].to_vec());
        start = split;
    }
    chunks.push(bytes[start..].to_vec());
    chunks
}

proptest! {
    #[test]
    fn no_prefix_collision(
        bytes in prop::collection::vec(any::<u8>(), 1..256),
        splits_a in prop::collection::vec(1..256usize, 0..10),
        splits_b in prop::collection::vec(1..256usize, 0..10),
    ) {
        let partition_a = split_into_chunks(&bytes, &splits_a);
        let partition_b = split_into_chunks(&bytes, &splits_b);

        // Ensure partitions are actually different (different chunk boundaries)
        prop_assume!(partition_a != partition_b);

        let hash_a = hash_deleted_keys_length_prefixed(&partition_a);
        let hash_b = hash_deleted_keys_length_prefixed(&partition_b);

        prop_assert_ne!(
            hash_a,
            hash_b,
            "Prefix collision detected for distinct partitions {:?} and {:?}",
            partition_a,
            partition_b
        );
    }
}

#[test]
fn test_explicit_prefix_collision_ab_c_vs_a_bc() {
    let partition_a = vec![b"ab".to_vec(), b"c".to_vec()];
    let partition_b = vec![b"a".to_vec(), b"bc".to_vec()];

    let hash_a = hash_deleted_keys_length_prefixed(&partition_a);
    let hash_b = hash_deleted_keys_length_prefixed(&partition_b);

    assert_ne!(
        hash_a, hash_b,
        "Length-prefixed hashing MUST produce different hashes for [\"ab\", \"c\"] vs [\"a\", \"bc\"]"
    );
}
