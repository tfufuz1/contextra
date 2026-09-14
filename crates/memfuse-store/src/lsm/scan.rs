use super::*;
use memfuse_core::{Result, TOMBSTONE_BIT};

pub(super) enum SstableScanMode<'a> {
    Prefix(&'a [u8]),
    Range(std::ops::Bound<&'a [u8]>, std::ops::Bound<&'a [u8]>),
}

#[inline]
pub(super) fn check_in_range(
    k: &[u8],
    start: std::ops::Bound<&[u8]>,
    end: std::ops::Bound<&[u8]>,
) -> bool {
    use std::ops::Bound;
    (match start {
        Bound::Included(s) => k >= s,
        Bound::Excluded(s) => k > s,
        Bound::Unbounded => true,
    }) && (match end {
        Bound::Included(e) => k <= e,
        Bound::Excluded(e) => k < e,
        Bound::Unbounded => true,
    })
}

impl LsmStorage {
    /// Evaluates detailed traversal metrics (evaluated_sstables, bloom_passes, range_passes, block_reads, found) for point lookups.
    pub async fn point_lookup_metrics(&self, key: &[u8]) -> (usize, usize, usize, usize, bool) {
        let sstables = self.sstables.read().await;
        let mut total_eval = 0usize;
        let mut total_bloom_pass = 0usize;
        let mut total_range_pass = 0usize;
        let mut total_block_read = 0usize;
        let mut found = false;

        for sst in sstables.iter().rev() {
            total_eval += 1;
            let (b_pass, r_pass, blk_read, k_found) = sst.lookup_metrics(key).await;
            if b_pass {
                total_bloom_pass += 1;
            }
            if r_pass {
                total_range_pass += 1;
            }
            if blk_read {
                total_block_read += 1;
            }
            if k_found {
                found = true;
                break;
            }
        }

        (
            total_eval,
            total_bloom_pass,
            total_range_pass,
            total_block_read,
            found,
        )
    }

    /// Internal helper that collects visible entries across SSTables, immutable MemTables,
    /// and active MemTable according to MVCC visibility, tombstone masking, and optional accumulator limits.
    pub(super) async fn collect_visible_entries<F>(
        &self,
        mode: SstableScanMode<'_>,
        entry_filter: F,
        check_accumulator: bool,
        per_source_limit: Option<usize>,
        context_name: &str,
    ) -> Result<std::collections::BTreeMap<Bytes, (Bytes, u64)>>
    where
        F: Fn(&[u8], u64, u64) -> bool,
    {
        let mut map: std::collections::BTreeMap<Bytes, (Bytes, u64)> =
            std::collections::BTreeMap::new();
        let state = self.state.read().await;
        let sstables = self.sstables.read().await;
        // H-7 FIX: last_tx NACH Lock-Erwerb laden für korrekte Snapshot-Isolation.
        let last_tx = self.last_committed_tx.load(Ordering::Acquire);

        // 1. SSTables
        for sst in sstables.iter() {
            let entries = match mode {
                SstableScanMode::Prefix(prefix) => {
                    let first = sst.first_key();
                    let last = sst.last_key();
                    if !first.is_empty() && !last.is_empty() {
                        if prefix > last.as_ref() {
                            continue;
                        }
                        let mut prefix_end = prefix.to_vec();
                        if let Some(last_byte) = prefix_end.last_mut() {
                            if let Some(next_byte) = last_byte.checked_add(1) {
                                *last_byte = next_byte;
                                if first.as_ref() >= prefix_end.as_slice() {
                                    continue;
                                }
                            }
                        }
                    }
                    sst.scan_prefix(prefix).await?
                }
                SstableScanMode::Range(start, end) => {
                    sst.scan_range(start.map(|s| s), end.map(|e| e)).await?
                }
            };

            let mut source_count = 0usize;
            for (k, v, seq, tx) in entries {
                let raw_seq = seq & !TOMBSTONE_BIT;
                if (tx <= last_tx || tx >= TxId::INTERNAL_BASE)
                    && entry_filter(k.as_ref(), raw_seq, tx)
                {
                    let entry = map.entry(k).or_insert_with(|| (v.clone(), seq));
                    if (seq & !TOMBSTONE_BIT) > (entry.1 & !TOMBSTONE_BIT) {
                        *entry = (v, seq);
                    }
                    if check_accumulator && map.len() > memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR {
                        return Err(MemFuseError::LimitExceeded {
                            limit: memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR,
                            context: format!(
                                "{context_name}: internal merge accumulator exceeded — range too wide, narrow the scan range"
                            ),
                        });
                    }
                    source_count += 1;
                    if let Some(lim) = per_source_limit {
                        if source_count >= lim {
                            break;
                        }
                    }
                }
            }
        }

        // Helper inline closure to scan a MemTable using range/prefix or full-scan fallback
        let scan_memtable =
            |mt: &MemTable, target: &mut std::collections::BTreeMap<Bytes, (Bytes, u64)>| {
                match mode {
                    SstableScanMode::Prefix(prefix) => {
                        mt.scan_prefix_into_matching(
                            prefix,
                            u64::MAX,
                            TxId(last_tx),
                            target,
                            &entry_filter,
                        );
                    }
                    SstableScanMode::Range(
                        std::ops::Bound::Unbounded,
                        std::ops::Bound::Unbounded,
                    ) => {
                        // Fallback: Vollscan ohne Range-Einschränkung iteriert über alle Einträge
                        for (k, v, seq, tx) in mt.iter() {
                            if tx > last_tx && tx < TxId::INTERNAL_BASE {
                                continue;
                            }
                            let raw_seq = seq & !TOMBSTONE_BIT;
                            if entry_filter(k.as_ref(), raw_seq, tx) {
                                let entry =
                                    target.entry(k.clone()).or_insert_with(|| (v.clone(), seq));
                                if (seq & !TOMBSTONE_BIT) > (entry.1 & !TOMBSTONE_BIT) {
                                    *entry = (v.clone(), seq);
                                }
                            }
                        }
                    }
                    SstableScanMode::Range(start, end) => {
                        mt.scan_range_into_matching(
                            start,
                            end,
                            u64::MAX,
                            TxId(last_tx),
                            target,
                            &entry_filter,
                        );
                    }
                }
            };

        // 2. Immutable memtables (older -> newer)
        for mt in &state.immutable_memtables {
            scan_memtable(mt, &mut map);
            if check_accumulator && map.len() > memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR {
                return Err(MemFuseError::LimitExceeded {
                    limit: memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR,
                    context: format!(
                        "{context_name}: internal merge accumulator exceeded — range too wide, narrow the scan range"
                    ),
                });
            }
        }

        // 3. Active memtable
        scan_memtable(&state.memtable, &mut map);
        if check_accumulator && map.len() > memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR {
            return Err(MemFuseError::LimitExceeded {
                limit: memfuse_core::MAX_SCAN_MERGE_ACCUMULATOR,
                context: format!(
                    "{context_name}: internal merge accumulator exceeded — range too wide, narrow the scan range"
                ),
            });
        }

        Ok(map)
    }
}
