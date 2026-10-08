//! FILE-CONTEXT: STAND/ZWECK/INVARIANTEN
//! Stand: 2026-10-06
//! Zweck: Background flusher actor processing WAL I/O commands sequentially with error truncation.
//! Invarianten:
//! - I-2 (Atomarer Group-Commit): Ungefluste Data-Bytes werden bei sync_all-Fehler sofort per set_len(size_before) zurückgeschnitten.
//! - I-3 (Crash-Konsistenz): Nach fsync-Fehler verbleiben keine unbestätigten Bytes im Page-Cache.
//! - I-5 (Poison-State Isolation): Nach I/O- oder fsync-Fehler bleibt das Handle poisoned bis zur expliziten Replay-Recovery.

use contextra_core::{ContextraError, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use super::{Wal, WalEntry, WalVersion, WAL_V3_HEADER};

#[allow(dead_code)]
static WAL_SEAL_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[cfg(feature = "fault-injection")]
pub static FAIL_SYNC_ONCE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[allow(dead_code)]
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
            Self::Append {
                payload,
                last_hmac_val,
                ..
            } => f
                .debug_struct("Append")
                .field("payload_len", &payload.len())
                .field("last_hmac_val", last_hmac_val)
                .finish(),
            Self::Truncate {
                offset,
                new_last_hmac,
                ..
            } => f
                .debug_struct("Truncate")
                .field("offset", offset)
                .field("new_last_hmac", new_last_hmac)
                .finish(),
            Self::Seal { .. } => f.debug_struct("Seal").finish(),
            Self::Rewrite {
                replayed_entries, ..
            } => f
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
/// into a single `sync_all()` call to reduce fsync overhead under load,
/// and sets bounded backpressure queue capacity.
#[derive(Debug, Clone, Copy)]
pub struct WalFlusherConfig {
    /// Maximum time to wait for additional messages after receiving the first,
    /// before issuing `sync_all()`. Set to 0 to disable (immediate flush).
    ///
    /// Typical range: 50–500 µs. Higher values coalesce more writes per fsync
    /// at the cost of added tail latency. Default: 100 µs.
    pub batch_window_micros: u64,

    /// Queue capacity for the bounded flusher actor command channel.
    /// Default: 1_024. Must be > 0.
    pub queue_capacity: usize,
}

impl Default for WalFlusherConfig {
    fn default() -> Self {
        Self {
            batch_window_micros: 100,
            queue_capacity: super::DEFAULT_WAL_QUEUE_CAPACITY,
        }
    }
}

#[allow(dead_code)]
fn encode_encrypted_rewrite_chunks(
    v3_entries: &[WalEntry],
    km: &crate::wal::KeyManager,
) -> Result<Vec<u8>> {
    let mut total_bytes = Vec::new();
    let mut current_batch_plaintext = Vec::new();

    const OVERHEAD: usize = 12 + 16;
    let max_frame_size = super::MAX_WAL_ENTRY_SIZE as usize;

    for entry in v3_entries {
        let entry_bytes = entry.to_bytes()?;
        if OVERHEAD + entry_bytes.len() > max_frame_size {
            return Err(ContextraError::Storage(format!(
                "Single WAL entry size {} exceeds MAX_WAL_ENTRY_SIZE frame limit {}",
                entry_bytes.len(),
                max_frame_size
            )));
        }

        if OVERHEAD + current_batch_plaintext.len() + entry_bytes.len() > max_frame_size {
            let (encrypted, nonce) = km.encrypt_auto_nonce(&current_batch_plaintext)?;
            let chunk_len = u32::try_from(12 + encrypted.len())
                .map_err(|e| ContextraError::Storage(e.to_string()))?;

            total_bytes.extend_from_slice(&chunk_len.to_le_bytes());
            total_bytes.extend_from_slice(&nonce);
            total_bytes.extend_from_slice(&encrypted);

            current_batch_plaintext.clear();
        }

        current_batch_plaintext.extend_from_slice(&entry_bytes);
    }

    if !current_batch_plaintext.is_empty() {
        let (encrypted, nonce) = km.encrypt_auto_nonce(&current_batch_plaintext)?;
        let chunk_len = u32::try_from(12 + encrypted.len())
            .map_err(|e| ContextraError::Storage(e.to_string()))?;

        total_bytes.extend_from_slice(&chunk_len.to_le_bytes());
        total_bytes.extend_from_slice(&nonce);
        total_bytes.extend_from_slice(&encrypted);
    }

    Ok(total_bytes)
}

impl Wal {
    /// Enables background flusher actor for processing WAL I/O commands sequentially.
    #[allow(dead_code)]
    pub(crate) fn enable_flusher_with_config(
        &self,
        mut file: crate::wal::fs::File,
        config: WalFlusherConfig,
    ) -> Result<()> {
        if config.queue_capacity == 0 {
            return Err(ContextraError::invalid_input(
                "WAL queue_capacity must be greater than 0",
            ));
        }

        let mut tx_guard = match self.flusher_tx.write() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        if tx_guard.is_some() {
            return Ok(());
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel::<WalCommand>(config.queue_capacity);
        let path = self.path.clone();
        let header_written = Arc::clone(&self.header_written);
        let size = Arc::clone(&self.size);
        let last_hmac = Arc::clone(&self.last_hmac);
        let poisoned = Arc::clone(&self.poisoned);
        let key_manager = self.key_manager.clone();
        let fallback_integrity_key = self.fallback_integrity_key;
        let allow_legacy_integrity_key_fallback =
            Arc::clone(&self.allow_legacy_integrity_key_fallback);
        let min_wal_version = self.min_wal_version;
        let legacy_key_used = Arc::clone(&self.legacy_key_used);

        let handle = tokio::spawn(async move {
            let mut pending_cmd: Option<WalCommand> = None;
            let mut flusher_last_hmac = *last_hmac.lock().await;

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
                        if poisoned.load(std::sync::atomic::Ordering::SeqCst) {
                            let _ = ack.send(Err(ContextraError::Storage(
                                "WAL handle poisoned after suspected torn write; requires explicit recovery replay before further appends".into(),
                            )));
                            while let Ok(next_cmd) = rx.try_recv() {
                                if let WalCommand::Append { ack, .. } = next_cmd {
                                    let _ = ack.send(Err(ContextraError::Storage(
                                        "WAL handle poisoned after suspected torn write; requires explicit recovery replay before further appends".into(),
                                    )));
                                }
                            }
                            continue;
                        }

                        let mut batch_payload = payload;
                        let mut final_last_hmac_val = last_hmac_val;
                        let mut acks = vec![ack];

                        while let Ok(next_cmd) = rx.try_recv() {
                            match next_cmd {
                                WalCommand::Append {
                                    payload: p,
                                    last_hmac_val: h,
                                    ack: a,
                                } => {
                                    batch_payload.extend_from_slice(&p);
                                    final_last_hmac_val = h;
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
                                        last_hmac_val: h,
                                        ack: a,
                                    })) => {
                                        batch_payload.extend_from_slice(&p);
                                        final_last_hmac_val = h;
                                        acks.push(a);
                                        while let Ok(next_cmd) = rx.try_recv() {
                                            match next_cmd {
                                                WalCommand::Append {
                                                    payload: p,
                                                    last_hmac_val: h,
                                                    ack: a,
                                                } => {
                                                    batch_payload.extend_from_slice(&p);
                                                    final_last_hmac_val = h;
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

                        let size_before = size.load(std::sync::atomic::Ordering::Acquire);
                        let write_header = !header_written
                            .load(std::sync::atomic::Ordering::Acquire)
                            && size_before == 0;

                        let write_res: Result<()> = async {
                            if write_header {
                                file.write_all(&WAL_V3_HEADER).await.map_err(|e| {
                                    ContextraError::Storage(format!(
                                        "WAL flusher header write failed for {}: {}",
                                        path.display(),
                                        e
                                    ))
                                })?;
                            }

                            #[cfg(feature = "fault-injection")]
                            if crate::wal::FAIL_APPEND_PARTIAL_ONCE
                                .compare_exchange(
                                    true,
                                    false,
                                    std::sync::atomic::Ordering::SeqCst,
                                    std::sync::atomic::Ordering::SeqCst,
                                )
                                .is_ok()
                            {
                                let partial_bytes = crate::wal::FAIL_APPEND_AFTER_PARTIAL_BYTES
                                    .swap(0, std::sync::atomic::Ordering::SeqCst);
                                let cut = (partial_bytes as usize).min(batch_payload.len());
                                if cut > 0 {
                                    if let Err(err) = file.write_all(&batch_payload[..cut]).await {
                                        tracing::warn!(
                                            "Failed write_all during simulated WAL append partial write: {}",
                                            err
                                        );
                                    }
                                    if let Err(err) = file.flush().await {
                                        tracing::warn!(
                                            "Failed flush during simulated WAL append partial write: {}",
                                            err
                                        );
                                    }
                                }
                                crate::wal::FAIL_APPEND_AFTER_PARTIAL_BYTES
                                    .store(0, std::sync::atomic::Ordering::SeqCst);
                                crate::wal::FAIL_APPEND_PARTIAL_ONCE
                                    .store(false, std::sync::atomic::Ordering::SeqCst);
                                return Err(ContextraError::Storage(
                                    "Simulated WAL append partial write failure (FAIL_APPEND_PARTIAL_ONCE)".into(),
                                ));
                            }

                            file.write_all(&batch_payload).await.map_err(|e| {
                                ContextraError::Storage(format!(
                                    "WAL flusher write failed for {}: {}",
                                    path.display(),
                                    e
                                ))
                            })?;
                            file.flush().await.map_err(|e| {
                                ContextraError::Storage(format!(
                                    "WAL flusher flush failed for {}: {}",
                                    path.display(),
                                    e
                                ))
                            })?;

                            Ok(())
                        }
                        .await;

                        if let Err(write_err) = write_res {
                            if let Err(e) = file.set_len(size_before).await {
                                tracing::warn!(
                                    wal_path = %path.display(),
                                    error = %e,
                                    "Failed to truncate file length to size_before during append write error recovery"
                                );
                            }
                            if let Err(e) = file.seek(std::io::SeekFrom::Start(size_before)).await {
                                tracing::warn!(
                                    wal_path = %path.display(),
                                    error = %e,
                                    "Failed to seek to size_before during append write error recovery"
                                );
                            }
                            poisoned.store(true, std::sync::atomic::Ordering::SeqCst);

                            for ack in acks {
                                let _ =
                                    ack.send(Err(ContextraError::Storage(write_err.to_string())));
                            }
                            continue;
                        }

                        let sync_res: Result<()> = async {
                            #[cfg(feature = "fault-injection")]
                            if FAIL_SYNC_ONCE
                                .compare_exchange(
                                    true,
                                    false,
                                    std::sync::atomic::Ordering::SeqCst,
                                    std::sync::atomic::Ordering::SeqCst,
                                )
                                .is_ok()
                            {
                                return Err(ContextraError::Storage(
                                    "WAL fsync failed, outcome unknown: simulated fsync failure (FAIL_SYNC_ONCE)".into(),
                                ));
                            }

                            file.sync_all().await.map_err(|e| {
                                ContextraError::Storage(format!(
                                    "WAL fsync failed, outcome unknown: {}",
                                    e
                                ))
                            })?;
                            Ok(())
                        }
                        .await;

                        if let Err(sync_err) = sync_res {
                            if let Err(e) = file.set_len(size_before).await {
                                tracing::warn!(
                                    wal_path = %path.display(),
                                    error = %e,
                                    "Failed to truncate file length to size_before during append sync error recovery"
                                );
                            }
                            if let Err(e) = file.seek(std::io::SeekFrom::Start(size_before)).await {
                                tracing::warn!(
                                    wal_path = %path.display(),
                                    error = %e,
                                    "Failed to seek to size_before during append sync error recovery"
                                );
                            }
                            poisoned.store(true, std::sync::atomic::Ordering::SeqCst);
                            for ack in acks {
                                let _ =
                                    ack.send(Err(ContextraError::Storage(sync_err.to_string())));
                            }
                            continue;
                        }

                        if write_header {
                            header_written.store(true, std::sync::atomic::Ordering::Release);
                        }

                        let written_len = (if write_header { WAL_V3_HEADER.len() } else { 0 })
                            + batch_payload.len();
                        let written_len_u64 = u64::try_from(written_len).unwrap_or(u64::MAX);

                        size.fetch_add(written_len_u64, std::sync::atomic::Ordering::SeqCst);
                        flusher_last_hmac = final_last_hmac_val;
                        let mut last_hmac_guard = last_hmac.lock().await;
                        *last_hmac_guard = final_last_hmac_val;

                        for ack in acks {
                            let _ = ack.send(Ok(()));
                        }
                    }

                    WalCommand::Truncate {
                        offset,
                        new_last_hmac,
                        ack,
                    } => {
                        if poisoned.load(std::sync::atomic::Ordering::SeqCst) {
                            let _ = ack.send(Err(ContextraError::Storage(
                                "WAL handle poisoned after suspected torn write; requires explicit recovery replay before further appends".into(),
                            )));
                            continue;
                        }

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
                                return Err(ContextraError::Storage(
                                    "Simulated WAL truncate I/O failure (FAIL_TRUNCATE_ONCE)"
                                        .into(),
                                ));
                            }

                            if let Err(e) = file.set_len(offset).await {
                                return Err(ContextraError::Storage(format!(
                                    "WAL truncate failed for {}: {}",
                                    path.display(),
                                    e
                                )));
                            }

                            if let Err(e) = file.sync_all().await {
                                poisoned.store(true, std::sync::atomic::Ordering::SeqCst);
                                return Err(ContextraError::Storage(format!(
                                    "WAL truncate fsync failed for {}: {}",
                                    path.display(),
                                    e
                                )));
                            }

                            if let Err(e) = file.seek(std::io::SeekFrom::Start(offset)).await {
                                poisoned.store(true, std::sync::atomic::Ordering::SeqCst);
                                return Err(ContextraError::Storage(format!(
                                    "WAL seek after truncate failed for {}: {}",
                                    path.display(),
                                    e
                                )));
                            }
                            if offset < 4 {
                                header_written.store(false, std::sync::atomic::Ordering::Release);
                            } else {
                                header_written.store(true, std::sync::atomic::Ordering::Release);
                            }

                            flusher_last_hmac = new_last_hmac;
                            let mut last_hmac_guard = last_hmac.lock().await;
                            *last_hmac_guard = new_last_hmac;

                            size.store(offset, std::sync::atomic::Ordering::SeqCst);

                            Ok(())
                        }
                        .await;

                        let _ = ack.send(res);
                    }

                    WalCommand::Seal { ack } => {
                        let res: Result<PathBuf> = async {
                            #[cfg(feature = "fault-injection")]
                            if FAIL_SYNC_ONCE
                                .compare_exchange(
                                    true,
                                    false,
                                    std::sync::atomic::Ordering::SeqCst,
                                    std::sync::atomic::Ordering::SeqCst,
                                )
                                .is_ok()
                            {
                                poisoned.store(true, std::sync::atomic::Ordering::SeqCst);
                                return Err(ContextraError::Storage(
                                    "WAL fsync failed, outcome unknown: simulated fsync failure in seal".into(),
                                ));
                            }

                            file.sync_all().await.map_err(|e| {
                                poisoned.store(true, std::sync::atomic::Ordering::SeqCst);
                                ContextraError::Storage(format!(
                                    "WAL fsync failed, outcome unknown: vor rotate_and_seal fehlgeschlagen: {}",
                                    e
                                ))
                            })?;

                            let mut seq = WAL_SEAL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            let sealed_path = loop {
                                let sealed_name = format!(
                                    "{}.sealed.{}",
                                    path.file_name().and_then(|n| n.to_str()).unwrap_or("wal"),
                                    seq
                                );
                                let candidate_path = path.with_file_name(sealed_name);

                                match crate::wal::fs::try_exists(&candidate_path).await {
                                    Ok(true) => {
                                        seq = WAL_SEAL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    }
                                    Ok(false) => break candidate_path,
                                    Err(e) => {
                                        return Err(ContextraError::Storage(format!(
                                            "WAL try_exists check failed during rotate_and_seal for {}: {}",
                                            candidate_path.display(),
                                            e
                                        )));
                                    }
                                }
                            };

                            crate::wal::fs::rename(&path, &sealed_path)
                                .await
                                .map_err(|e| {
                                    ContextraError::Storage(format!(
                                        "WAL rotate_and_seal rename {} → {} fehlgeschlagen: {}",
                                        path.display(),
                                        sealed_path.display(),
                                        e
                                    ))
                                })?;

                            crate::util::fsync_parent_dir(&sealed_path).await?;

                            let mut perms = crate::wal::fs::metadata(&sealed_path)
                                .await
                                .map_err(|e| {
                                    ContextraError::Storage(format!(
                                        "WAL metadata nach seal fehlgeschlagen: {}",
                                        e
                                    ))
                                })?
                                .permissions();
                            perms.set_readonly(true);
                            crate::wal::fs::set_permissions(&sealed_path, perms)
                                .await
                                .map_err(|e| {
                                    ContextraError::Storage(format!(
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

                            let mut total_bytes = Vec::new();
                            total_bytes.extend_from_slice(&WAL_V3_HEADER);

                            let last_hmac_val = if let Some(last_entry) = v3_entries.last() {
                                last_entry.checksum
                            } else {
                                [0u8; 32]
                            };

                            if let Some(km) = &key_manager {
                                let chunk_bytes = encode_encrypted_rewrite_chunks(&v3_entries, km)?;
                                total_bytes.extend_from_slice(&chunk_bytes);
                            } else {
                                for entry in &v3_entries {
                                    let bytes = entry.to_bytes()?;
                                    total_bytes.extend_from_slice(&bytes);
                                }
                            }

                            let write_res: Result<()> = async {
                                file.seek(std::io::SeekFrom::Start(0)).await.map_err(|e| {
                                    ContextraError::Storage(format!(
                                        "WAL seek failed during migration for {}: {}",
                                        path.display(),
                                        e
                                    ))
                                })?;
                                file.set_len(0).await.map_err(|e| {
                                    ContextraError::Storage(format!(
                                        "WAL truncate failed during migration for {}: {}",
                                        path.display(),
                                        e
                                    ))
                                })?;

                                file.write_all(&total_bytes).await.map_err(|e| {
                                    ContextraError::Storage(format!(
                                        "WAL migration write failed for {}: {}",
                                        path.display(),
                                        e
                                    ))
                                })?;
                                file.flush().await.map_err(|e| {
                                    ContextraError::Storage(format!(
                                        "WAL migration flush failed for {}: {}",
                                        path.display(),
                                        e
                                    ))
                                })?;

                                #[cfg(feature = "fault-injection")]
                                if FAIL_SYNC_ONCE
                                    .compare_exchange(
                                        true,
                                        false,
                                        std::sync::atomic::Ordering::SeqCst,
                                        std::sync::atomic::Ordering::SeqCst,
                                    )
                                    .is_ok()
                                {
                                    return Err(ContextraError::Storage(
                                        "WAL fsync failed, outcome unknown: simulated fsync failure in migration".into(),
                                    ));
                                }

                                file.sync_all().await.map_err(|e| {
                                    ContextraError::Storage(format!(
                                        "WAL migration fsync failed for {}: {}",
                                        path.display(),
                                        e
                                    ))
                                })?;

                                Ok(())
                            }
                            .await;

                            if write_res.is_err() {
                                poisoned.store(true, std::sync::atomic::Ordering::SeqCst);
                                return write_res;
                            }

                            let total_len_u64 = u64::try_from(total_bytes.len())
                                .map_err(|e| ContextraError::Storage(e.to_string()))?;
                            size.store(total_len_u64, std::sync::atomic::Ordering::SeqCst);
                            header_written.store(true, std::sync::atomic::Ordering::Release);
                            flusher_last_hmac = last_hmac_val;
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
                            allow_legacy_integrity_key_fallback
                                .load(std::sync::atomic::Ordering::SeqCst),
                            min_wal_version,
                            &legacy_key_used,
                            |seq, entry, pos| item_tx.send((seq, entry, pos)).is_ok(),
                        )
                        .await;

                        let _ = file.seek(std::io::SeekFrom::End(0)).await; // INTENTIONAL-DROP

                        let _ = ack.send(res);
                    }
                }
            }
        });

        *tx_guard = Some(tx);
        if let Ok(mut guard) = self.flusher_task.lock() {
            *guard = Some(handle);
        }
        Ok(())
    }
}
