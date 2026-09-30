use super::block_search::get_entry_at_index;
use super::reader::{parse_block_trailer, SstableReader};
use bytes::Bytes;
use contextra_core::{ContextraError, Result};
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

                let (num_offsets, offsets_start, _) =
                    parse_block_trailer(&block_data, self.reader.format_version)?;
                self.num_offsets = num_offsets;
                self.offsets_start = offsets_start;
                self.current_block = Some(block_data);
            }

            // Yield an entry from the current block
            if let Some(block_data) = &self.current_block {
                if self.entry_idx < self.num_offsets {
                    let is_v3 = self.reader.format_version >= 3;
                    let (entry_off, k_len) = get_entry_at_index(
                        block_data,
                        self.offsets_start,
                        self.entry_idx,
                        is_v3,
                    )?;

                    let mut ep = entry_off + 2;
                    let entry_key = block_data
                        .get(ep..ep + k_len)
                        .ok_or_else(|| ContextraError::Storage("missing entry_key".into()))?;
                    ep += k_len;

                    let seq_no = u64::from_le_bytes(
                        block_data
                            .get(ep..ep + 8)
                            .ok_or_else(|| ContextraError::Storage("missing seq_no".into()))?
                            .try_into()
                            .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
                    );
                    ep += 8;

                    let tx_id = u64::from_le_bytes(
                        block_data
                            .get(ep..ep + 8)
                            .ok_or_else(|| ContextraError::Storage("missing tx_id".into()))?
                            .try_into()
                            .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
                    );
                    ep += 8;

                    let v_len = usize::try_from(u32::from_le_bytes(
                        block_data
                            .get(ep..ep + 4)
                            .ok_or_else(|| ContextraError::Storage("missing v_len".into()))?
                            .try_into()
                            .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
                    ))
                    .map_err(|_| {
                        ContextraError::Storage("value length exceeds platform usize".into())
                    })?;
                    ep += 4;
                    if ep + v_len > block_data.len() {
                        return Err(ContextraError::Storage(
                            "malformed block: value length out of bounds".into(),
                        ));
                    }
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
