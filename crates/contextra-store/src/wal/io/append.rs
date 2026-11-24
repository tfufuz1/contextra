//! WAL Append-Routinen, Payload-Vorbereitung und Enqueueing.
//! Low-level WAL IO Submodule.
use contextra_crypto::wal_crypto::{IntegrityVerifier, WalEntrySnapshot};
use crate::error::{StoreError, StoreResult as Result};
use contextra_ids::TxId;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::wal::legacy_integrity_key;
use crate::wal::{
    PreparedBatch, Wal, WalCommand, WalEntry, WalOp, WalVersion, MAX_WAL_ENTRY_SIZE, WAL_V2_HEADER,
    WAL_V3_HEADER,
};

#[cfg(feature = "fault-injection")]
use crate::wal::{DELAY_APPEND_FOR_TX, DELAY_APPEND_MS, FAIL_APPEND_FOR_TX};

impl Wal {
    pub(crate) async fn prepare_append_payload(
        &self,
        batch: &PreparedBatch,
    ) -> Result<(Vec<u8>, [u8; 32])> {
        if self.is_sealed() {
            return Err(StoreError::Storage(format!(
                "Cannot append to sealed WAL segment {}",
                self.path.display()
            )));
        }

        let entries = &batch.0;
        if entries.is_empty() {
            return Ok((Vec::new(), [0u8; 32]));
        }

        #[cfg(feature = "fault-injection")]
        {
            let fail_tx = FAIL_APPEND_FOR_TX.load(std::sync::atomic::Ordering::SeqCst);
            if fail_tx != 0
                && entries
                    .iter()
                    .any(|e| e.tx_id().inner() == fail_tx || fail_tx == u64::MAX)
            {
                FAIL_APPEND_FOR_TX.store(0, std::sync::atomic::Ordering::SeqCst);
                return Err(StoreError::Storage(
                    "Simulated WAL append_batch I/O failure via fault injection".into(),
                ));
            }

            let delay_tx = DELAY_APPEND_FOR_TX.load(std::sync::atomic::Ordering::SeqCst);
            if delay_tx != 0 && entries.iter().any(|e| e.tx_id().inner() == delay_tx) {
                let delay_ms = DELAY_APPEND_MS.load(std::sync::atomic::Ordering::SeqCst);
                DELAY_APPEND_FOR_TX.store(0, std::sync::atomic::Ordering::SeqCst);
                if delay_ms > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                }
            }
        }

        let estimated_size = entries.len() * 256;
        let mut payload_bytes = Vec::with_capacity(estimated_size);
        let mut last_hmac_val = [0u8; 32];

