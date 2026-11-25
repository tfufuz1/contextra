use super::reader::SstableReader;
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
                    super::reader::parse_block_trailer(&block_data, self.reader.format_version)?;
                self.num_offsets = num_offsets;
                self.offsets_start = offsets_start;
                self.current_block = Some(block_data);
            }

            // Yield an entry from the current block
            if let Some(block_data) = &self.current_block {
                if self.entry_idx < self.num_offsets {
                    let is_v3 = self.reader.format_version >= 3;
                    let (entry_off, k_len) = super::block_search::get_entry_at_index(
                        block_data,
                        self.offsets_start,
                        self.entry_idx,
                        is_v3,
                    )?;

                    let ep = entry_off.checked_add(2).ok_or_else(|| {
                        ContextraError::Storage("overflow calculating entry offset".into())
                    })?;
                    let end_k = ep.checked_add(k_len).ok_or_else(|| {
                        ContextraError::Storage("overflow calculating entry key length".into())
                    })?;
                    let entry_key = block_data
                        .get(ep..end_k)
                        .ok_or_else(|| ContextraError::Storage("missing entry_key".into()))?;

                    let ep_seq = end_k;
                    let end_seq = ep_seq.checked_add(8).ok_or_else(|| {
                        ContextraError::Storage("overflow calculating seq position".into())
                    })?;
                    let seq_no = u64::from_le_bytes(
                        block_data
                            .get(ep_seq..end_seq)
                            .ok_or_else(|| ContextraError::Storage("missing seq_no".into()))?
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
                            .ok_or_else(|| ContextraError::Storage("missing tx_id".into()))?
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
                            .ok_or_else(|| ContextraError::Storage("missing v_len".into()))?
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
                    let entry_val = block_data.slice(ep_val..end_val);
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
