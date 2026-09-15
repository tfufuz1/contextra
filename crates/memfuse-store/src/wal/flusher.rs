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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wal::{PreparedBatch, WalConfig, WalEntry, WalOp};
    use memfuse_core::TxId;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_wal_flusher_actor_coalescing() -> Result<()> {
        let dir = tempdir()?;
        let wal_path = dir.path().join("test_flusher.wal");

        let wal = Arc::new(Wal::open(&wal_path).await?);
        wal.enable_flusher();

        let num_tasks = 10;
        let mut handles = Vec::new();

        for i in 0..num_tasks {
            let wal_clone = Arc::clone(&wal);
            handles.push(tokio::spawn(async move {
                let op = WalOp::Put {
                    tx_id: TxId::new(i + 1),
                    key: format!("flusher_k_{i}").into_bytes(),
                    value: format!("flusher_v_{i}").into_bytes(),
                };
                let (batch, _) = wal_clone.prepare_batch(vec![(op, i + 1)]).await?;
                wal_clone.append_batch(batch).await
            }));
        }

        for h in handles {
            h.await
                .map_err(|e| MemFuseError::Storage(e.to_string()))??;
        }

        let replayed = wal.replay().await?;
        assert_eq!(replayed.len(), num_tasks as usize);

        for (i, (_seq, entry, _pos)) in replayed.iter().enumerate() {
            assert_eq!(entry.seq_no, (i + 1) as u64);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_flusher_batch_window_coalesces_writes() -> Result<()> {
        let dir = tempdir()?;
        let wal_path = dir.path().join("batch_window_test.wal");

        let wal = Arc::new(
            Wal::open_with_config(
                &wal_path,
                WalConfig {
                    flusher_config: WalFlusherConfig {
                        batch_window_micros: 50,
                    },
                    ..Default::default()
                },
            )
            .await?,
        );

        let num_tasks = 5;
        let mut handles = Vec::new();

        for i in 0u64..num_tasks {
            let wal_clone = Arc::clone(&wal);
            handles.push(tokio::spawn(async move {
                let op = WalOp::Put {
                    tx_id: TxId::new(i + 1),
                    key: format!("key-{i}").into_bytes(),
                    value: b"val".to_vec(),
                };
                let (batch, _) = wal_clone.prepare_batch(vec![(op, i + 1)]).await?;
                wal_clone.append_batch(batch).await
            }));
        }

        for h in handles {
            h.await
                .map_err(|e| MemFuseError::Storage(e.to_string()))??;
        }

        let replayed = wal.replay().await?;
        assert_eq!(
            replayed.len(),
            num_tasks as usize,
            "Alle 5 Batches müssen sicher im WAL landen"
        );

        for (i, (_seq, entry, _pos)) in replayed.iter().enumerate() {
            assert_eq!(entry.seq_no, (i + 1) as u64);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_wal_flusher_actor_no_write_to_sealed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let wal_path = dir.path().join("test_flusher_sealed.wal");

        let wal = Wal::open(&wal_path).await.expect("open WAL");
        let op1 = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        };
        let (batch1, _) = wal
            .prepare_batch(vec![(op1, 1)])
            .await
            .expect("prepare batch 1");
        wal.append_batch(batch1).await.expect("append batch 1");

        assert!(
            !wal.is_sealed(),
            "WAL must not be sealed before rotate_and_seal"
        );

        let sealed_path = wal.rotate_and_seal().await.expect("rotate_and_seal");
        assert!(wal.is_sealed(), "WAL must be sealed after rotate_and_seal");

        // Attempting to prepare or append to the sealed WAL segment must return Err
        let op2 = WalOp::Put {
            tx_id: TxId::new(2),
            key: b"k2".to_vec(),
            value: b"v2".to_vec(),
        };
        let prep_res = wal.prepare_batch(vec![(op2, 2)]).await;
        assert!(
            prep_res.is_err(),
            "prepare_batch on sealed WAL must return Err, no panic or silent write"
        );

        let dummy_entry = WalEntry::try_new(
            WalOp::Put {
                tx_id: TxId::new(3),
                key: b"k3".to_vec(),
                value: b"v3".to_vec(),
            },
            3,
            &[0u8; 32],
            [0u8; 32],
        )
        .expect("dummy entry");
        let manual_batch = PreparedBatch(vec![dummy_entry]);
        let append_res = wal.append_batch(manual_batch).await;
        assert!(
            append_res.is_err(),
            "append_batch on sealed WAL must return Err, no panic or silent write"
        );

        assert!(sealed_path.exists(), "Sealed path must exist");
    }
}
