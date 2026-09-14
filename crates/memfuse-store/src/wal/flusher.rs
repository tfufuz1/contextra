use memfuse_core::{MemFuseError, Result};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

use super::{Wal, WAL_V3_HEADER};

#[derive(Debug)]
pub(crate) struct FlusherMessage {
    pub payload: Vec<u8>,
    pub last_hmac_val: [u8; 32],
    pub ack: tokio::sync::oneshot::Sender<Result<()>>,
}

/// Configuration for the WAL background flusher actor.
///
/// Controls how aggressively the flusher coalesces concurrent writes
/// into a single `sync_all()` call to reduce fsync overhead under load.
#[derive(Debug, Clone, Copy)]
pub struct WalFlusherConfig {
    /// Maximum time to wait for additional messages after receiving the first,
    /// before issuing `sync_all()`. Set to 0 to disable (immediate flush).
    ///
    /// Typical range: 50–500 µs. Higher values coalesce more writes per fsync
    /// at the cost of added tail latency. Default: 100 µs.
    pub batch_window_micros: u64,
}

impl Default for WalFlusherConfig {
    fn default() -> Self {
        Self {
            batch_window_micros: 100,
        }
    }
}

impl Wal {
    pub(crate) fn enable_flusher(&self) {
        self.enable_flusher_with_config(WalFlusherConfig::default());
    }

    /// Enables background flusher actor for coalescing concurrent WAL writes and fsync calls.
    pub(crate) fn enable_flusher_with_config(&self, config: WalFlusherConfig) {
        let mut tx_guard = self.flusher_tx.write().unwrap_or_else(|e| e.into_inner());
        if tx_guard.is_some() {
            return;
        }

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<FlusherMessage>();
        let file = Arc::clone(&self.file);
        let path = self.path.clone();
        let header_written = Arc::clone(&self.header_written);
        let size = Arc::clone(&self.size);
        let last_hmac = Arc::clone(&self.last_hmac);

        tokio::spawn(async move {
            while let Some(first_msg) = rx.recv().await {
                let mut batch_payload = Vec::new();
                let mut acks = Vec::new();
                let mut final_last_hmac_val = first_msg.last_hmac_val;

                batch_payload.extend_from_slice(&first_msg.payload);
                acks.push(first_msg.ack);

                // Drain immediately available messages (zero-cost fast path)
                while let Ok(msg) = rx.try_recv() {
                    batch_payload.extend_from_slice(&msg.payload);
                    final_last_hmac_val = msg.last_hmac_val;
                    acks.push(msg.ack);
                }

                // Accumulation window: wait for additional messages before fsync.
                // This coalesces concurrent writers that enqueue slightly after the
                // first message, reducing total sync_all() call count under load.
                if config.batch_window_micros > 0 {
                    let deadline = tokio::time::Instant::now()
                        + tokio::time::Duration::from_micros(config.batch_window_micros);
                    loop {
                        match tokio::time::timeout_at(deadline, rx.recv()).await {
                            Ok(Some(msg)) => {
                                batch_payload.extend_from_slice(&msg.payload);
                                final_last_hmac_val = msg.last_hmac_val;
                                acks.push(msg.ack);
                                // Drain any further immediately available messages
                                while let Ok(more) = rx.try_recv() {
                                    batch_payload.extend_from_slice(&more.payload);
                                    final_last_hmac_val = more.last_hmac_val;
                                    acks.push(more.ack);
                                }
                            }
                            Ok(None) => break, // Channel closed — flusher task will exit on next iteration
                            Err(_deadline_elapsed) => break, // Window expired — proceed to fsync
                        }
                    }
                }

                let res: Result<()> = async {
                    let mut file_guard = file.lock().await;
                    let write_header = !header_written.load(std::sync::atomic::Ordering::Acquire)
                        && size.load(std::sync::atomic::Ordering::Acquire) == 0;

                    if write_header {
                        file_guard.write_all(&WAL_V3_HEADER).await.map_err(|e| {
                            MemFuseError::Storage(format!(
                                "WAL flusher header write failed for {}: {}",
                                path.display(),
                                e
                            ))
                        })?;
                    }
                    file_guard.write_all(&batch_payload).await.map_err(|e| {
                        MemFuseError::Storage(format!(
                            "WAL flusher write failed for {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                    file_guard.flush().await.map_err(|e| {
                        MemFuseError::Storage(format!(
                            "WAL flusher flush failed for {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                    file_guard.sync_all().await.map_err(|e| {
                        MemFuseError::Storage(format!(
                            "WAL flusher fsync failed for {}: {}",
                            path.display(),
                            e
                        ))
                    })?;

                    if write_header {
                        header_written.store(true, std::sync::atomic::Ordering::Release);
                    }

                    let written_len =
                        (if write_header { WAL_V3_HEADER.len() } else { 0 }) + batch_payload.len();
                    size.fetch_add(written_len as u64, std::sync::atomic::Ordering::SeqCst);

                    let mut last_hmac_guard = last_hmac.lock().await;
                    *last_hmac_guard = final_last_hmac_val;

                    Ok(())
                }
                .await;

                for ack in acks {
                    let send_res = match &res {
                        Ok(()) => Ok(()),
                        Err(e) => Err(MemFuseError::Storage(e.to_string())),
                    };
                    let _ = ack.send(send_res);
                }
            }
        });

        *tx_guard = Some(tx);
    }
}
