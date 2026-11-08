//! Deterministic Fixed Hasher for Hot-Path sets/maps where ordering or reproducible traversal is critical.

use ahash::RandomState;
use std::collections::{HashMap, HashSet};

const FIXED_SEEDS: (u64, u64, u64, u64) = (
    0x2345_6789_abcd_ef01,
    0x3456_789a_bcde_f012,
    0x4567_89ab_cdef_0123,
    0x5678_9abc_def0_1234,
);

/// Returns a deterministic `RandomState` build hasher seeded with constant values.
pub fn fixed_hasher() -> RandomState {
    RandomState::with_seeds(FIXED_SEEDS.0, FIXED_SEEDS.1, FIXED_SEEDS.2, FIXED_SEEDS.3)
}

/// A deterministic `HashSet` using constant seeds.
pub type FixedHashSet<T> = HashSet<T, RandomState>;

/// A deterministic `HashMap` using constant seeds.
pub type FixedHashMap<K, V> = HashMap<K, V, RandomState>;

/// Creates a new deterministic `FixedHashSet` with the given capacity.
pub fn new_fixed_hashset<T>(capacity: usize) -> FixedHashSet<T> {
    HashSet::with_capacity_and_hasher(capacity, fixed_hasher())
}

/// Creates a new deterministic `FixedHashMap` with the given capacity.
pub fn new_fixed_hashmap<K, V>(capacity: usize) -> FixedHashMap<K, V> {
    HashMap::with_capacity_and_hasher(capacity, fixed_hasher())
}
