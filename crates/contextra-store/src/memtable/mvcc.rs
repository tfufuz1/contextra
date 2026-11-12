//! MemTable MVCC-Operationen: Einfügen, Lesen, Rollback und Versionslisten (Ring 1)
//!
//! Verwaltet Point-in-Time-Lesen über Sequenznummern und atomare Rollback-Aktionen.

use bytes::Bytes;
use contextra_ids::TxId;
use contextra_mvcc::tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES;
use std::sync::atomic::Ordering;

use super::shard::shard_for;
use super::tombstone::TOMBSTONE_BIT;
use crate::memtable::MemTable;

impl MemTable {
    /// Inserts a key-value pair with sequence number and transaction id into the appropriate shard.
    pub fn put(&self, key: Bytes, value: Bytes, seq_no: u64, tx_id: u64) {
        let additional_size = key.len() + value.len() + STAGING_ENTRY_OVERHEAD_BYTES;
        let shard_idx = Self::shard_for(&key);
        let mut entries = self.shards[shard_idx].entries.write();

        let versions = entries.entry(key).or_insert_with(|| Vec::with_capacity(2));
        let raw_seq = seq_no & !TOMBSTONE_BIT;

        if versions
            .last()
            .map_or(true, |(last_s, _, _)| (*last_s & !TOMBSTONE_BIT) <= raw_seq)
        {
            // Fast path: sequence numbers are monotonically non-decreasing
            versions.push((seq_no, value, tx_id));
        } else {
            // Out-of-order insertion: find the correct sorted position for `raw_seq`
            let idx = match versions.binary_search_by_key(&raw_seq, |(s, _, _)| *s & !TOMBSTONE_BIT)
            {
                Ok(i) => {
                    let mut pos = i + 1;
                    while pos < versions.len() && (versions[pos].0 & !TOMBSTONE_BIT) == raw_seq {
                        pos += 1;
                    }
                    pos
                }
                Err(i) => i,
            };
            versions.insert(idx, (seq_no, value, tx_id));
        }

        // Note: Simple size tracking (sums all versions)
        self.size.fetch_add(additional_size, Ordering::Relaxed);

        // Track TX range
        loop {
            let current_min = self.min_tx.load(Ordering::Acquire);
            if tx_id >= current_min
                || self
                    .min_tx
                    .compare_exchange_weak(current_min, tx_id, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
            {
                break;
            }
        }
        loop {
            let current_max = self.max_tx.load(Ordering::Acquire);
            if tx_id <= current_max
                || self
                    .max_tx
                    .compare_exchange_weak(current_max, tx_id, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
            {
                break;
            }
        }
    }

    /// Returns the minimum and maximum transaction IDs contained in this memtable.
    pub fn tx_range(&self) -> (u64, u64) {
        (
            self.min_tx.load(Ordering::Acquire),
            self.max_tx.load(Ordering::Acquire),
        )
    }

    /// Rollback: Removes all entries belonging to `tx_id` across all shards.
    pub fn rollback(&self, tx_id: u64) {
        let mut total_freed_size = 0usize;
        let mut new_min_tx = u64::MAX;
        let mut new_max_tx = 0u64;
        let mut has_entries = false;

        for shard in &self.shards {
            let mut entries = shard.entries.write();
            entries.retain(|key, versions| {
                versions.retain(|(_seq, val, tx)| {
                    if *tx == tx_id {
                        total_freed_size += key.len() + val.len() + STAGING_ENTRY_OVERHEAD_BYTES;
                        false
                    } else {
                        new_min_tx = new_min_tx.min(*tx);
                        new_max_tx = new_max_tx.max(*tx);
                        has_entries = true;
                        true
                    }
                });
                !versions.is_empty()
            });
        }

        if total_freed_size > 0 {
            #[cfg(debug_assertions)]
            {
                let before = self.size.load(Ordering::Acquire);
                debug_assert!(
                    before >= total_freed_size,
                    "MemTable size underflow detected: before={before}, freed={total_freed_size}"
                );
            }
            self.saturating_sub_size(total_freed_size);
        }

        if !has_entries {
            self.min_tx.store(u64::MAX, Ordering::Release);
            self.max_tx.store(0, Ordering::Release);
        } else {
            self.min_tx.store(new_min_tx, Ordering::Release);
            self.max_tx.store(new_max_tx, Ordering::Release);
        }
    }

    /// Retrieves the latest value and sequence number for a key.
    pub fn get(&self, key: &[u8]) -> Option<(Bytes, u64)> {
        let shard_idx = Self::shard_for(key);
        let entries = self.shards[shard_idx].entries.read();
        entries
            .get(key)
            .and_then(|versions| versions.last().map(|(seq, val, _tx)| (val.clone(), *seq)))
    }

    /// Retrieves a value, sequence number, and transaction ID by key at or below a specific sequence number
    /// and bounded by maximum transaction ID for Snapshot Isolation.
    pub fn get_at_seq(&self, key: &[u8], seq_no: u64, max_tx: u64) -> Option<(Bytes, u64, u64)> {
        let shard_idx = Self::shard_for(key);
        let entries = self.shards[shard_idx].entries.read();
        let versions = entries.get(key)?;

        let target_raw_seq = seq_no & !TOMBSTONE_BIT;

        let idx =
            match versions.binary_search_by_key(&target_raw_seq, |(s, _, _)| *s & !TOMBSTONE_BIT) {
                Ok(i) => {
                    let mut last = i;
                    while last + 1 < versions.len()
                        && (versions[last + 1].0 & !TOMBSTONE_BIT) == target_raw_seq
                    {
                        last += 1;
                    }
                    last
                }
                Err(i) => {
                    if i == 0 {
                        return None;
                    }
                    i - 1
                }
            };

        // Linear search backwards for the latest version satisfying (tx <= max_tx || tx >= INTERNAL_BASE)
        for i in (0..=idx).rev() {
            let (s, v, tx) = &versions[i];
            if *tx <= max_tx || *tx >= TxId::INTERNAL_BASE {
                return Some((v.clone(), *s, *tx));
            }
        }
        None
    }
}
