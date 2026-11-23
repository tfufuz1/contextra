use contextra_storage::codec::tags::{tag, Kind};
use contextra_storage::store::run::rangedel::{decode, encode, RangeMarker};
use contextra_storage_ports::BranchId;
use zeroize::Zeroizing;

fn marker(branch: u32, start: &[u8], end: &[u8], stamp: u64) -> RangeMarker {
    RangeMarker { branch: BranchId(branch), start: Zeroizing::new(start.to_vec()), end: Zeroizing::new(end.to_vec()), tag: tag(stamp, Kind::RangeDelete).unwrap() }
}

#[test]
fn range_delete_block_round_trips_sorted_branch_local_intervals() {
    let source = vec![marker(0, b"", b"m", 2), marker(1, b"a", b"z", 3)];
    let encoded = encode(&source, 4096).unwrap();
    assert_eq!(decode(&encoded, 4096, 10).unwrap(), source);
}

#[test]
fn range_delete_block_rejects_bad_bounds_kind_order_and_corruption() {
    assert!(encode(&[marker(0, b"z", b"a", 1)], 4096).is_err());
    let mut wrong_kind = marker(0, b"a", b"z", 1);
    wrong_kind.tag = tag(1, Kind::Delete).unwrap();
    assert!(encode(&[wrong_kind], 4096).is_err());
    assert!(encode(&[marker(1, b"a", b"z", 1), marker(0, b"b", b"y", 2)], 4096).is_err());
    let mut encoded = encode(&[marker(0, b"a", b"z", 1)], 4096).unwrap();
    *encoded.get_mut(12).unwrap() ^= 1;
    assert!(decode(&encoded, 4096, 10).is_err());
}
