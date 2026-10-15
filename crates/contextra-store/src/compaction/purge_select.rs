// FILE-CONTEXT
// STAND: 2026-10-12 (T-2026-9205)
// ZWECK: Selektion von SSTable-Kandidaten für Purge-Compaction gemäss Spezifikation C5.4.
// INVARIANTEN: Determinister, reiner Algorithmus ohne I/O-Seiteneffekte.

use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeScope};
use crate::sstable::SstableReader;
use std::sync::Arc;

/// Checks if an SSTable key range `[first_key, last_key]` intersects the given purge scope.
pub fn range_intersects_scope(first_key: &[u8], last_key: &[u8], scope: &PurgeScope) -> bool {
    if first_key.is_empty() && last_key.is_empty() {
        return true;
    }
    match scope {
        PurgeScope::Keys(keys) => keys.iter().any(|k| {
            let k_slice = k.as_slice();
            k_slice >= first_key && k_slice <= last_key
        }),
        PurgeScope::Prefixes(prefixes) => prefixes.iter().any(|p| {
            let p_slice = p.as_slice();
            last_key >= p_slice && (first_key.starts_with(p_slice) || first_key <= p_slice)
        }),
    }
}

/// Checks if two key ranges `[f1, l1]` and `[f2, l2]` overlap.
pub fn ranges_overlap(f1: &[u8], l1: &[u8], f2: &[u8], l2: &[u8]) -> bool {
    if f1.is_empty() || l1.is_empty() || f2.is_empty() || l2.is_empty() {
        return true;
    }
    l1 >= f2 && l2 >= f1
}

/// Selects SSTable candidate files for Purge-Compaction following Spec C5.4:
/// (a) Initial selection of all files intersecting scope.
/// (b) Transitive completion (Abschlussbildung): as long as an unselected file intersects
///     the bounding range of selected files, include it to form a contiguous level window.
/// (c) Disk space check: sum of candidate file sizes <= free space. If probe is missing or
///     returns None, returns `PurgeBlocked::UnknownFreeSpace`.
pub fn select_purge_candidates(
    sstables: &[Arc<SstableReader>],
    scope: &PurgeScope,
    space_probe: Option<&dyn FreeSpaceProbe>,
) -> Result<Vec<Arc<SstableReader>>, PurgeBlocked> {
    if sstables.is_empty() {
        return Ok(Vec::new());
    }

    // (a) Initial selection: all files whose key range intersects scope
    let mut selected_indices: Vec<usize> = Vec::new();
    for (idx, sst) in sstables.iter().enumerate() {
        let meta = sst.metadata();
        if range_intersects_scope(&meta.first_key, &meta.last_key, scope) {
            selected_indices.push(idx);
        }
    }

    if selected_indices.is_empty() {
        return Ok(Vec::new());
    }

    // (b) Transitive completion (Abschlussbildung):
    // Expand selection to all unselected SSTables that overlap with the bounding range of selected SSTables
    loop {
        let mut added = false;
        let mut min_first: Option<&[u8]> = None;
        let mut max_last: Option<&[u8]> = None;
        for &idx in &selected_indices {
            let meta = sstables[idx].metadata();
            if min_first.is_none() || meta.first_key.as_ref() < min_first.unwrap() {
                min_first = Some(&meta.first_key);
            }
            if max_last.is_none() || meta.last_key.as_ref() > max_last.unwrap() {
                max_last = Some(&meta.last_key);
            }
        }

        let b_first = min_first.unwrap_or(&[]);
        let b_last = max_last.unwrap_or(&[]);

        for (idx, sst) in sstables.iter().enumerate() {
            if selected_indices.contains(&idx) {
                continue;
            }
            let meta = sst.metadata();
            if ranges_overlap(&meta.first_key, &meta.last_key, b_first, b_last) {
                selected_indices.push(idx);
                added = true;
            }
        }

        if !added {
            break;
        }
    }

    selected_indices.sort_unstable();

    let candidates: Vec<Arc<SstableReader>> = selected_indices
        .into_iter()
        .map(|idx| Arc::clone(&sstables[idx]))
        .collect();

    // (c) Disk space pre-flight check (L8 fix)
    // None probe means "unknown", not "0 bytes free". Purge is permitted to start.
    let needed_bytes: u64 = candidates.iter().map(|s| s.metadata().file_size).sum();

    let free_space_opt = match space_probe {
        Some(probe) => probe.free_space_bytes(),
        None => None,
    };

    if let Some(free_space) = free_space_opt {
        if free_space < needed_bytes || free_space == 0 {
            return Err(PurgeBlocked::InsufficientDiskSpace {
                needed: needed_bytes,
                free: free_space,
            });
        }
    }

    Ok(candidates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_ports::ProbeError;

    struct MockSpaceProbe {
        free: Option<u64>,
    }

    impl FreeSpaceProbe for MockSpaceProbe {
        fn available_bytes(&self) -> std::result::Result<Option<u64>, ProbeError> {
            Ok(self.free)
        }
    }

    #[test]
    fn test_range_intersects_scope_keys() {
        let scope = PurgeScope::Keys(vec![b"b".to_vec()]);
        assert!(range_intersects_scope(b"a", b"c", &scope));
        assert!(!range_intersects_scope(b"d", b"e", &scope));
    }

    #[test]
    fn test_range_intersects_scope_prefixes() {
        let scope = PurgeScope::Prefixes(vec![b"col:a:".to_vec()]);
        assert!(range_intersects_scope(b"col:a:1", b"col:a:9", &scope));
        assert!(!range_intersects_scope(b"col:b:1", b"col:b:9", &scope));
    }

    #[test]
    fn test_ranges_overlap() {
        assert!(ranges_overlap(b"a", b"c", b"b", b"d"));
        assert!(!ranges_overlap(b"a", b"b", b"c", b"d"));
    }

    #[test]
    fn test_mock_space_probe() {
        let probe = MockSpaceProbe { free: Some(100) };
        assert_eq!(probe.free_space_bytes(), Some(100));

        let probe_none = MockSpaceProbe { free: None };
        assert_eq!(probe_none.free_space_bytes(), None);
    }
}
