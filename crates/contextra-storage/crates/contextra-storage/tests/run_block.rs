use contextra_storage::ports::{BranchId, Error};
use contextra_storage::store::run::block::{decode, encode, BlockEntry, MAX_DECODED_BLOCK_BYTES};
use zeroize::Zeroizing;

fn entry(key: &[u8], tag: u64) -> BlockEntry {
    BlockEntry { branch: BranchId(0), key: Zeroizing::new(key.to_vec()), tag,
        header: Zeroizing::new(vec![1, 2]), body: Zeroizing::new(vec![3, 4, 5]) }
}

#[test]
fn data_block_round_trip_crosses_prefix_restart_boundaries() -> Result<(), Error> {
    let entries: Vec<_> = (0..35).map(|index| {
        let key = format!("prefix/document/{index:03}");
        entry(key.as_bytes(), (index as u64 + 1) << 8 | 1)
    }).collect();
    let encoded = encode(&entries, MAX_DECODED_BLOCK_BYTES)?;
    assert_eq!(decode(&encoded, MAX_DECODED_BLOCK_BYTES)?, entries);
    Ok(())
}

#[test]
fn data_block_rejects_unsorted_duplicate_oversized_and_corrupt_input() {
    assert!(encode(&[entry(b"z", 0x101), entry(b"a", 0x101)], 1024).is_err());
    assert!(encode(&[entry(b"a", 0x101), entry(b"a", 0x101)], 1024).is_err());
    assert!(encode(&[entry(b"a", 0x101)], 8).is_err());
    let mut bytes = encode(&[entry(b"a", 0x101)], 1024).expect("encode");
    if let Some(last) = bytes.last_mut() { *last ^= 1; }
    assert!(decode(&bytes, 1024).is_err());
}

#[test]
fn data_block_rejects_noncanonical_or_reserved_tags() {
    assert!(encode(&[entry(b"a", 1)], 1024).is_err());
    assert!(encode(&[entry(b"a", 0x106)], 1024).is_err());
    assert!(encode(&[entry(b"a", 0x1ff)], 1024).is_err());
    assert!(encode(&[entry(b"a", 0x103)], 1024).is_err());
    assert!(encode(&[entry(b"a", 0x104)], 1024).is_err());
}
