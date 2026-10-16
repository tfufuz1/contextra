//! Integration / unit tests for LSM scan limits, source distribution, and range bounds safety.

use bytes::Bytes;
use contextra_core::TxId;
use contextra_store::memtable::{is_valid_range_bounds, MemTable};
use std::collections::BTreeMap;
use std::ops::Bound;

#[test]
fn test_range_bounds_validation_helper() {
    let start = Bound::Included(b"b".as_slice());
    let end = Bound::Included(b"a".as_slice());

    // Inverted bounds (start > end) -> invalid
    assert!(!is_valid_range_bounds(start, end));

    // Equal exclusive bounds -> invalid
    let eq_excl_start = Bound::Excluded(b"a".as_slice());
    let eq_excl_end = Bound::Excluded(b"a".as_slice());
    assert!(!is_valid_range_bounds(eq_excl_start, eq_excl_end));

    // Valid bounds
    let valid_start = Bound::Included(b"a".as_slice());
    let valid_end = Bound::Included(b"b".as_slice());
    assert!(is_valid_range_bounds(valid_start, valid_end));
}

#[test]
fn test_memtable_scan_range_inverted_bounds_does_not_panic() {
    let mt = MemTable::new();
    mt.put(Bytes::from("a"), Bytes::from("val_a"), 1, 1);
    mt.put(Bytes::from("m"), Bytes::from("val_m"), 2, 1);
    mt.put(Bytes::from("z"), Bytes::from("val_z"), 3, 1);

    let mut target = BTreeMap::new();
    // Inverted range scan: start "z", end "a"
    mt.scan_range_into(
        Bound::Included(b"z".as_slice()),
        Bound::Included(b"a".as_slice()),
        100,
        TxId(10),
        &mut target,
    );

    assert!(
        target.is_empty(),
        "Inverted range scan must return empty target without panic"
    );
}

#[test]
fn test_memtable_scan_range_equal_excluded_bounds_does_not_panic() {
    let mt = MemTable::new();
    mt.put(Bytes::from("a"), Bytes::from("val_a"), 1, 1);

    let mut target = BTreeMap::new();
    // Excluded range with equal start and end
    mt.scan_range_into(
        Bound::Excluded(b"a".as_slice()),
        Bound::Excluded(b"a".as_slice()),
        100,
        TxId(10),
        &mut target,
    );

    assert!(
        target.is_empty(),
        "Equal excluded range scan must return empty target without panic"
    );
}
