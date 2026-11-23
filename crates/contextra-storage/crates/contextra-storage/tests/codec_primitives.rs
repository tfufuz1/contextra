use contextra_storage::codec::checksums::crc32c;
use contextra_storage::codec::comparator::{cmp_entry, EntryKey};
use contextra_storage::codec::tags::{tag, untag, Kind, TagError};
use contextra_storage::codec::varint::{varint_get, varint_put, VarintError};
use contextra_storage::ports::{BranchId, ClusterId, STAMP_MAX};
use std::cmp::Ordering;

#[test]
fn canonical_varints_round_trip_boundaries_and_reject_noncanonical_forms() {
    // INV-FMT-1
    for value in [0, 1, 127, 128, 16_384, u32::MAX as u64, u64::MAX] {
        let mut encoded = Vec::new();
        varint_put(&mut encoded, value);
        assert_eq!(varint_get(&encoded), Ok((value, encoded.len())));
    }
    assert_eq!(varint_get(&[0x80, 0]), Err(VarintError::NonCanonical));
    assert_eq!(varint_get(&[0x80]), Err(VarintError::Truncated));
    assert_eq!(varint_get(&[0xff; 10]), Err(VarintError::Overflow));
}

#[test]
fn tags_reserve_zero_stamps_and_merge_kind() {
    // INV-FMT-1
    assert_eq!(tag(0, Kind::Put), Err(TagError::StampZero));
    assert_eq!(untag(Kind::Put as u64), Err(TagError::StampZero));
    let last = tag(STAMP_MAX, Kind::Put).expect("valid maximum stamp");
    assert_eq!(untag(last), Ok((STAMP_MAX, Kind::Put)));
    assert_eq!(untag((1 << 8) | 6), Err(TagError::ReservedMerge));
    assert_eq!(untag((1 << 8) | 0xff), Err(TagError::UnknownKind(0xff)));
}

#[test]
fn crc32c_matches_the_castagnoli_check_vector() {
    // INV-FMT-1
    assert_eq!(crc32c(b"123456789"), 0xe306_9283);
}

#[test]
fn entry_comparator_uses_branch_key_then_descending_tag() {
    let cluster = ClusterId::from_wire([1; 16]);
    let base = EntryKey { cluster, branch: BranchId(0), user_key: b"k", tag: (2 << 8) | 1 };
    let branch = EntryKey { cluster, branch: BranchId(1), user_key: b"a", tag: (3 << 8) | 1 };
    let newer = EntryKey { cluster, branch: BranchId(0), user_key: b"k", tag: (3 << 8) | 1 };
    assert_eq!(cmp_entry(&base, &branch), Ordering::Less);
    assert_eq!(cmp_entry(&newer, &base), Ordering::Less);
}