        if let Some(km) = &self.key_manager {
            let max_frame_len = MAX_WAL_ENTRY_SIZE as usize;
            let max_plaintext_per_frame = max_frame_len.saturating_sub(28);

            let mut serialized_entries = Vec::with_capacity(entries.len());
            for entry in entries {
                let bytes = entry.to_bytes()?;
                let inner_record_len = bytes.len();

                if inner_record_len > max_plaintext_per_frame {
                    return Err(StoreError::invalid_input(format!(
                        "WAL entry too large for frame: {} bytes plaintext (max frame payload {})",
                        inner_record_len, max_plaintext_per_frame
                    )));
                }

                let is_tx_end = matches!(entry.op, WalOp::TxEnd { .. });
                serialized_entries.push((is_tx_end, bytes, inner_record_len));
            }

            let km_clone = Arc::clone(km);
            let frames_payload = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
                let mut out_bytes = Vec::with_capacity(estimated_size);
                let mut current_frame_plaintext = Vec::new();

                for (is_tx_end, bytes, inner_record_len) in serialized_entries {
                    if !current_frame_plaintext.is_empty()
                        && current_frame_plaintext
                            .len()
                            .saturating_add(inner_record_len)
                            > max_plaintext_per_frame
                    {
                        let (encrypted, nonce) =
                            km_clone.encrypt_auto_nonce(&current_frame_plaintext)?;
                        let chunk_len_usize =
                            12usize.checked_add(encrypted.len()).ok_or_else(|| {
                                StoreError::invalid_input("Frame length calculation overflow")
                            })?;
                        let chunk_len = u32::try_from(chunk_len_usize).map_err(|_| {
                            StoreError::invalid_input("Frame length exceeds u32")
                        })?;
                        if chunk_len > MAX_WAL_ENTRY_SIZE {
                            return Err(StoreError::invalid_input(format!(
                                "Frame size {} exceeds MAX_WAL_ENTRY_SIZE {}",
                                chunk_len, MAX_WAL_ENTRY_SIZE
                            )));
                        }
                        out_bytes.extend_from_slice(&chunk_len.to_le_bytes());
                        out_bytes.extend_from_slice(&nonce);
                        out_bytes.extend_from_slice(&encrypted);

                        current_frame_plaintext.clear();
                    }

                    current_frame_plaintext.extend_from_slice(&bytes);

                    if is_tx_end {
                        let (encrypted, nonce) =
                            km_clone.encrypt_auto_nonce(&current_frame_plaintext)?;
                        let chunk_len_usize =
                            12usize.checked_add(encrypted.len()).ok_or_else(|| {
                                StoreError::invalid_input("Frame length calculation overflow")
                            })?;
                        let chunk_len = u32::try_from(chunk_len_usize).map_err(|_| {
                            StoreError::invalid_input("Frame length exceeds u32")
                        })?;
                        if chunk_len > MAX_WAL_ENTRY_SIZE {
                            return Err(StoreError::invalid_input(format!(
                                "Frame size {} exceeds MAX_WAL_ENTRY_SIZE {}",
                                chunk_len, MAX_WAL_ENTRY_SIZE
                            )));
                        }
                        out_bytes.extend_from_slice(&chunk_len.to_le_bytes());
                        out_bytes.extend_from_slice(&nonce);
                        out_bytes.extend_from_slice(&encrypted);

                        current_frame_plaintext.clear();
                    }
                }

                if !current_frame_plaintext.is_empty() {
                    let (encrypted, nonce) =
                        km_clone.encrypt_auto_nonce(&current_frame_plaintext)?;
                    let chunk_len_usize =
                        12usize.checked_add(encrypted.len()).ok_or_else(|| {
                            StoreError::invalid_input("Frame length calculation overflow")
                        })?;
                    let chunk_len = u32::try_from(chunk_len_usize)
                        .map_err(|_| StoreError::invalid_input("Frame length exceeds u32"))?;
                    if chunk_len > MAX_WAL_ENTRY_SIZE {
                        return Err(StoreError::invalid_input(format!(
                            "Frame size {} exceeds MAX_WAL_ENTRY_SIZE {}",
                            chunk_len, MAX_WAL_ENTRY_SIZE
                        )));
                    }
                    out_bytes.extend_from_slice(&chunk_len.to_le_bytes());
                    out_bytes.extend_from_slice(&nonce);
                    out_bytes.extend_from_slice(&encrypted);
                }

                Ok(out_bytes)
            })
            .await
            .map_err(|e| StoreError::Storage(format!("WAL encryption task panicked: {e}")))??;

            payload_bytes = frames_payload;
            if let Some(last) = entries.last() {
                last_hmac_val = last.checksum;
            }
        } else {
            for entry in entries {
                let bytes = entry.to_bytes()?;
                payload_bytes.extend_from_slice(&bytes);
                last_hmac_val = entry.checksum;
            }
        }

        Ok((payload_bytes, last_hmac_val))
    }

    pub async fn append_batch(&self, batch: PreparedBatch) -> Result<()> {
        let ack_rx = {
            let truncate_guard = self.truncate_lock.lock().await;
            self.enqueue_append_batch_locked(batch, &truncate_guard)
                .await?
        };
        ack_rx
            .await
            .map_err(|_| StoreError::Storage("WAL flusher dropped".into()))??;
        Ok(())
    }

    pub async fn enqueue_append_batch_locked(
        &self,
        batch: PreparedBatch,
        _guard: &tokio::sync::MutexGuard<'_, ()>,
    ) -> Result<tokio::sync::oneshot::Receiver<Result<()>>> {
        if let Some(cause) = self.poison_cause() {
            return Err(StoreError::Storage(format!(
                "WAL handle poisoned ({cause:?}); requires explicit recovery or reload before further appends"
            )));
        }
        let entries = &batch.0;
        if entries.is_empty() {
            let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
            let _ = ack_tx.send(Ok(()));
            return Ok(ack_rx);
        }

        let (payload_bytes, last_hmac_val) = self.prepare_append_payload(&batch).await?;

        let flusher_tx = {
            let guard = self.flusher_tx.read().unwrap_or_else(|e| e.into_inner());
            guard.clone()
        };

        let tx = flusher_tx
            .ok_or_else(|| StoreError::Storage("WAL flusher actor is not enabled".into()))?;

        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        tx.send(WalCommand::Append {
            payload: payload_bytes,
            last_hmac_val,
            ack: ack_tx,
        })
        .await
        .map_err(|_| StoreError::Storage("WAL flusher channel closed".into()))?;

        Ok(ack_rx)
    }

    pub async fn append_batch_locked(
        &self,
        batch: PreparedBatch,
        guard: &tokio::sync::MutexGuard<'_, ()>,
    ) -> Result<()> {
        let ack_rx = self.enqueue_append_batch_locked(batch, guard).await?;
        ack_rx
            .await
            .map_err(|_| StoreError::Storage("WAL flusher dropped".into()))??;
        Ok(())
    }

    /// Non-blocking attempt to append a prepared batch. Returns `Err(StoreError::Storage("WAL queue full (backpressure)"))`
    /// if the flusher channel buffer is full.
    pub async fn try_append_batch(&self, batch: PreparedBatch) -> Result<()> {
        let ack_rx = {
            let truncate_guard = self.truncate_lock.lock().await;
            self.try_enqueue_append_batch_locked(batch, &truncate_guard)
                .await?
        };
        ack_rx
            .await
            .map_err(|_| StoreError::Storage("WAL flusher dropped".into()))??;
        Ok(())
    }

    pub async fn try_enqueue_append_batch_locked(
        &self,
        batch: PreparedBatch,
        _guard: &tokio::sync::MutexGuard<'_, ()>,
    ) -> Result<tokio::sync::oneshot::Receiver<Result<()>>> {
        if let Some(cause) = self.poison_cause() {
            return Err(StoreError::Storage(format!(
                "WAL handle poisoned ({cause:?}); requires explicit recovery or reload before further appends"
            )));
        }
        let entries = &batch.0;
        if entries.is_empty() {
            let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
            let _ = ack_tx.send(Ok(()));
            return Ok(ack_rx);
        }

        let (payload_bytes, last_hmac_val) = self.prepare_append_payload(&batch).await?;

        let flusher_tx = {
            let guard = self.flusher_tx.read().unwrap_or_else(|e| e.into_inner());
            guard.clone()
        };

        let tx = flusher_tx
            .ok_or_else(|| StoreError::Storage("WAL flusher actor is not enabled".into()))?;

        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        tx.try_send(WalCommand::Append {
            payload: payload_bytes,
            last_hmac_val,
            ack: ack_tx,
        })
        .map_err(|e| match e {
            tokio::sync::mpsc::error::TrySendError::Full(_) => {
                StoreError::Storage("WAL queue full (backpressure)".into())
            }
            tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                StoreError::Storage("WAL flusher channel closed".into())
            }
        })?;

        Ok(ack_rx)
    }

    pub async fn try_append_batch_locked(
        &self,
        batch: PreparedBatch,
        guard: &tokio::sync::MutexGuard<'_, ()>,
    ) -> Result<()> {
        let ack_rx = self.try_enqueue_append_batch_locked(batch, guard).await?;
        ack_rx
            .await
            .map_err(|_| StoreError::Storage("WAL flusher dropped".into()))??;
        Ok(())
    }
}
