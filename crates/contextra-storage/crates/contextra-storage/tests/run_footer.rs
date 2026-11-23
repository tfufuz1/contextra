use contextra_storage::store::run::format::{RunFooter, RUN_FOOTER_LEN};

#[test]
fn footer_round_trip_checks_ordered_bounded_offsets() {
    let file_len = 256;
    let footer = RunFooter { directory_offset: 80, directory_len: 40, pack_offset: 120, extension_offset: 180 };
    let bytes = footer.encode(file_len).unwrap();
    assert_eq!(bytes.len(), RUN_FOOTER_LEN);
    assert_eq!(RunFooter::decode(&bytes, file_len).unwrap(), footer);

    let mut corrupt = bytes;
    *corrupt.get_mut(0).unwrap() ^= 1;
    assert!(RunFooter::decode(&corrupt, file_len).is_err());
    assert!(RunFooter::decode(&bytes, 100).is_err());
}

#[test]
fn footer_rejects_overflow_and_overlapping_regions() {
    assert!(RunFooter { directory_offset: u64::MAX, directory_len: 10, pack_offset: 1, extension_offset: 2 }
        .encode(256).is_err());
    assert!(RunFooter { directory_offset: 80, directory_len: 50, pack_offset: 120, extension_offset: 180 }
        .encode(256).is_err());
}
