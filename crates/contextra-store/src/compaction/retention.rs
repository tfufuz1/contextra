use contextra_core::TOMBSTONE_BIT;

/// Filters a sequence of entries for a SINGLE key (sorted by raw sequence DESCENDING)
/// according to MVCC retention rules given a `floor` sequence number.
///
/// Rules:
/// 1. Retain ALL versions with `raw_seq > floor_seq`.
/// 2. Retain the FIRST (newest) version with `raw_seq <= floor_seq`.
/// 3. Discard all older versions with `raw_seq <= floor_seq`.
///
/// Note: Sequence numbers are compared after masking `TOMBSTONE_BIT` (`seq & !TOMBSTONE_BIT`).
pub fn retain_key_versions<T, F>(entries: Vec<T>, get_seq: F, floor_seq: u64) -> Vec<T>
where
    F: Fn(&T) -> u64,
{
    let mut retained = Vec::with_capacity(entries.len());
    let mut found_floor_version = false;

    for entry in entries {
        let raw_seq = get_seq(&entry) & !TOMBSTONE_BIT;
        if raw_seq > floor_seq {
            retained.push(entry);
        } else if !found_floor_version {
            retained.push(entry);
            found_floor_version = true;
        }
    }

    retained
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retain_versions_with_pinned_floor() {
        // Versions: 30, 20, 10, 5 for floor = 10
        let versions = vec![
            (b"k1".to_vec(), 30u64),
            (b"k1".to_vec(), 20u64),
            (b"k1".to_vec(), 10u64),
            (b"k1".to_vec(), 5u64),
        ];

        let retained = retain_key_versions(versions, |(_, seq)| *seq, 10);
        let seqs: Vec<u64> = retained.into_iter().map(|(_, seq)| seq).collect();
        assert_eq!(seqs, vec![30, 20, 10]);
    }

    #[test]
    fn test_retain_versions_unpinned_floor_max() {
        // Versions: 30, 20, 10, 5 for floor = u64::MAX
        let versions = vec![
            (b"k1".to_vec(), 30u64),
            (b"k1".to_vec(), 20u64),
            (b"k1".to_vec(), 10u64),
            (b"k1".to_vec(), 5u64),
        ];

        let retained = retain_key_versions(versions, |(_, seq)| *seq, u64::MAX);
        let seqs: Vec<u64> = retained.into_iter().map(|(_, seq)| seq).collect();
        assert_eq!(seqs, vec![30, 20, 10, 5]);
    }

    #[test]
    fn test_tombstone_bit_masking() {
        let versions = vec![
            (b"k1".to_vec(), 20u64 | TOMBSTONE_BIT),
            (b"k1".to_vec(), 10u64),
        ];

        let retained = retain_key_versions(versions, |(_, seq)| *seq, 15);
        let seqs: Vec<u64> = retained
            .into_iter()
            .map(|(_, seq)| seq & !TOMBSTONE_BIT)
            .collect();
        assert_eq!(seqs, vec![20, 10]);
    }
}
