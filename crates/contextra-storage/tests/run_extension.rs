use contextra_storage::store::run::extension::{decode, encode, ExtensionBlock};

#[test]
fn extension_region_round_trips_unknown_payloads_without_interpreting_them() {
    let blocks = vec![ExtensionBlock::new(17, 3, vec![0, 1, 0xff]).unwrap(), ExtensionBlock::new(99, 1, vec![]).unwrap()];
    let bytes = encode(&blocks, 4096).unwrap();
    assert_eq!(decode(&bytes, 4096, 8).unwrap(), blocks);
}

#[test]
fn extension_region_rejects_bounds_and_corruption() {
    assert!(ExtensionBlock::new(0, 1, vec![]).is_err());
    assert!(ExtensionBlock::new(1, 0, vec![]).is_err());
    let mut bytes = encode(&[ExtensionBlock::new(1, 1, vec![1, 2, 3]).unwrap()], 4096).unwrap();
    *bytes.get_mut(20).unwrap() ^= 0x80;
    assert!(decode(&bytes, 4096, 8).is_err());
    assert!(decode(&bytes, 4096, 0).is_err());
}
