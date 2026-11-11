// FILE-CONTEXT
// ZWECK: Deterministischer Fixed-Hasher für Hot-Path Mengen/Maps mit reproduzierbarer Traversierung (P28/INV-5).
// INVARIANTEN: Konstante Seeds, deterministische Iteration, Schutzschicht gegen HashMap-Drift.
// HOTSPOTS: fixed_hasher.rs:25-30 (fixed_hasher), fixed_hasher.rs:40-50 (new_fixed_hash*)
// STAND: 2026-11-11

//! Deterministischer Fixed Hasher für Hot-Path sets/maps where ordering or reproducible traversal is critical.
//!
//! Verhindert unkontrollierten Hash-Drift und Random-State-Injektionen in der HNSW-Graph-Traversierung.

use ahash::RandomState;
use std::collections::{HashMap, HashSet};

// SECTION Konstanten & Initialisierung

// ANCHOR[contextra-vector:HASHER-SEEDS]
const FIXED_SEEDS: (u64, u64, u64, u64) = (
    0x2345_6789_abcd_ef01,
    0x3456_789a_bcde_f012,
    0x4567_89ab_cdef_0123,
    0x5678_9abc_def0_1234,
);

// ANCHOR[contextra-vector:HASHER-BUILDER]
/// Liefert einen deterministischen `RandomState` Build-Hasher mit fixierten Seeds.
pub fn fixed_hasher() -> RandomState {
    RandomState::with_seeds(FIXED_SEEDS.0, FIXED_SEEDS.1, FIXED_SEEDS.2, FIXED_SEEDS.3)
}
// !SECTION

// SECTION Deterministische Kollektionen

// ANCHOR[contextra-vector:HASHER-COLLECTIONS]
/// Deterministisches `HashSet` mit fixem Hasher.
pub type FixedHashSet<T> = HashSet<T, RandomState>;

/// Deterministische `HashMap` mit fixem Hasher.
pub type FixedHashMap<K, V> = HashMap<K, V, RandomState>;

/// Erzeugt ein neues deterministisches `FixedHashSet` mit vorgegebener Kapazität.
pub fn new_fixed_hashset<T>(capacity: usize) -> FixedHashSet<T> {
    HashSet::with_capacity_and_hasher(capacity, fixed_hasher())
}

/// Erzeugt eine neue deterministische `FixedHashMap` mit vorgegebener Kapazität.
pub fn new_fixed_hashmap<K, V>(capacity: usize) -> FixedHashMap<K, V> {
    HashMap::with_capacity_and_hasher(capacity, fixed_hasher())
}
// !SECTION
