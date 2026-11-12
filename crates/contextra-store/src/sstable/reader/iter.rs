//! SSTable Reader: Iteration und Streaming über alle SSTable-Einträge.
// ANCHOR[contextra-store:SST-RDR-130]

use super::parse_block_trailer;
use super::SstableReader;
use crate::error::{StoreError as ContextraError, StoreResult as Result};
use crate::sstable::block_search::get_entry_at_index;
use crate::sstable::builder::SstableMetadata;
use crate::sstable::stream::SstableStream;
use bytes::Bytes;
use std::sync::Arc;

impl SstableReader {
    pub fn metadata(&self) -> &SstableMetadata {
        &self.metadata
    }

    #[allow(clippy::unused_async)]
    pub async fn stream(self: &Arc<Self>) -> Result<SstableStream> {
        Ok(SstableStream {
            reader: Arc::clone(self),
            block_idx: 0,
            entry_idx: 0,
            current_block: None,
            num_offsets: 0,
            offsets_start: 0,
        })
    }

    /// Iterates over all entries in sorted key order (allocates memory for all entries).
    pub async fn iter(&self) -> Result<Vec<(Bytes, Bytes, u64)>> {
        let mut results = Vec::new();
        if self.index.is_empty() {
            return Ok(results);
        }

        let is_v3 = self.format_version >= 3;

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

            for i in 0..num_offsets {
                let (entry_off, k_len) = get_entry_at_index(&block_data, offsets_start, i, is_v3)?;

                let ep = entry_off.checked_add(2).ok_or_else(|| {
                    ContextraError::Storage("overflow calculating entry offset".into())
                })?;
                let end_k = ep.checked_add(k_len).ok_or_else(|| {
                    ContextraError::Storage("overflow calculating entry key length".into())
                })?;
                let _entry_key = block_data
                    .get(ep..end_k)
                    .ok_or_else(|| ContextraError::Storage("malformed block: entry_key".into()))?;

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
                let _tx_id = u64::from_le_bytes(
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
                if end_val > offsets_start {
                    return Err(ContextraError::Storage(
                        "malformed block: value length out of bounds".into(),
                    ));
                }
                let key_bytes = block_data.slice(ep..end_k);
                let val_bytes = block_data.slice(ep_val..end_val);

                results.push((key_bytes, val_bytes, seq_no));
            }
        }

        Ok(results)
    }
}
