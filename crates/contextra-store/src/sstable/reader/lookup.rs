//! SSTable Reader: Punkt-Lookup und Bloom-Filter-Abfragen.
// ANCHOR[contextra-store:SST-RDR-120]

use super::may_contain_key_in_block_bloom;
use super::parse_block_trailer;
use super::SstableReader;
use crate::error::{StoreError as ContextraError, StoreResult as Result};
use crate::sstable::block_search::{
    binary_search_entry_in_block, binary_search_first_index_in_block, get_entry_at_index,
};
use bytes::Bytes;

/// Retrieves a value, sequence number, and transaction ID from the SSTable by key
/// bounded by maximum sequence number (`max_seq`) and maximum transaction ID (`max_tx`).
impl SstableReader {
    pub async fn get_at(
        &self,
        key: &[u8],
        max_seq: u64,
        max_tx: u64,
    ) -> Result<Option<(Bytes, u64, u64)>> {
        // 1. Whole-SSTable Bloom Filter Pre-check
        if let Some(bloom) = &self.bloom_filter {
            if !bloom.may_contain(key) {
                return Ok(None);
            }
        }

        if key < self.metadata.first_key || key > self.metadata.last_key {
            return Ok(None);
        }

        let mut start_idx = match self.index.binary_search_by(|(k, _)| k.as_ref().cmp(key)) {
            Ok(i) => i,
            Err(i) => i,
        };

        // Rewind start_idx to the earliest block whose last_key >= key (handles multi-block key spans)
        while start_idx > 0 && self.index[start_idx - 1].0.as_ref() >= key {
            start_idx -= 1;
        }

        let max_seq_raw = max_seq & !contextra_ids::TOMBSTONE_BIT;
        let is_v3 = self.format_version >= 3;

        for idx in start_idx..self.index.len() {
            let (block_last_key, offset) = &self.index[idx];
            let next_offset = if idx + 1 < self.index.len() {
                self.index[idx + 1].1
            } else {
                self.index_offset
            };

            let block_data = self.get_block(*offset, next_offset).await?;

            let (num_offsets, offsets_start, bloom_bytes) =
                parse_block_trailer(&block_data, self.format_version)?;

            let may_contain = may_contain_key_in_block_bloom(bloom_bytes, key, self.format_version);

            if may_contain {
                if let Some(first_entry_idx) = binary_search_first_index_in_block(
                    &block_data,
                    offsets_start,
                    num_offsets,
                    key,
                    is_v3,
                )? {
                    for entry_i in first_entry_idx..num_offsets {
                        let (entry_off, k_len) =
                            get_entry_at_index(&block_data, offsets_start, entry_i, is_v3)?;
                        let ep = entry_off.checked_add(2).ok_or_else(|| {
                            ContextraError::Storage("overflow calculating entry offset".into())
                        })?;
                        let end_k = ep.checked_add(k_len).ok_or_else(|| {
                            ContextraError::Storage("overflow calculating entry key length".into())
                        })?;
                        let entry_key = block_data
                            .get(ep..end_k)
                            .ok_or_else(|| ContextraError::Storage("malformed block".into()))?;
                        if entry_key != key {
                            break;
                        }

                        let ep_seq = end_k;
                        let end_seq = ep_seq.checked_add(8).ok_or_else(|| {
                            ContextraError::Storage("overflow calculating seq position".into())
                        })?;
                        let seq_no = u64::from_le_bytes(
                            block_data
                                .get(ep_seq..end_seq)
                                .ok_or_else(|| {
                                    ContextraError::Storage("malformed block: seq_no".into())
                                })?
                                .try_into()
                                .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
                        );

                        let ep_tx = end_seq;
                        let end_tx = ep_tx.checked_add(8).ok_or_else(|| {
                            ContextraError::Storage("overflow calculating tx position".into())
                        })?;
                        let tx_id = u64::from_le_bytes(
                            block_data
                                .get(ep_tx..end_tx)
                                .ok_or_else(|| {
                                    ContextraError::Storage("malformed block: tx_id".into())
                                })?
                                .try_into()
                                .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
                        );

                        let ep_vlen = end_tx;
                        let end_vlen = ep_vlen.checked_add(4).ok_or_else(|| {
                            ContextraError::Storage("overflow calculating v_len position".into())
                        })?;
                        let v_len = usize::try_from(u32::from_le_bytes(
                            block_data
                                .get(ep_vlen..end_vlen)
                                .ok_or_else(|| {
                                    ContextraError::Storage("malformed block: v_len".into())
                                })?
                                .try_into()
                                .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
                        ))
                        .map_err(|_| {
                            ContextraError::Storage("value length exceeds platform usize".into())
                        })?;

                        let ep_val = end_vlen;
                        let end_val = ep_val.checked_add(v_len).ok_or_else(|| {
                            ContextraError::Storage("overflow calculating value position".into())
                        })?;
                        if end_val > offsets_start {
                            return Err(ContextraError::Storage(
                                "malformed block: value length out of bounds".into(),
                            ));
                        }

                        let raw_seq = seq_no & !contextra_ids::TOMBSTONE_BIT;
                        if raw_seq <= max_seq_raw
                            && (tx_id <= max_tx || tx_id >= contextra_ids::TxId::INTERNAL_BASE)
                        {
                            let entry_val = block_data.slice(ep_val..end_val);
                            return Ok(Some((entry_val, seq_no, tx_id)));
                        }
                    }
                }
            }

            if block_last_key.as_ref() != key {
                break;
            }
        }

        Ok(None)
    }

    /// Retrieves a value from the SSTable by key.
    pub async fn get(&self, key: &[u8]) -> Result<Option<(Bytes, u64, u64)>> {
        self.get_at(key, u64::MAX, u64::MAX).await
    }

    /// Metrics result for point lookup instrumentation.
    pub async fn lookup_metrics(&self, key: &[u8]) -> (bool, bool, bool, bool) {
        let whole_bloom_passed = match &self.bloom_filter {
            Some(bloom) => bloom.may_contain(key),
            None => true,
        };

        if key < self.metadata.first_key.as_ref() || key > self.metadata.last_key.as_ref() {
            return (whole_bloom_passed, false, false, false);
        }

        let idx = match self.index.binary_search_by(|(k, _)| k.as_ref().cmp(key)) {
            Ok(i) => i,
            Err(i) => i,
        };

        if idx >= self.index.len() {
            return (whole_bloom_passed, true, false, false);
        }

        let offset = match self.index.get(idx) {
            Some((_, off)) => *off,
            None => return (whole_bloom_passed, true, false, false),
        };
        let next_offset = if idx + 1 < self.index.len() {
            match self.index.get(idx + 1) {
                Some((_, off)) => *off,
                None => self.index_offset,
            }
        } else {
            self.index_offset
        };

        let block_data = match self.get_block(offset, next_offset).await {
            Ok(b) => b,
            Err(_) => return (whole_bloom_passed, true, false, false),
        };

        let is_v3 = self.format_version >= 3;
        let (num_offsets, offsets_start, bloom_bytes) =
            match parse_block_trailer(&block_data, self.format_version) {
                Ok(t) => t,
                Err(_) => return (whole_bloom_passed, true, false, false),
            };

        let may_contain = may_contain_key_in_block_bloom(bloom_bytes, key, self.format_version);
        if !may_contain {
            return (whole_bloom_passed, true, false, false);
        }

        match binary_search_entry_in_block(&block_data, offsets_start, num_offsets, key, is_v3) {
            Ok(Some(_)) => (whole_bloom_passed, true, true, true),
            _ => (whole_bloom_passed, true, true, false),
        }
    }
}
