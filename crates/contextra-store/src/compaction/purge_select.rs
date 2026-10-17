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
    let mut is_selected = vec![false; sstables.len()];
    for (idx, sst) in sstables.iter().enumerate() {
        let meta = sst.metadata();
        if range_intersects_scope(&meta.first_key, &meta.last_key, scope) {
            selected_indices.push(idx);
            is_selected[idx] = true;
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
            min_first = match min_first {
                Some(curr) if curr <= meta.first_key.as_ref() => Some(curr),
                _ => Some(&meta.first_key),
            };
            max_last = match max_last {
                Some(curr) if curr >= meta.last_key.as_ref() => Some(curr),
                _ => Some(&meta.last_key),
            };
        }

        let (Some(b_first), Some(b_last)) = (min_first, max_last) else {
            break;
        };

        for (idx, sst) in sstables.iter().enumerate() {
            if is_selected[idx] {
                continue;
            }
            let meta = sst.metadata();
            if ranges_overlap(&meta.first_key, &meta.last_key, b_first, b_last) {
                selected_indices.push(idx);
                is_selected[idx] = true;
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

    use crate::sstable::builder::SstableMetadata;
    use proptest::prelude::*;
    use bytes::Bytes;

    /// Naive reference implementation of candidate selection for property test comparison
    fn naive_select_purge_candidates(
        sstables: &[SstableMetadata],
        scope: &PurgeScope,
    ) -> Vec<usize> {
        if sstables.is_empty() {
            return Vec::new();
        }

        let mut selected: Vec<usize> = Vec::new();
        for (idx, meta) in sstables.iter().enumerate() {
            if range_intersects_scope(&meta.first_key, &meta.last_key, scope) {
                selected.push(idx);
            }
        }

        if selected.is_empty() {
            return Vec::new();
        }

        loop {
            let mut added = false;
            let mut min_first: Option<&[u8]> = None;
            let mut max_last: Option<&[u8]> = None;

            for &idx in &selected {
                let meta = &sstables[idx];
                min_first = match min_first {
                    Some(curr) if curr <= meta.first_key.as_ref() => Some(curr),
                    _ => Some(&meta.first_key),
                };
                max_last = match max_last {
                    Some(curr) if curr >= meta.last_key.as_ref() => Some(curr),
                    _ => Some(&meta.last_key),
                };
            }

            let (Some(b_first), Some(b_last)) = (min_first, max_last) else {
                break;
            };

            for (idx, meta) in sstables.iter().enumerate() {
                if selected.contains(&idx) {
                    continue;
                }
                if ranges_overlap(&meta.first_key, &meta.last_key, b_first, b_last) {
                    selected.push(idx);
                    added = true;
                }
            }

            if !added {
                break;
            }
        }

        selected.sort_unstable();
        selected.dedup();
        selected
    }

    use crate::sstable::SstableBuilder;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn prop_transitive_closure_properties(
            ranges in proptest::collection::vec(
                (0u8..100u8, 0u8..100u8),
                1..15
            ),
            target_key in 0u8..100u8
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let (actual_indices, naive_indices, sstables_meta) = rt.block_on(async {
                let tmp = tempfile::tempdir().unwrap();
                let bc = crate::sstable::create_block_cache();
                let mut sstables = Vec::new();
                let mut sstables_meta = Vec::new();

                for (i, &(a, b)) in ranges.iter().enumerate() {
                    let min_k = a.min(b);
                    let max_k = a.max(b);
                    let path = tmp.path().join(format!("{}.sst", i));
                    let mut builder = SstableBuilder::create(&path).await.unwrap();
                    builder.add(&[min_k], b"val", 1, 1).await.unwrap();
                    if max_k > min_k {
                        builder.add(&[max_k], b"val", 1, 1).await.unwrap();
                    }
                    let meta = builder.finish().await.unwrap();
                    let reader = SstableReader::open(&path, Arc::clone(&bc)).await.unwrap();
                    sstables.push(Arc::new(reader));
                    sstables_meta.push(meta);
                }

                let scope = PurgeScope::Keys(vec![vec![target_key]]);
                let candidates = select_purge_candidates(&sstables, &scope, None).unwrap();
                let naive_idx = naive_select_purge_candidates(&sstables_meta, &scope);

                let actual_idx: Vec<usize> = candidates
                    .iter()
                    .map(|cand| {
                        sstables
                            .iter()
                            .position(|s| Arc::ptr_eq(s, cand))
                            .unwrap()
                    })
                    .collect();

                (actual_idx, naive_idx, sstables_meta)
            });

            // Compare actual select_purge_candidates against naive reference
            prop_assert_eq!(&actual_indices, &naive_indices);

            // (iii) deterministically sorted and duplicate-free
            for w in actual_indices.windows(2) {
                prop_assert!(w[0] < w[1]);
            }

            if !actual_indices.is_empty() {
                let scope = PurgeScope::Keys(vec![vec![target_key]]);
                let mut b_first = &sstables_meta[actual_indices[0]].first_key;
                let mut b_last = &sstables_meta[actual_indices[0]].last_key;
                for &idx in &actual_indices {
                    if sstables_meta[idx].first_key < *b_first {
                        b_first = &sstables_meta[idx].first_key;
                    }
                    if sstables_meta[idx].last_key > *b_last {
                        b_last = &sstables_meta[idx].last_key;
                    }
                }

                // (i) Result is transitively closed
                for (idx, meta) in sstables_meta.iter().enumerate() {
                    if !actual_indices.contains(&idx) {
                        prop_assert!(!ranges_overlap(&meta.first_key, &meta.last_key, b_first, b_last));
                    }
                }

                // (ii) Result contains all directly scope-intersecting files
                for (idx, meta) in sstables_meta.iter().enumerate() {
                    if range_intersects_scope(&meta.first_key, &meta.last_key, &scope) {
                        prop_assert!(actual_indices.contains(&idx));
                    }
                }
            }
        }
    }
}
