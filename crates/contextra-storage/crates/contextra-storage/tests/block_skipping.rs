use contextra_storage::ports::Error;
use contextra_storage::store::run::skip_scan::{block_skip_scan, BlockSkipQuery};
use contextra_storage::store::run::subindex::{SubIndex, SubIndexEntry};
use zeroize::Zeroizing;

fn summary(last: &[u8], offset: u64, min: u64, max: u64, uid: u8) -> SubIndexEntry {
    SubIndexEntry { last_key: Zeroizing::new(last.to_vec()), offset, len: 100, entries: 10,
        weight_sum: [10, 20, 30, 40], min_stamp: min, max_stamp: max, block_uid: [uid; 16] }
}

#[test]
fn block_skipping_uses_only_subindex_ranges() -> Result<(), Error> {
    let index = SubIndex::new(vec![
        summary(b"a", 0, 1, 3, 1),
        summary(b"d", 100, 2, 4, 2),
        summary(b"g", 200, 5, 8, 3),
        summary(b"j", 300, 9, 12, 4),
    ])?;
    let selected: Vec<_> = block_skip_scan(&index, BlockSkipQuery {
        start_inclusive: Some(b"c"), end_exclusive: Some(b"h"), min_stamp: 5, max_stamp: 10,
    }).map(|entry| entry.offset).collect();
    assert_eq!(selected, vec![200, 300]);
    assert!(block_skip_scan(&index, BlockSkipQuery {
        start_inclusive: Some(b"x"), end_exclusive: Some(b"z"), min_stamp: 0, max_stamp: 20,
    }).next().is_none());
    Ok(())
}

#[test]
fn subindex_round_trip_and_rejects_overlapping_blocks() -> Result<(), Error> {
    let index = SubIndex::new(vec![summary(b"a", 0, 1, 3, 1), summary(b"d", 100, 2, 4, 2)])?;
    let bytes = index.encode(4096)?;
    assert_eq!(SubIndex::decode(&bytes, 4096, 100)?, index);
    assert!(SubIndex::new(vec![summary(b"a", 0, 1, 3, 1), summary(b"d", 50, 2, 4, 2)]).is_err());
    Ok(())
}
