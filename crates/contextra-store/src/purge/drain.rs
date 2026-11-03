//! Memtable drain implementation (SPEC §8.8, §11.4).

use super::backend::DrainStats;
use crate::memtable::MemTable;
use contextra_types::key::{KeyBuf, Matcher};

/// Discards all entries in `memtable` matching `matcher`.
pub fn drain_memtable(memtable: &MemTable, matcher: &Matcher) -> DrainStats {
    let mut entries_drained = 0;
    let initial_size = memtable.size();

    // Drain matching keys from memory table
    memtable.retain(|raw_key| {
        if raw_key.len() == 78 {
            let mut arr = [0u8; 78];
            arr.copy_from_slice(raw_key);
            let key_buf = KeyBuf::from_raw(arr);
            if matcher.matches(&key_buf) {
                entries_drained += 1;
                return false;
            }
        }
        true
    });

    let bytes_drained = (initial_size.saturating_sub(memtable.size())) as u64;

    DrainStats {
        entries_drained,
        bytes_drained,
    }
}
