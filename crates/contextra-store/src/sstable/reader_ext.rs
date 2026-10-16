use super::block_search::{binary_search_index_in_block, get_entry_at_index};
use super::reader::{parse_block_trailer, SstableReader};
use bytes::Bytes;
use contextra_core::{ContextraError, Result};

impl SstableReader {
    /// Scans the SSTable for keys starting with the given prefix.
    pub async fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Bytes, Bytes, u64, u64)>> {
        let mut results = Vec::with_capacity(16);

        let mut start_idx = match self.index.binary_search_by(|(k, _)| k.as_ref().cmp(prefix)) {
            Ok(i) => i,
            Err(i) => {
                if i > 0 {
                    i - 1
                } else {
                    0
                }
            }
        };

        // Rewind start_idx to the earliest block whose last_key >= prefix
        while start_idx > 0 && self.index[start_idx - 1].0.as_ref() >= prefix {
            start_idx -= 1;
        }

        for idx in start_idx..self.index.len() {
            let offset = self
                .index
                .get(idx)
                .ok_or_else(|| ContextraError::Storage("index out of bounds".into()))?
                .1;
            let next_offset = if idx + 1 < self.index.len() {
                self.index
                    .get(idx + 1)
                    .ok_or_else(|| ContextraError::Storage("index out of bounds".into()))?
                    .1
            } else {
                self.index_offset
            };

            let block_data = self.get_block(offset, next_offset).await?;

            let (num_offsets, offsets_start, _) =
                parse_block_trailer(&block_data, self.format_version)?;

            let is_v3 = self.format_version >= 3;
            let block_start_i = match binary_search_index_in_block(
                &block_data,
                offsets_start,
                num_offsets,
                prefix,
                self.format_version >= 3,
            )? {
                Ok(i) => i,
                Err(i) => i,
            };

            let mut broke = false;
            for i in block_start_i..num_offsets {
                let (entry_off, k_len) = get_entry_at_index(&block_data, offsets_start, i, is_v3)?;

                let mut ep = entry_off.checked_add(2).ok_or_else(|| {
                    ContextraError::Storage("overflow calculating entry offset".into())
                })?;

                let entry_key = block_data
                    .get(
                        ep..ep.checked_add(k_len).ok_or_else(|| {
                            ContextraError::Storage("overflow calculating entry key length".into())
                        })?,
                    )
                    .ok_or_else(|| ContextraError::Storage("malformed block: entry_key".into()))?;
                ep += k_len;

                if !entry_key.starts_with(prefix) && entry_key > prefix {
                    broke = true;
                    break; // Passed prefix lexicographically
                }

                if entry_key.starts_with(prefix) {
                    let end_k = ep;
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
                    if end_val > block_data.len() {
                        return Err(ContextraError::Storage(
                            "malformed block: value length out of bounds".into(),
                        ));
                    }
                    let start_k = entry_off.checked_add(2).ok_or_else(|| {
                        ContextraError::Storage("overflow calculating key position".into())
                    })?;
                    let key_bytes = block_data.slice(start_k..end_k);
                    let val_bytes = block_data.slice(ep_val..end_val);
                    results.push((key_bytes, val_bytes, seq_no, tx_id));
                }
            }
            if broke {
                break;
            }
        }

        Ok(results)
    }

    /// Scans all entries within a key range.
    pub async fn scan_range(
        &self,
        start: std::ops::Bound<&[u8]>,
        end: std::ops::Bound<&[u8]>,
    ) -> Result<Vec<(Bytes, Bytes, u64, u64)>> {
        use std::ops::Bound;

        let mut results = Vec::with_capacity(16);
        if self.index.is_empty() {
            return Ok(results);
        }

        for idx in 0..self.index.len() {
            let offset = self
                .index
                .get(idx)
                .ok_or_else(|| ContextraError::Storage("index out of bounds".into()))?
                .1;
            let next_offset = if idx + 1 < self.index.len() {
                self.index
                    .get(idx + 1)
                    .ok_or_else(|| ContextraError::Storage("index out of bounds".into()))?
                    .1
            } else {
                self.index_offset
            };

            let block_data = self.get_block(offset, next_offset).await?;

            let (num_offsets, offsets_start, _) =
                parse_block_trailer(&block_data, self.format_version)?;

            let is_v3 = self.format_version >= 3;
            let start_offset_idx = match start {
                Bound::Included(s) | Bound::Excluded(s) => {
                    match binary_search_index_in_block(
                        &block_data,
                        offsets_start,
                        num_offsets,
                        s,
                        self.format_version >= 3,
                    )? {
                        Ok(idx) => idx,
                        Err(idx) => idx,
                    }
                }
                Bound::Unbounded => 0,
            };

            for i in start_offset_idx..num_offsets {
                let (entry_off, k_len) = get_entry_at_index(&block_data, offsets_start, i, is_v3)?;

                let mut ep = entry_off.checked_add(2).ok_or_else(|| {
                    ContextraError::Storage("overflow calculating entry offset".into())
                })?;

                let entry_key = block_data
                    .get(
                        ep..ep.checked_add(k_len).ok_or_else(|| {
                            ContextraError::Storage("overflow calculating entry key length".into())
                        })?,
                    )
                    .ok_or_else(|| ContextraError::Storage("malformed block: entry_key".into()))?;
                ep += k_len;

                // Check start bound
                let after_start = match start {
                    Bound::Included(s) => entry_key >= s,
                    Bound::Excluded(s) => entry_key > s,
                    Bound::Unbounded => true,
                };
                if !after_start {
                    continue;
                }

                // Check end bound
                let before_end = match end {
                    Bound::Included(e) => entry_key <= e,
                    Bound::Excluded(e) => entry_key < e,
                    Bound::Unbounded => true,
                };
                if !before_end {
                    return Ok(results); // Past the range, done
                }

                let end_k = ep;
                let ep_seq = end_k;
                let end_seq = ep_seq.checked_add(8).ok_or_else(|| {
                    ContextraError::Storage("overflow calculating seq position".into())
                })?;
                let seq_no = u64::from_le_bytes(
                    block_data
                        .get(ep_seq..end_seq)
                        .ok_or_else(|| ContextraError::Storage("malformed block: seq_no".into()))?
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
                        .ok_or_else(|| ContextraError::Storage("malformed block: tx_id".into()))?
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
                        .ok_or_else(|| ContextraError::Storage("malformed block: v_len".into()))?
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
                if end_val > block_data.len() {
                    return Err(ContextraError::Storage(
                        "malformed block: value length out of bounds".into(),
                    ));
                }
                let start_k = entry_off.checked_add(2).ok_or_else(|| {
                    ContextraError::Storage("overflow calculating key position".into())
                })?;
                let key_bytes = block_data.slice(start_k..end_k);
                let val_bytes = block_data.slice(ep_val..end_val);
                results.push((key_bytes, val_bytes, seq_no, tx_id));
            }
        }

        Ok(results)
    }
}
