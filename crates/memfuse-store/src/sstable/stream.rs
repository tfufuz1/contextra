use super::reader::SstableReader;
use bytes::Bytes;
use memfuse_core::{MemFuseError, Result};
use std::sync::Arc;

pub struct SstableStream {
    pub(super) reader: Arc<SstableReader>,
    pub(super) block_idx: usize,
    pub(super) entry_idx: usize,
    pub(super) current_block: Option<Bytes>,
    pub(super) num_offsets: usize,
    pub(super) offsets_start: usize,
}

impl SstableStream {
    pub async fn next(&mut self) -> Result<Option<(Bytes, Bytes, u64, u64)>> {
        if self.reader.index.is_empty() {
            return Ok(None);
        }

        loop {
            // Load a new block if needed
            if self.current_block.is_none() || self.entry_idx >= self.num_offsets {
                if self.block_idx >= self.reader.index.len() {
                    return Ok(None);
                }

                let offset = self.reader.index[self.block_idx].1;
                let next_offset = if self.block_idx + 1 < self.reader.index.len() {
                    self.reader.index[self.block_idx + 1].1
                } else {
                    self.reader.index_offset
                };

                let block_data = self.reader.get_block(offset, next_offset).await?;
                self.block_idx += 1;
                self.entry_idx = 0;

                let n = block_data.len();
                if n < 10 {
                    continue; // Empty or malformed block, try next
                }

                self.num_offsets = u16::from_le_bytes(
                    block_data
                        .get(n.saturating_sub(2)..n)
                        .ok_or_else(|| MemFuseError::Storage("missing num_offsets".into()))?
                        .try_into()
                        .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                ) as usize;

                let offsets_len = self.num_offsets * 2;
                if n < 10 + offsets_len {
                    continue;
                }
                self.offsets_start = n - 2 - offsets_len;
                self.current_block = Some(block_data);
            }

            // Yield an entry from the current block
            if let Some(block_data) = &self.current_block {
                if self.entry_idx < self.num_offsets {
                    let off_pos = self.offsets_start + self.entry_idx * 2;
                    let entry_off = u16::from_le_bytes(
                        block_data
                            .get(off_pos..off_pos + 2)
                            .ok_or_else(|| MemFuseError::Storage("missing off_pos".into()))?
                            .try_into()
                            .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                    ) as usize;

                    let mut ep = entry_off;
                    let k_len = u16::from_le_bytes(
                        block_data
                            .get(ep..ep + 2)
                            .ok_or_else(|| MemFuseError::Storage("missing k_len".into()))?
                            .try_into()
                            .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                    ) as usize;
                    ep += 2;
                    let entry_key = block_data
                        .get(ep..ep + k_len)
                        .ok_or_else(|| MemFuseError::Storage("missing entry_key".into()))?;
                    ep += k_len;

                    let seq_no = u64::from_le_bytes(
                        block_data
                            .get(ep..ep + 8)
                            .ok_or_else(|| MemFuseError::Storage("missing seq_no".into()))?
                            .try_into()
                            .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                    );
                    ep += 8;

                    let tx_id = u64::from_le_bytes(
                        block_data
                            .get(ep..ep + 8)
                            .ok_or_else(|| MemFuseError::Storage("missing tx_id".into()))?
                            .try_into()
                            .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                    );
                    ep += 8;

                    let v_len = u32::from_le_bytes(
                        block_data
                            .get(ep..ep + 4)
                            .ok_or_else(|| MemFuseError::Storage("missing v_len".into()))?
                            .try_into()
                            .map_err(|_| MemFuseError::Storage("invalid slice".into()))?,
                    ) as usize;
                    ep += 4;
                    let entry_val = block_data.slice(ep..ep + v_len);
                    self.entry_idx += 1;
                    return Ok(Some((
                        Bytes::copy_from_slice(entry_key),
                        entry_val,
                        seq_no,
                        tx_id,
                    )));
                }
            }
            self.current_block = None;
        }
    }

    /// Optimized for compaction: avoids copying if possible.
    pub async fn next_entry(&mut self) -> Result<Option<(Bytes, Bytes, u64, u64)>> {
        self.next().await
    }
}
