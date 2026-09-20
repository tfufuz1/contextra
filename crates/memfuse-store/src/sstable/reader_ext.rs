use super::block_search::binary_search_index_in_block;
use super::reader::SstableReader;
use bytes::Bytes;
use memfuse_core::{MemFuseError, Result};

impl SstableReader {
    /// Scans the SSTable for keys starting with the given prefix.
    pub async fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Bytes, Bytes, u64, u64)>> {
        let mut results = Vec::with_capacity(16);

        let start_idx = match self.index.binary_search_by(|(k, _)| k.as_ref().cmp(prefix)) {
            Ok(i) => i,
            Err(i) => {
                if i > 0 {
                    i - 1
                } else {
                    0
                }
            }
        };

        for idx in start_idx..self.index.len() {
            let offset = self
                .index
                .get(idx)
                .ok_or_else(|| MemFuseError::Storage("index out of bounds".into()))?
                .1;
            let next_offset = if idx + 1 < self.index.len() {
                self.index
                    .get(idx + 1)
                    .ok_or_else(|| MemFuseError::Storage("index out of bounds".into()))?
                    .1
            } else {
                self.index_offset
            };

            let block_data = self.get_block(offset, next_offset).await?;

            let n = block_data.len();
            if n < 10 {
                continue;
            }

            let num_offsets = u16::from_le_bytes(
                block_data
                    .get(n.saturating_sub(2)..n)
                    .ok_or_else(|| {
                        MemFuseError::Storage("malformed block: missing num_offsets".into())
                    })?
                    .try_into()
                    .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
            ) as usize;

            let offsets_len = num_offsets * 2;
            if n < 10 + offsets_len {
                return Err(MemFuseError::Storage(
                    "malformed block: num_offsets too large".into(),
                ));
            }
            let offsets_start = n - 2 - offsets_len;

            let block_start_i = match binary_search_index_in_block(
                &block_data,
                offsets_start,
                num_offsets,
                prefix,
            )? {
                Ok(i) => i,
                Err(i) => i,
            };

            let mut broke = false;
            for i in block_start_i..num_offsets {
                let off_pos = offsets_start + i * 2;
                let entry_off = u16::from_le_bytes(
                    block_data
                        .get(off_pos..off_pos + 2)
                        .ok_or_else(|| MemFuseError::Storage("malformed block: off_pos".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                ) as usize;

                let mut ep = entry_off;
                let k_len = u16::from_le_bytes(
                    block_data
                        .get(ep..ep + 2)
                        .ok_or_else(|| MemFuseError::Storage("malformed block: k_len".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                ) as usize;
                ep += 2;
                let entry_key = block_data
                    .get(ep..ep + k_len)
                    .ok_or_else(|| MemFuseError::Storage("malformed block: entry_key".into()))?;
                ep += k_len;

                if !entry_key.starts_with(prefix) && entry_key > prefix {
                    broke = true;
                    break; // Passed prefix lexicographically
                }

                if entry_key.starts_with(prefix) {
                    let seq_no = u64::from_le_bytes(
                        block_data
                            .get(ep..ep + 8)
                            .ok_or_else(|| MemFuseError::Storage("malformed block: seq_no".into()))?
                            .try_into()
                            .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                    );
                    ep += 8;
                    let tx_id = u64::from_le_bytes(
                        block_data
                            .get(ep..ep + 8)
                            .ok_or_else(|| MemFuseError::Storage("malformed block: tx_id".into()))?
                            .try_into()
                            .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                    );
                    ep += 8;
                    let v_len = u32::from_le_bytes(
                        block_data
                            .get(ep..ep + 4)
                            .ok_or_else(|| MemFuseError::Storage("malformed block: v_len".into()))?
                            .try_into()
                            .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                    ) as usize;
                    ep += 4;
                    if ep + v_len > block_data.len() {
                        return Err(MemFuseError::Storage(
                            "malformed block: value length out of bounds".into(),
                        ));
                    }
                    let key_bytes = block_data.slice(entry_off + 2..entry_off + 2 + k_len);
                    let val_bytes = block_data.slice(ep..ep + v_len);
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
                .ok_or_else(|| MemFuseError::Storage("index out of bounds".into()))?
                .1;
            let next_offset = if idx + 1 < self.index.len() {
                self.index
                    .get(idx + 1)
                    .ok_or_else(|| MemFuseError::Storage("index out of bounds".into()))?
                    .1
            } else {
                self.index_offset
            };

            let block_data = self.get_block(offset, next_offset).await?;

            let n = block_data.len();
            if n < 10 {
                continue;
            }

            let num_offsets = u16::from_le_bytes(
                block_data
                    .get(n.saturating_sub(2)..n)
                    .ok_or_else(|| {
                        MemFuseError::Storage("malformed block: missing num_offsets".into())
                    })?
                    .try_into()
                    .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
            ) as usize;

            let offsets_len = num_offsets * 2;
            if n < 10 + offsets_len {
                return Err(MemFuseError::Storage(
                    "malformed block: num_offsets too large".into(),
                ));
            }
            let offsets_start = n - 2 - offsets_len;

            let start_offset_idx = match start {
                Bound::Included(s) | Bound::Excluded(s) => {
                    match binary_search_index_in_block(&block_data, offsets_start, num_offsets, s)?
                    {
                        Ok(idx) => idx,
                        Err(idx) => idx,
                    }
                }
                Bound::Unbounded => 0,
            };

            for i in start_offset_idx..num_offsets {
                let off_pos = offsets_start + i * 2;
                let entry_off = u16::from_le_bytes(
                    block_data
                        .get(off_pos..off_pos + 2)
                        .ok_or_else(|| MemFuseError::Storage("malformed block: off_pos".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                ) as usize;

                let mut ep = entry_off;
                let k_len = u16::from_le_bytes(
                    block_data
                        .get(ep..ep + 2)
                        .ok_or_else(|| MemFuseError::Storage("malformed block: k_len".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                ) as usize;
                ep += 2;
                let entry_key = block_data
                    .get(ep..ep + k_len)
                    .ok_or_else(|| MemFuseError::Storage("malformed block: entry_key".into()))?;
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

                let seq_no = u64::from_le_bytes(
                    block_data
                        .get(ep..ep + 8)
                        .ok_or_else(|| MemFuseError::Storage("malformed block: seq_no".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                );
                ep += 8;
                let tx_id = u64::from_le_bytes(
                    block_data
                        .get(ep..ep + 8)
                        .ok_or_else(|| MemFuseError::Storage("malformed block: tx_id".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                );
                ep += 8;
                let v_len = u32::from_le_bytes(
                    block_data
                        .get(ep..ep + 4)
                        .ok_or_else(|| MemFuseError::Storage("malformed block: v_len".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                ) as usize;
                ep += 4;
                if ep + v_len > block_data.len() {
                    return Err(MemFuseError::Storage(
                        "malformed block: value length out of bounds".into(),
                    ));
                }
                let key_bytes = block_data.slice(entry_off + 2..entry_off + 2 + k_len);
                let val_bytes = block_data.slice(ep..ep + v_len);
                results.push((key_bytes, val_bytes, seq_no, tx_id));
            }
        }

        Ok(results)
    }
}
