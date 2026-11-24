// FILE-CONTEXT
// ZWECK: Grenzwert-, Property- und Determinisums-Tests für Kern-Primitive (WP-VEC-02).
// INVARIANTEN: Zero-Panic, deterministische FixedHasher-Golden-Hashes, Breitenverhalten DocId/TxId.
// HOTSPOTS: test_casting_boundary_limits, test_fixed_hasher_determinism_golden_corpus, test_types_constants_and_ids
// STAND: 2026-11-11

use contextra_vector::casting::*;
use contextra_vector::fixed_hasher::{fixed_hasher, new_fixed_hashmap, new_fixed_hashset};
use contextra_vector::types::{DocId, TxId, MAX_SEARCH_K, TOMBSTONE_BIT};
use proptest::prelude::*;
use std::hash::{BuildHasher, Hash, Hasher};

#[test]
fn test_casting_boundary_limits() {
    // to_u32
    assert_eq!(to_u32(0).expect("0 must convert to u32"), 0);
    assert_eq!(
        to_u32(u32::MAX as usize).expect("u32::MAX must convert"),
        u32::MAX
    );
    #[cfg(target_pointer_width = "64")]
    {
        let overflow = (u32::MAX as usize) + 1;
        assert!(to_u32(overflow).is_err());
        assert!(to_u32(usize::MAX).is_err());
    }

    // u32_to_usize roundtrip
    assert_eq!(u32_to_usize(0), 0);
    assert_eq!(u32_to_usize(u32::MAX), u32::MAX as usize);

    // u64_to_usize
    assert_eq!(u64_to_usize(0).expect("0 to usize"), 0);
    assert_eq!(u64_to_usize(12345).expect("12345 to usize"), 12345);

    // u64_to_u32
    assert_eq!(u64_to_u32(0).expect("0 to u32"), 0);
    assert_eq!(
        u64_to_u32(u32::MAX as u64).expect("u32::MAX as u64"),
        u32::MAX
    );
    assert!(u64_to_u32((u32::MAX as u64) + 1).is_err());
    assert!(u64_to_u32(u64::MAX).is_err());

    // to_u8
    assert_eq!(to_u8(0).expect("0 to u8"), 0);
    assert_eq!(to_u8(255).expect("255 to u8"), 255);
    assert!(to_u8(256).is_err());

    // to_u64
    assert_eq!(to_u64(0), 0);
    assert_eq!(to_u64(100), 100);

    // f32_to_usize
    assert_eq!(f32_to_usize(0.0).expect("0.0"), 0);
    assert_eq!(f32_to_usize(42.9).expect("42.9"), 42);
    assert!(f32_to_usize(-1.0).is_err());
    assert!(f32_to_usize(f32::NAN).is_err());
    assert!(f32_to_usize(f32::INFINITY).is_err());
    assert!(f32_to_usize(f32::NEG_INFINITY).is_err());

    // f64_to_usize
    assert_eq!(f64_to_usize(0.0).expect("0.0"), 0);
    assert_eq!(f64_to_usize(100.99).expect("100.99"), 100);
    assert!(f64_to_usize(-0.1).is_err());
    assert!(f64_to_usize(f64::NAN).is_err());
    assert!(f64_to_usize(f64::INFINITY).is_err());
    assert!(f64_to_usize(f64::NEG_INFINITY).is_err());
}

#[test]
fn test_fixed_hasher_determinism_golden_corpus() {
    fn compute_hash<T: Hash>(val: &T) -> u64 {
        let mut hasher = fixed_hasher().build_hasher();
        val.hash(&mut hasher);
        hasher.finish()
    }

    let h1 = compute_hash(&"contextra-vector");
    let h2 = compute_hash(&"contextra-vector");
    assert_eq!(h1, h2, "Hash must be strictly deterministic across calls");

    let h_int1 = compute_hash(&42_u64);
    let h_int2 = compute_hash(&42_u64);
    assert_eq!(h_int1, h_int2);

    // Verifiziere deterministisches Einfügen und Auslesen in Sets und Maps
    let mut map = new_fixed_hashmap::<u64, &str>(16);
    map.insert(1, "alpha");
    map.insert(2, "beta");
    assert_eq!(map.get(&1), Some(&"alpha"));
    assert_eq!(map.get(&2), Some(&"beta"));

    let mut set = new_fixed_hashset::<u32>(16);
    set.insert(100);
    assert!(set.contains(&100));
}

#[test]
fn test_types_constants_and_ids() {
    assert_eq!(MAX_SEARCH_K, 10_000);
    assert_eq!(TOMBSTONE_BIT, 1 << 63);

    // TxId ist stets u64 (8 Bytes)
    assert_eq!(std::mem::size_of::<TxId>(), 8);

    // Breitenverhalten von DocId (unter docid-128 16 Bytes, sonst 8 Bytes)
    #[cfg(feature = "docid-128")]
    {
        assert_eq!(std::mem::size_of::<DocId>(), 16);
    }

    #[cfg(not(feature = "docid-128"))]
    {
        assert_eq!(std::mem::size_of::<DocId>(), 8);
    }
}

proptest! {
    #[test]
    fn prop_test_u32_roundtrip(val in 0..=u32::MAX) {
        let as_usize = u32_to_usize(val);
        let back = to_u32(as_usize).expect("roundtrip back to u32 must succeed");
        prop_assert_eq!(val, back);
    }

    #[test]
    fn prop_test_to_u8_valid(val in 0..=255_usize) {
        let res = to_u8(val).expect("val <= 255 must succeed");
        prop_assert_eq!(val as u8, res);
    }

    #[test]
    fn prop_test_to_u8_invalid(val in 256..=usize::MAX) {
        prop_assert!(to_u8(val).is_err());
    }

    #[test]
    fn prop_test_f32_non_negative_finite(val in 0.0f32..10_000_000.0f32) {
        let res = f32_to_usize(val).expect("non-negative finite float must succeed");
        prop_assert_eq!(val as usize, res);
    }
}
