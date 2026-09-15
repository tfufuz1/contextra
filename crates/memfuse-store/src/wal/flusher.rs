use memfuse_core::{MemFuseError, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

use super::{Wal, WalEntry, WalVersion, WAL_V3_HEADER};

pub(crate) enum WalCommand {
    Append {
        payload: Vec<u8>,
        last_hmac_val: [u8; 32],
        ack: tokio::sync::oneshot::Sender<Result<()>>,
    },
    Truncate {
        offset: u64,
        new_last_hmac: [u8; 32],
        ack: tokio::sync::oneshot::Sender<Result<()>>,
    },
    Seal {
        ack: tokio::sync::oneshot::Sender<Result<PathBuf>>,
    },
    Rewrite {
        replayed_entries: Vec<(u64, WalEntry, u64)>,
        integrity_key: [u8; 32],
        ack: tokio::sync::oneshot::Sender<Result<()>>,
    },
    Scan {
        file_size: u64,
        item_tx: tokio::sync::mpsc::UnboundedSender<(u64, WalEntry, u64)>,
        ack: tokio::sync::oneshot::Sender<Result<WalVersion>>,
    },
}

impl std::fmt::Debug for WalCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Append { payload, last_hmac_val, .. } => f
                .debug_struct("Append")
                .field("payload_len", &payload.len())
                .field("last_hmac_val", last_hmac_val)
                .finish(),
            Self::Truncate { offset, new_last_hmac, .. } => f
                .debug_struct("Truncate")
                .field("offset", offset)
                .field("new_last_hmac", new_last_hmac)
                .finish(),
            Self::Seal { .. } => f.debug_struct("Seal").finish(),
            Self::Rewrite { replayed_entries, .. } => f
                .debug_struct("Rewrite")
                .field("replayed_entries_len", &replayed_entries.len())
                .finish(),
            Self::Scan { file_size, .. } => f
                .debug_struct("Scan")
                .field("file_size", file_size)
                .finish(),
        }
    }
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

use tokio::io::AsyncSeekExt;

