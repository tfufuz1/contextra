//! MemTable Iteration und Scans (Ring 1)
//!
//! Stellt Range-Scans, Präfix-Scans und geordnete Gesamtiteration bereit.

use bytes::Bytes;
use contextra_ids::TxId;
use std::collections::BTreeMap;
use std::ops::Bound;

use super::shard::{shard_range_for_bounds, shard_range_for_prefix};
use super::tombstone::{is_valid_range_bounds, TOMBSTONE_BIT};
use crate::memtable::MemTable;

impl MemTable {
    /// Iterates all entries across all shards in ascending key order.
    pub fn iter(&self) -> Vec<(Bytes, Bytes, u64, u64)> {
        let approx_cap = (self.size() / 64).max(16);
        let mut all_entries = Vec::with_capacity(approx_cap);

        for shard in &self.shards {
            let map = shard.entries.read();
            for (key, versions) in map.iter() {
                for &(seq, ref val, tx) in versions {
                    all_entries.push((key.clone(), val.clone(), seq, tx));
                }
            }
        }

        all_entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.2.cmp(&b.2)));
        all_entries
    }

    /// Scans entries matching a key prefix into `target`. Higher `seq_no` versions shadow lower `seq_no` versions.
    pub fn scan_prefix_into(
        &self,
        prefix: &[u8],
        seq_no: u64,
        max_tx: TxId,
        target: &mut BTreeMap<Bytes, (Bytes, u64)>,
    ) {
        self.scan_prefix_into_matching(prefix, seq_no, max_tx, target, |_, _, _| true);
    }

    /// Scans entries matching a key prefix into `target` with an additional matching predicate.
    pub fn scan_prefix_into_matching<F>(
        &self,
        prefix: &[u8],
        seq_no: u64,
        max_tx: TxId,
        target: &mut BTreeMap<Bytes, (Bytes, u64)>,
        predicate: F,
    ) where
        F: Fn(&[u8], u64, u64) -> bool,
    {
        let max_tx_val = max_tx.inner();
        let max_seq = seq_no & !TOMBSTONE_BIT;
        let shard_range = Self::shard_range_for_prefix(prefix);

        for shard_idx in shard_range {
            let entries = self.shards[shard_idx].entries.read();
            for (k, versions) in
                entries.range::<[u8], _>((Bound::Included(prefix), Bound::Unbounded))
            {
                if !k.starts_with(prefix) {
                    break;
                }
                let mut best_version: Option<(&Bytes, u64)> = None;
                for (seq, val, tx) in versions {
                    let raw_seq = seq & !TOMBSTONE_BIT;
                    if raw_seq <= max_seq
                        && (*tx <= max_tx_val || *tx >= TxId::INTERNAL_BASE)
                        && predicate(k.as_ref(), raw_seq, *tx)
                    {
                        match best_version {
                            Some((_, best_seq)) => {
                                if raw_seq > (best_seq & !TOMBSTONE_BIT) {
                                    best_version = Some((val, *seq));
                                }
                            }
                            None => {
                                best_version = Some((val, *seq));
                            }
                        }
                    }
                }
                if let Some((val, seq)) = best_version {
                    let raw_seq = seq & !TOMBSTONE_BIT;
                    let entry = target
                        .entry(k.clone())
                        .or_insert_with(|| (val.clone(), seq));
                    if raw_seq > (entry.1 & !TOMBSTONE_BIT) {
                        *entry = (val.clone(), seq);
                    }
                }
            }
        }
    }

    /// Scans entries matching a key range into `target` according to MVCC visibility rules.
    pub fn scan_range_into(
        &self,
        start: Bound<&[u8]>,
        end: Bound<&[u8]>,
        seq_no: u64,
        max_tx: TxId,
        target: &mut BTreeMap<Bytes, (Bytes, u64)>,
    ) {
        self.scan_range_into_matching(start, end, seq_no, max_tx, target, |_, _, _| true);
    }

    /// Scans entries matching a key range into `target` with an additional matching predicate.
    pub fn scan_range_into_matching<F>(
        &self,
        start: Bound<&[u8]>,
        end: Bound<&[u8]>,
        seq_no: u64,
        max_tx: TxId,
        target: &mut BTreeMap<Bytes, (Bytes, u64)>,
        predicate: F,
    ) where
        F: Fn(&[u8], u64, u64) -> bool,
    {
        if !is_valid_range_bounds(start, end) {
            return;
        }
        let max_tx_val = max_tx.inner();
        let max_seq = seq_no & !TOMBSTONE_BIT;
        let shard_range = Self::shard_range_for_bounds(start, end);

        for shard_idx in shard_range {
            let entries = self.shards[shard_idx].entries.read();
            for (k, versions) in entries.range::<[u8], _>((start, end)) {
                let mut best_version: Option<(&Bytes, u64)> = None;
                for (seq, val, tx) in versions {
                    let raw_seq = seq & !TOMBSTONE_BIT;
                    if raw_seq <= max_seq
                        && (*tx <= max_tx_val || *tx >= TxId::INTERNAL_BASE)
                        && predicate(k.as_ref(), raw_seq, *tx)
                    {
                        match best_version {
                            Some((_, best_seq)) => {
                                if raw_seq > (best_seq & !TOMBSTONE_BIT) {
                                    best_version = Some((val, *seq));
                                }
                            }
                            None => {
                                best_version = Some((val, *seq));
                            }
                        }
                    }
                }
                if let Some((val, seq)) = best_version {
                    let raw_seq = seq & !TOMBSTONE_BIT;
                    let entry = target
                        .entry(k.clone())
                        .or_insert_with(|| (val.clone(), seq));
                    if raw_seq > (entry.1 & !TOMBSTONE_BIT) {
                        *entry = (val.clone(), seq);
                    }
                }
            }
        }
    }

    /// Iterates over only the latest version of each key in sorted key order (including tombstones).
    pub fn iter_latest(&self) -> Vec<(Bytes, Bytes, u64, u64)> {
        let estimated_entries: usize = self.shards.iter().map(|s| s.entries.read().len()).sum();
        let mut results = Vec::with_capacity(estimated_entries);
        for shard in &self.shards {
            let entries = shard.entries.read();
            for (k, versions) in entries.iter() {
                if let Some((seq, val, tx)) = versions.last() {
                    results.push((k.clone(), val.clone(), *seq, *tx));
                }
            }
        }
        results.sort_by(|a, b| a.0.cmp(&b.0));
        results
    }
}
