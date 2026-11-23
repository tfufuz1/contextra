//! SubIndex-only block selection. It never reads a DataBlock or Blob body.

use super::subindex::{SubIndex, SubIndexEntry};
use contextra_storage_ports::Stamp;

pub struct BlockSkipQuery<'a> {
    pub start_inclusive: Option<&'a [u8]>,
    pub end_exclusive: Option<&'a [u8]>,
    pub min_stamp: Stamp,
    pub max_stamp: Stamp,
}

/// Yields only summaries whose key interval and stamp range intersect the
/// query. A block represents `(previous.last_key, current.last_key]`.
pub fn block_skip_scan<'a>(
    subindex: &'a SubIndex,
    query: BlockSkipQuery<'a>,
) -> impl Iterator<Item = &'a SubIndexEntry> + 'a {
    let valid_stamps = query.min_stamp <= query.max_stamp;
    let valid_keys = query.start_inclusive.zip(query.end_exclusive)
        .map_or(true, |(start, end)| start < end);
    subindex.entries().iter().enumerate().filter_map(move |(index, entry)| {
        let previous_last = index.checked_sub(1).and_then(|previous| subindex.entries().get(previous))
            .map(|previous| previous.last_key.as_slice());
        let after_start = query.start_inclusive.map_or(true, |start| entry.last_key.as_slice() >= start);
        let before_end = query.end_exclusive.map_or(true, |end| previous_last.map_or(true, |last| last < end));
        let stamps_overlap = entry.min_stamp <= query.max_stamp && entry.max_stamp >= query.min_stamp;
        (valid_stamps && valid_keys && after_start && before_end && stamps_overlap).then_some(entry)
    })
}