impl Wal {
    /// Enables background flusher actor for processing WAL I/O commands sequentially.
    pub(crate) fn enable_flusher_with_config(&self, mut file: tokio::fs::File, config: WalFlusherConfig) {
        let mut tx_guard = self.flusher_tx.write().unwrap_or_else(|e| e.into_inner());
        if tx_guard.is_some() {
            return;
        }

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WalCommand>();
        let path = self.path.clone();
        let header_written = Arc::clone(&self.header_written);
        let size = Arc::clone(&self.size);
        let last_hmac = Arc::clone(&self.last_hmac);
        let simulate_append_failure = Arc::clone(&self.simulate_append_failure);
        let key_manager = self.key_manager.clone();
        let fallback_integrity_key = self.fallback_integrity_key;
        let allow_legacy_integrity_key_fallback = self.allow_legacy_integrity_key_fallback;

        tokio::spawn(async move {
            let mut pending_cmd: Option<WalCommand> = None;

            loop {
                let cmd = match pending_cmd.take() {
                    Some(c) => c,
                    None => match rx.recv().await {
                        Some(c) => c,
                        None => break,
                    },
                };

                match cmd {
                    WalCommand::Append {
                        payload,
                        last_hmac_val,
                        ack,
                    } => {
                        let mut batch_payload = payload;
                        let mut acks = vec![ack];
                        let mut final_last_hmac_val = last_hmac_val;

                        while let Ok(next_cmd) = rx.try_recv() {
                            match next_cmd {
                                WalCommand::Append {
                                    payload: p,
                                    last_hmac_val: l,
                                    ack: a,
                                } => {
                                    batch_payload.extend_from_slice(&p);
                                    final_last_hmac_val = l;
                                    acks.push(a);
                                }
                                other => {
                                    pending_cmd = Some(other);
                                    break;
                                }
                            }
                        }

                        if pending_cmd.is_none() && config.batch_window_micros > 0 {
                            let deadline = tokio::time::Instant::now()
                                + tokio::time::Duration::from_micros(config.batch_window_micros);
                            loop {
                                match tokio::time::timeout_at(deadline, rx.recv()).await {
                                    Ok(Some(WalCommand::Append {
                                        payload: p,
                                        last_hmac_val: l,
                                        ack: a,
                                    })) => {
                                        batch_payload.extend_from_slice(&p);
                                        final_last_hmac_val = l;
                                        acks.push(a);
                                        while let Ok(next_cmd) = rx.try_recv() {
                                            match next_cmd {
                                                WalCommand::Append {
                                                    payload: p,
                                                    last_hmac_val: l,
                                                    ack: a,
                                                } => {
                                                    batch_payload.extend_from_slice(&p);
                                                    final_last_hmac_val = l;
                                                    acks.push(a);
                                                }
                                                other => {
                                                    pending_cmd = Some(other);
                                                    break;
                                                }
                                            }
                                        }
                                        if pending_cmd.is_some() {
                                            break;
                                        }
                                    }
                                    Ok(Some(other)) => {
                                        pending_cmd = Some(other);
                                        break;
                                    }
                                    Ok(None) => break,
                                    Err(_elapsed) => break,
                                }
                            }
                        }

                        let res: Result<()> = async {
                            if simulate_append_failure
                                .swap(false, std::sync::atomic::Ordering::SeqCst)
                            {
                                return Err(MemFuseError::Storage(
                                    "Simulated WAL append_batch I/O failure".into(),
                                ));
                            }

                            let write_header = !header_written
                                .load(std::sync::atomic::Ordering::Acquire)
                                && size.load(std::sync::atomic::Ordering::Acquire) == 0;

                            if write_header {
                                file.write_all(&WAL_V3_HEADER).await.map_err(|e| {
                                    MemFuseError::Storage(format!(
                                        "WAL flusher header write failed for {}: {}",
                                        path.display(),
                                        e
                                    ))
                                })?;
                            }
                            file.write_all(&batch_payload).await.map_err(|e| {
                                MemFuseError::Storage(format!(
                                    "WAL flusher write failed for {}: {}",
                                    path.display(),
                                    e
                                ))
                            })?;
                            file.flush().await.map_err(|e| {
                                MemFuseError::Storage(format!(
                                    "WAL flusher flush failed for {}: {}",
                                    path.display(),
                                    e
                                ))
                            })?;
                            file.sync_all().await.map_err(|e| {
                                MemFuseError::Storage(format!(
                                    "WAL flusher fsync failed for {}: {}",
                                    path.display(),
                                    e
                                ))
                            })?;

                            if write_header {
                                header_written.store(true, std::sync::atomic::Ordering::Release);
                            }

                            let written_len = (if write_header { WAL_V3_HEADER.len() } else { 0 })
                                + batch_payload.len();
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

                    WalCommand::Truncate {
                        offset,
                        new_last_hmac,
                        ack,
                    } => {
                        let res: Result<()> = async {
                            #[cfg(feature = "fault-injection")]
                            if crate::wal::FAIL_TRUNCATE_ONCE
                                .compare_exchange(
                                    true,
                                    false,
                                    std::sync::atomic::Ordering::SeqCst,
                                    std::sync::atomic::Ordering::SeqCst,
                                )
                                .is_ok()
                            {
                                return Err(MemFuseError::Storage(
                                    "Simulated WAL truncate I/O failure (FAIL_TRUNCATE_ONCE)".into(),
                                ));
                            }

                            file.set_len(offset).await.map_err(|e| {
                                MemFuseError::Storage(format!("WAL truncate failed: {e}"))
                            })?;

                            size.store(offset, std::sync::atomic::Ordering::SeqCst);
                            if offset < 4 {
                                header_written.store(false, std::sync::atomic::Ordering::Release);
                            }

                            file.sync_all().await.map_err(|e| {
                                MemFuseError::Storage(format!("WAL truncate fsync failed: {e}"))
                            })?;

                            file.seek(std::io::SeekFrom::Start(offset))
                                .await
                                .map_err(|e| {
                                    MemFuseError::Storage(format!(
                                        "WAL seek after truncate failed: {e}"
                                    ))
                                })?;

                            let mut last_hmac_guard = last_hmac.lock().await;
                            *last_hmac_guard = new_last_hmac;

                            Ok(())
                        }
                        .await;

                        let _ = ack.send(res);
                    }

                    WalCommand::Seal { ack } => {
                        let res: Result<PathBuf> = async {
                            file.sync_all().await.map_err(|e| {
                                MemFuseError::Storage(format!(
                                    "WAL fsync vor rotate_and_seal fehlgeschlagen: {}",
                                    e
                                ))
                            })?;

                            let micros = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_micros();
                            let sealed_name = format!(
                                "{}.sealed.{}",
                                path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("wal"),
                                micros
                            );
                            let sealed_path = path.with_file_name(sealed_name);

                            tokio::fs::rename(&path, &sealed_path).await.map_err(|e| {
                                MemFuseError::Storage(format!(
                                    "WAL rotate_and_seal rename {} → {} fehlgeschlagen: {}",
                                    path.display(),
                                    sealed_path.display(),
                                    e
                                ))
                            })?;

                            crate::util::fsync_parent_dir(&sealed_path).await?;

                            let mut perms = tokio::fs::metadata(&sealed_path)
                                .await
                                .map_err(|e| {
                                    MemFuseError::Storage(format!(
                                        "WAL metadata nach seal fehlgeschlagen: {}",
                                        e
                                    ))
                                })?
                                .permissions();
                            perms.set_readonly(true);
                            tokio::fs::set_permissions(&sealed_path, perms)
                                .await
                                .map_err(|e| {
                                    MemFuseError::Storage(format!(
                                        "WAL set_readonly fehlgeschlagen: {}",
                                        e
                                    ))
                                })?;

                            Ok(sealed_path)
                        }
                        .await;

                        let _ = ack.send(res);
                    }

                    WalCommand::Rewrite {
                        replayed_entries,
                        integrity_key,
                        ack,
                    } => {
                        let res: Result<()> = async {
                            let mut v3_entries = Vec::with_capacity(replayed_entries.len());
                            let mut prev_hmac = [0u8; 32];

                            for (_, entry, _) in &replayed_entries {
                                let v3_entry = WalEntry::try_new(
                                    entry.op.clone(),
                                    entry.seq_no,
                                    &integrity_key,
                                    prev_hmac,
                                )?;
                                prev_hmac = v3_entry.checksum;
                                v3_entries.push(v3_entry);
                            }

                            file.seek(std::io::SeekFrom::Start(0)).await.map_err(|e| {
                                MemFuseError::Storage(format!(
                                    "WAL seek failed during migration: {}",
                                    e
                                ))
                            })?;
                            file.set_len(0).await.map_err(|e| {
                                MemFuseError::Storage(format!(
                                    "WAL truncate failed during migration: {}",
                                    e
                                ))
                            })?;

                            let mut total_bytes = Vec::new();
                            total_bytes.extend_from_slice(&WAL_V3_HEADER);

                            let mut last_hmac_val = [0u8; 32];
                            if let Some(km) = &key_manager {
                                let mut batch_plaintext = Vec::new();
                                for entry in &v3_entries {
                                    let bytes = entry.to_bytes()?;
                                    batch_plaintext.extend_from_slice(&bytes);
                                    last_hmac_val = entry.checksum;
                                }

                                let (encrypted, nonce) = km.encrypt_auto_nonce(&batch_plaintext)?;
                                let chunk_len = (12 + encrypted.len()) as u32;

                                total_bytes.extend_from_slice(&chunk_len.to_le_bytes());
                                total_bytes.extend_from_slice(&nonce);
                                total_bytes.extend_from_slice(&encrypted);
                            } else {
                                for entry in &v3_entries {
                                    let bytes = entry.to_bytes()?;
                                    total_bytes.extend_from_slice(&bytes);
                                    last_hmac_val = entry.checksum;
                                }
                            }

                            file.write_all(&total_bytes).await.map_err(|e| {
                                MemFuseError::Storage(format!(
                                    "WAL migration write failed: {}",
                                    e
                                ))
                            })?;
                            file.flush().await.map_err(|e| {
                                MemFuseError::Storage(format!(
                                    "WAL migration flush failed: {}",
                                    e
                                ))
                            })?;
                            file.sync_all().await.map_err(|e| {
                                MemFuseError::Storage(format!(
                                    "WAL migration fsync failed: {}",
                                    e
                                ))
                            })?;

                            size.store(
                                total_bytes.len() as u64,
                                std::sync::atomic::Ordering::SeqCst,
                            );
                            header_written.store(true, std::sync::atomic::Ordering::Release);
                            let mut last_hmac_guard = last_hmac.lock().await;
                            *last_hmac_guard = last_hmac_val;

                            Ok(())
                        }
                        .await;

                        let _ = ack.send(res);
                    }

                    WalCommand::Scan {
                        file_size,
                        item_tx,
                        ack,
                    } => {
                        let res = crate::wal::io::do_scan_entries_with_callback(
                            &mut file,
                            file_size,
                            &path,
                            key_manager.as_deref(),
                            fallback_integrity_key,
                            allow_legacy_integrity_key_fallback,
                            |seq, entry, pos| item_tx.send((seq, entry, pos)).is_ok(),
                        )
                        .await;

                        let _ = ack.send(res);
                    }
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
