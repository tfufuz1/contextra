//! FILE-CONTEXT:
//! STAND: 2026-10-06
//! ZWECK: Synchrones/asynchrones Stream-Reading, Scans, Batch-Appending und File-I/O für das WAL.
//! INVARIANTEN:
//! - I-1: HMAC-Verkettung muss intakt sein. Nach verworfenem Tail dürfen keine weiteren Frames verarbeitet werden.
//! - I-3: End-of-File Partial Entries werden als Tail Truncation behandelt.

use contextra_types::{ContextraError, Result, TxId};
#[cfg(feature = "wal-integrity")]
use contextra_crypto::wal_crypto::{IntegrityVerifier, WalEntrySnapshot};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

#[cfg(feature = "wal-integrity")]
use super::legacy_integrity_key;
use super::{
    PreparedBatch, Wal, WalCommand, WalEntry, WalOp, WalVersion, MAX_WAL_ENTRY_SIZE, WAL_V2_HEADER,
    WAL_V3_HEADER,
};


#[cfg(feature = "wal-integrity")]
struct WalVerifierCtx<'a> {
    path: &'a Path,
    verifier: IntegrityVerifier,
    using_legacy_key: bool,
    allow_legacy_fallback: bool,
    legacy_key_used: &'a std::sync::atomic::AtomicBool,
}

#[cfg(feature = "wal-integrity")]
impl<'a> WalVerifierCtx<'a> {
    fn new(
        path: &'a Path,
        key_manager: Option<&super::KeyManager>,
        fallback_integrity_key: Option<[u8; 32]>,
        allow_legacy_fallback: bool,
        legacy_key_used: &'a std::sync::atomic::AtomicBool,
    ) -> Result<Self> {
        let integrity_key = if let Some(km) = key_manager {
            km.integrity_key()?
        } else if let Some(key) = fallback_integrity_key {
            key
        } else {
            return Err(ContextraError::Storage("No integrity key available".into()));
        };
        let verifier = IntegrityVerifier::new(&integrity_key);
        Ok(Self {
            path,
            verifier,
            using_legacy_key: false,
            allow_legacy_fallback,
            legacy_key_used,
        })
    }

    #[allow(deprecated)]
    fn verify(&mut self, entry: &WalEntry, version: WalVersion, chunk_start_pos: u64) -> Result<()> {
        let (op_type, key, value) = match &entry.op {
            WalOp::Put { key, value, .. } => (0u8, key.clone(), value.clone()),
            WalOp::Delete { key, .. } => (1u8, key.clone(), Vec::new()),
            WalOp::TxEnd { committed, .. } => (2u8, Vec::new(), vec![*committed as u8]),
        };
        let snapshot = WalEntrySnapshot {
            tx_id: entry.tx_id().inner(),
            seq_no: entry.seq_no,
            op_type,
            key,
            value,
            checksum: entry.checksum,
            prev_hmac: entry.prev_hmac,
        };

        let verify_res = match version {
            WalVersion::V3 => self.verifier.verify_and_update_v3(&snapshot, chunk_start_pos),
            WalVersion::V2 => self.verifier.verify_and_update_v2(&snapshot, chunk_start_pos),
            WalVersion::V1 => {
                self.verifier.skip_hmac_verify_legacy(&snapshot);
                Ok(())
            }
        };

        if let Err(e) = verify_res {
            if !self.using_legacy_key && self.allow_legacy_fallback {
                let legacy_key = legacy_integrity_key()?;
                let mut legacy_verifier = IntegrityVerifier::new(&legacy_key);
                legacy_verifier.set_last_hmac(self.verifier.last_hmac_snapshot());
                let legacy_res = match version {
                    WalVersion::V3 => legacy_verifier.verify_and_update_v3(&snapshot, chunk_start_pos),
                    WalVersion::V2 => legacy_verifier.verify_and_update_v2(&snapshot, chunk_start_pos),
                    WalVersion::V1 => {
                        legacy_verifier.skip_hmac_verify_legacy(&snapshot);
                        Ok(())
                    }
                };
                if legacy_res.is_ok() {
                    tracing::warn!(
                        target: "contextra_store::wal::legacy_fallback",
                        wal_path = %self.path.display(),
                        event = "legacy_integrity_key_fallback_used",
                        "Legacy-WAL-Integritätsschlüssel aktiv für Segment {} — dieses Segment hat keine reale Manipulationssicherheit, da der Rückfallschlüssel öffentlich im Quellcode liegt.",
                        self.path.display()
                    );
                    self.verifier = legacy_verifier;
                    self.using_legacy_key = true;
                    self.legacy_key_used.store(true, std::sync::atomic::Ordering::SeqCst);
                    Ok(())
                } else {
                    Err(e.into())
                }
            } else {
                Err(e.into())
            }
        } else {
            Ok(())
        }
    }
}

async fn detect_wal_header(
    reader: &mut tokio::io::BufReader<&mut super::fs::File>,
    file_size: u64,
) -> Result<(WalVersion, u64)> {
    if file_size >= 4 {
        let mut header_buf = [0u8; 4];
        if reader.read_exact(&mut header_buf).await.is_ok() {
            if header_buf == WAL_V3_HEADER {
                return Ok((WalVersion::V3, 4));
            } else if header_buf == WAL_V2_HEADER {
                return Ok((WalVersion::V2, 4));
            } else {
                reader
                    .seek(std::io::SeekFrom::Start(0))
                    .await
                    .map_err(|e| {
                        ContextraError::Storage(format!("WAL replay seek failed: {}", e))
                    })?;
                return Ok((WalVersion::V1, 0));
            }
        }
    }
    Ok((WalVersion::V1, 0))
}

async fn read_entry_length(
    reader: &mut tokio::io::BufReader<&mut super::fs::File>,
    pos: u64,
    file_size: u64,
    entries_count: u64,
) -> Result<Option<usize>> {
    let mut len_buf = [0u8; 4];
    if let Err(e) = reader.read_exact(&mut len_buf).await {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            tracing::warn!(
                "WAL tail corruption (partial entry length) at offset {}",
                pos
            );
            return Ok(None);
        }
        return Err(ContextraError::Storage(format!("WAL read failed: {}", e)));
    }

    let len = u32::from_le_bytes(len_buf) as usize;
    let len_u64 = u64::try_from(len).unwrap_or(u64::MAX);

    if len > MAX_WAL_ENTRY_SIZE as usize {
        if pos.saturating_add(4).saturating_add(len_u64) > file_size {
            if entries_count == 0 && file_size > 64 {
                return Err(ContextraError::wal_corruption(
                    pos,
                    format!(
                        "WAL entry length ({}) exceeds hard limit and file size",
                        len
                    ),
                ));
            }
            tracing::warn!("WAL tail corruption (huge len) at offset {}", pos);
            return Ok(None);
        }
        return Err(ContextraError::wal_corruption(
            pos,
            format!("WAL entry too large ({} bytes)", len),
        ));
    }

    if pos.saturating_add(4).saturating_add(len_u64) > file_size {
        if entries_count == 0 && file_size > 64 {
            return Err(ContextraError::wal_corruption(
                pos,
                format!(
                    "WAL entry length ({}) exceeds file size ({}) at start of file",
                    len, file_size
                ),
            ));
        }
        tracing::warn!("WAL tail corruption (partial entry) at offset {}", pos);
        return Ok(None);
    }

    Ok(Some(len))
}

fn process_inner_batch<F>(
    decrypted_data: &[u8],
    chunk_start_pos: u64,
    pos: u64,
    #[cfg(feature = "wal-integrity")] version: WalVersion,
    #[cfg(feature = "wal-integrity")] verifier_ctx: &mut WalVerifierCtx<'_>,
    entries_count: &mut u64,
    callback: &mut F,
) -> Result<bool>
where
    F: FnMut(u64, WalEntry, u64) -> bool,
{
    let mut inner_slice = decrypted_data;
    while !inner_slice.is_empty() {
        if inner_slice.len() < 4 {
            return Err(ContextraError::wal_corruption(
                chunk_start_pos,
                "Incomplete inner framing in decrypted batch",
            ));
        }
        let inner_len_bytes: [u8; 4] = match inner_slice.get(0..4).and_then(|s| s.try_into().ok()) {
            Some(b) => b,
            None => {
                return Err(ContextraError::wal_corruption(
                    chunk_start_pos,
                    "Failed to extract inner WAL entry length",
                ));
            }
        };
        let inner_len = usize::try_from(u32::from_le_bytes(inner_len_bytes)).unwrap_or(usize::MAX);
        if inner_slice.len() < 4 + inner_len {
            return Err(ContextraError::wal_corruption(
                chunk_start_pos,
                "Truncated inner WAL entry payload in decrypted batch",
            ));
        }
        let inner_entry_bytes = match inner_slice.get(4..4 + inner_len) {
            Some(b) => b,
            None => {
                return Err(ContextraError::wal_corruption(
                    chunk_start_pos,
                    "Truncated inner WAL entry payload in decrypted batch",
                ));
            }
        };
        inner_slice = inner_slice.get(4 + inner_len..).unwrap_or(&[]);

        let entry = WalEntry::from_bytes_classified(inner_entry_bytes).map_err(|parse_err| {
            ContextraError::wal_corruption(
                chunk_start_pos,
                format!("Deserialization failed in authenticated batch: {parse_err}"),
            )
        })?;

        #[cfg(feature = "wal-integrity")]
        verifier_ctx.verify(&entry, version, chunk_start_pos)?;

        *entries_count += 1;
        let seq = entry.seq_no;
        if !callback(seq, entry, pos) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn scan_single_frame<F>(
    entry_data_raw: &[u8],
    chunk_start_pos: u64,
    pos: u64,
    file_size: u64,
    version: WalVersion,
    path: &Path,
    key_manager: Option<&super::KeyManager>,
    #[cfg(feature = "wal-integrity")] verifier_ctx: &mut WalVerifierCtx<'_>,
    entries_count: &mut u64,
    callback: &mut F,
) -> Result<bool>
where
    F: FnMut(u64, WalEntry, u64) -> bool,
{
    if let (Some(km), WalVersion::V2 | WalVersion::V3) = (key_manager, version) {
        if entry_data_raw.len() < 12 {
            if pos >= file_size {
                tracing::warn!("WAL truncated during read at offset {}", chunk_start_pos);
                return Ok(false);
            }
            return Err(ContextraError::Storage(
                "WAL entry too short for nonce".into(),
            ));
        }
        let nonce_slice = match entry_data_raw.get(0..12) {
            Some(s) => s,
            None => {
                return Err(ContextraError::wal_corruption(
                    chunk_start_pos,
                    "WAL entry too short for nonce",
                ));
            }
        };
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(nonce_slice);
        let ciphertext = entry_data_raw.get(12..).ok_or_else(|| {
            ContextraError::wal_corruption(chunk_start_pos, "WAL entry missing ciphertext")
        })?;

        let decrypted_data = match km.decrypt_auto_nonce(ciphertext, &nonce) {
            Ok(data) => data,
            Err(e) => {
                if pos >= file_size {
                    tracing::error!(
                        "WAL truncation at tail (offset {}), AEAD decryption failed: {}. Tolerated as incomplete tail write.",
                        chunk_start_pos,
                        e
                    );
                    return Ok(false);
                }
                return Err(ContextraError::wal_corruption(
                    chunk_start_pos,
                    format!("Decryption failed: {}", e),
                ));
            }
        };

        process_inner_batch(
            &decrypted_data,
            chunk_start_pos,
            pos,
            #[cfg(feature = "wal-integrity")]
            version,
            #[cfg(feature = "wal-integrity")]
            verifier_ctx,
            entries_count,
            callback,
        )
    } else {
        let decrypted_data;
        let entry_data = if let Some(km) = key_manager {
            if entry_data_raw.len() < 12 {
                return Err(ContextraError::Storage(
                    "WAL entry too short for nonce".into(),
                ));
            }
            let nonce_slice = match entry_data_raw.get(0..12) {
                Some(s) => s,
                None => {
                    return Err(ContextraError::wal_corruption(
                        chunk_start_pos,
                        "WAL entry too short for nonce",
                    ));
                }
            };
            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(nonce_slice);
            let ciphertext = entry_data_raw.get(12..).ok_or_else(|| {
                ContextraError::wal_corruption(
                    chunk_start_pos,
                    "WAL entry missing ciphertext",
                )
            })?;
            decrypted_data = match km.decrypt_auto_nonce(ciphertext, &nonce) {
                Ok(data) => data,
                Err(e) => {
                    if version == WalVersion::V1 {
                        return Err(ContextraError::Storage(format!(
                            "WAL entry at {} claims V1/plaintext format while KeyManager is active for {} \
                             (decryption failed: {}) — refusing potential downgrade attack. \
                             Set allow_legacy_integrity_key_fallback / min_wal_version appropriately if \
                             this WAL genuinely predates encryption and requires migration.",
                            chunk_start_pos,
                            path.display(),
                            e
                        )));
                    } else {
                        if pos >= file_size {
                            tracing::error!(
                                "WAL truncation at tail (offset {}), AEAD decryption failed: {}. Tolerated as incomplete tail write.",
                                chunk_start_pos,
                                e
                            );
                            return Ok(false);
                        }
                        return Err(ContextraError::wal_corruption(
                            chunk_start_pos,
                            format!("Decryption failed: {}", e),
                        ));
                    }
                }
            };
            &decrypted_data
        } else {
            entry_data_raw
        };

        let entry = match WalEntry::from_bytes_classified(entry_data) {
            Ok(e) => e,
            Err(parse_err) => {
                if key_manager.is_some() && version != WalVersion::V1 {
                    return Err(ContextraError::wal_corruption(
                        chunk_start_pos,
                        format!(
                            "Deserialization failed in authenticated entry: {parse_err}"
                        ),
                    ));
                }
                if let Some(err) = Wal::handle_wal_entry_parse_error(
                    parse_err,
                    chunk_start_pos,
                    pos,
                    file_size,
                ) {
                    return Err(err);
                }
                return Ok(false);
            }
        };

        #[cfg(feature = "wal-integrity")]
        verifier_ctx.verify(&entry, version, chunk_start_pos)?;

        *entries_count += 1;
        let seq = entry.seq_no;
        Ok(callback(seq, entry, pos))
    }
}

#[allow(deprecated, clippy::too_many_arguments)]
pub(crate) async fn do_scan_entries_with_callback<F>(
    file: &mut super::fs::File,
    file_size: u64,
    path: &Path,
    key_manager: Option<&super::KeyManager>,
    fallback_integrity_key: Option<[u8; 32]>,
    allow_legacy_integrity_key_fallback: bool,
    min_wal_version: WalVersion,
    legacy_key_used: &std::sync::atomic::AtomicBool,
    mut callback: F,
) -> Result<WalVersion>
where
    F: FnMut(u64, WalEntry, u64) -> bool,
{
    file.seek(std::io::SeekFrom::Start(0))
        .await
        .map_err(|e| ContextraError::Storage(format!("WAL replay seek failed: {}", e)))?;

    if file_size == 0 {
        return Ok(WalVersion::V1);
    }

    let mut reader = tokio::io::BufReader::new(file);
    let (version, mut pos) = detect_wal_header(&mut reader, file_size).await?;

    if version < min_wal_version
        || (version != WalVersion::V3 && !allow_legacy_integrity_key_fallback)
    {
        return Err(ContextraError::invalid_input(format!(
            "WAL format version {:?} is disallowed by configuration (min_wal_version: {:?}, allow_legacy_integrity_key_fallback: {}). Explicit migration via open_for_legacy_migration / migrate_legacy_wal required.",
            version, min_wal_version, allow_legacy_integrity_key_fallback
        )));
    }

    #[cfg(feature = "wal-integrity")]
    let mut verifier_ctx = WalVerifierCtx::new(
        path,
        key_manager,
        fallback_integrity_key,
        allow_legacy_integrity_key_fallback,
        legacy_key_used,
    )?;

    let mut entries_count = 0u64;

    while pos < file_size {
        let len = match read_entry_length(&mut reader, pos, file_size, entries_count).await? {
            Some(l) => l,
            None => break,
        };

        let mut entry_data_raw = vec![0u8; len];
        if let Err(e) = reader.read_exact(&mut entry_data_raw).await {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                tracing::warn!(
                    "WAL tail corruption (truncated entry payload) at offset {}",
                    pos
                );
                break;
            }
            return Err(ContextraError::Storage(format!("WAL read failed: {}", e)));
        }

        let chunk_start_pos = pos;
        let len_u64 = u64::try_from(len).unwrap_or(u64::MAX);
        pos = pos.saturating_add(4).saturating_add(len_u64);

        let continue_scan = scan_single_frame(
            &entry_data_raw,
            chunk_start_pos,
            pos,
            file_size,
            version,
            path,
            key_manager,
            #[cfg(feature = "wal-integrity")]
            &mut verifier_ctx,
            &mut entries_count,
            &mut callback,
        )?;

        if !continue_scan {
            break;
        }
    }

    Ok(version)
}

impl Wal {
    pub(crate) async fn prepare_append_payload(
        &self,
        batch: &PreparedBatch,
    ) -> Result<(Vec<u8>, [u8; 32])> {
        if self.is_sealed() {
            return Err(ContextraError::Storage(format!(
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
            let fail_tx = self.fault_plan.fail_append_for_tx.load(std::sync::atomic::Ordering::SeqCst);
            if fail_tx != 0
                && entries
                    .iter()
                    .any(|e| e.tx_id().inner() == fail_tx || fail_tx == u64::MAX)
            {
                self.fault_plan.fail_append_for_tx.store(0, std::sync::atomic::Ordering::SeqCst);
                return Err(ContextraError::Storage(
                    "Simulated WAL append_batch I/O failure via fault injection".into(),
                ));
            }

            let delay_tx = self.fault_plan.delay_append_for_tx.load(std::sync::atomic::Ordering::SeqCst);
            if delay_tx != 0 && entries.iter().any(|e| e.tx_id().inner() == delay_tx) {
                let delay_ms = self.fault_plan.delay_append_ms.load(std::sync::atomic::Ordering::SeqCst);
                self.fault_plan.delay_append_for_tx.store(0, std::sync::atomic::Ordering::SeqCst);
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
                    return Err(ContextraError::invalid_input(format!(
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
                                ContextraError::invalid_input("Frame length calculation overflow")
                            })?;
                        let chunk_len = u32::try_from(chunk_len_usize).map_err(|_| {
                            ContextraError::invalid_input("Frame length exceeds u32")
                        })?;
                        if chunk_len > MAX_WAL_ENTRY_SIZE {
                            return Err(ContextraError::invalid_input(format!(
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
                                ContextraError::invalid_input("Frame length calculation overflow")
                            })?;
                        let chunk_len = u32::try_from(chunk_len_usize).map_err(|_| {
                            ContextraError::invalid_input("Frame length exceeds u32")
                        })?;
                        if chunk_len > MAX_WAL_ENTRY_SIZE {
                            return Err(ContextraError::invalid_input(format!(
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
                            ContextraError::invalid_input("Frame length calculation overflow")
                        })?;
                    let chunk_len = u32::try_from(chunk_len_usize)
                        .map_err(|_| ContextraError::invalid_input("Frame length exceeds u32"))?;
                    if chunk_len > MAX_WAL_ENTRY_SIZE {
                        return Err(ContextraError::invalid_input(format!(
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
            .map_err(|e| ContextraError::Storage(format!("WAL encryption task panicked: {e}")))??;

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
            .map_err(|_| ContextraError::Storage("WAL flusher dropped".into()))??;
        Ok(())
    }

    pub async fn enqueue_append_batch_locked(
        &self,
        batch: PreparedBatch,
        _guard: &tokio::sync::MutexGuard<'_, ()>,
    ) -> Result<tokio::sync::oneshot::Receiver<Result<()>>> {
        if self.is_poisoned() {
            return Err(ContextraError::Storage(
                "WAL handle poisoned after suspected torn write; requires explicit recovery replay before further appends".into(),
            ));
        }
        let entries = &batch.0;
        if entries.is_empty() {
            let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
            let _ = ack_tx.send(Ok(()));
            return Ok(ack_rx);
        }

        let (payload_bytes, last_hmac_val) = self.prepare_append_payload(&batch).await?;

        let flusher_tx = {
            let guard = match self.flusher_tx.read() {
                Ok(g) => g,
                Err(e) => e.into_inner(),
            };
            guard.clone()
        };

        let tx = flusher_tx
            .ok_or_else(|| ContextraError::Storage("WAL flusher actor is not enabled".into()))?;

        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        tx.send(WalCommand::Append {
            payload: payload_bytes,
            last_hmac_val,
            ack: ack_tx,
        })
        .await
        .map_err(|_| ContextraError::Storage("WAL flusher channel closed".into()))?;

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
            .map_err(|_| ContextraError::Storage("WAL flusher dropped".into()))??;
        Ok(())
    }

    /// Non-blocking attempt to append a prepared batch. Returns `Err(ContextraError::Storage("WAL queue full (backpressure)"))`
    /// if the flusher channel buffer is full.
    pub async fn try_append_batch(&self, batch: PreparedBatch) -> Result<()> {
        let truncate_guard = self.truncate_lock.lock().await;
        self.try_append_batch_locked(batch, &truncate_guard).await
    }

    pub async fn try_enqueue_append_batch_locked(
        &self,
        batch: PreparedBatch,
        _guard: &tokio::sync::MutexGuard<'_, ()>,
    ) -> Result<tokio::sync::oneshot::Receiver<Result<()>>> {
        if self.is_poisoned() {
            return Err(ContextraError::Storage(
                "WAL handle poisoned after suspected torn write; requires explicit recovery replay before further appends".into(),
            ));
        }
        let entries = &batch.0;
        if entries.is_empty() {
            let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
            let _ = ack_tx.send(Ok(()));
            return Ok(ack_rx);
        }

        let (payload_bytes, last_hmac_val) = self.prepare_append_payload(&batch).await?;

        let flusher_tx = {
            let guard = match self.flusher_tx.read() {
                Ok(g) => g,
                Err(e) => e.into_inner(),
            };
            guard.clone()
        };

        let tx = flusher_tx
            .ok_or_else(|| ContextraError::Storage("WAL flusher actor is not enabled".into()))?;

        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        tx.try_send(WalCommand::Append {
            payload: payload_bytes,
            last_hmac_val,
            ack: ack_tx,
        })
        .map_err(|e| match e {
            tokio::sync::mpsc::error::TrySendError::Full(_) => {
                ContextraError::Storage("WAL queue full (backpressure)".into())
            }
            tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                ContextraError::Storage("WAL flusher channel closed".into())
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
            .map_err(|_| ContextraError::Storage("WAL flusher dropped".into()))??;
        Ok(())
    }

    /// Scans the WAL entry by entry, executing full HMAC chain validation, CRC checks, and key manager decryption.
    ///
    /// Invokes `callback(seq_no, entry, end_offset)` for each valid entry.
    /// If `callback` returns `false`, scanning halts early.
    pub async fn scan_entries_with_callback<F>(
        &self,
        file_size: u64,
        mut callback: F,
    ) -> Result<WalVersion>
    where
        F: FnMut(u64, WalEntry, u64) -> bool,
    {
        let flusher_tx = {
            let guard = match self.flusher_tx.read() {
                Ok(g) => g,
                Err(e) => e.into_inner(),
            };
            guard.clone()
        };

        if let Some(tx) = flusher_tx {
            let (item_tx, mut item_rx) = tokio::sync::mpsc::unbounded_channel();
            let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();

            tx.send(WalCommand::Scan {
                file_size,
                item_tx,
                ack: ack_tx,
            })
            .await
            .map_err(|_| ContextraError::Storage("WAL flusher channel closed".into()))?;

            let mut stopped = false;
            while let Some((seq, entry, pos)) = item_rx.recv().await {
                if !stopped && !callback(seq, entry, pos) {
                    stopped = true;
                }
            }

            let version = ack_rx
                .await
                .map_err(|_| ContextraError::Storage("WAL flusher dropped".into()))??;

            Ok(version)
        } else {
            let mut file = super::fs::OpenOptions::new()
                .read(true)
                .open(&self.path)
                .await
                .map_err(|e| {
                    ContextraError::Storage(format!("Failed to open WAL for read_only scan: {e}"))
                })?;

            let key_manager = self.key_manager.as_deref();
            let fallback_integrity_key = self.fallback_integrity_key;
            let allow_legacy_fallback = self
                .allow_legacy_integrity_key_fallback
                .load(std::sync::atomic::Ordering::SeqCst);
            let min_wal_version = self.min_wal_version;

            do_scan_entries_with_callback(
                &mut file,
                file_size,
                &self.path,
                key_manager,
                fallback_integrity_key,
                allow_legacy_fallback,
                min_wal_version,
                &self.legacy_key_used,
                callback,
            )
            .await
        }
    }

    /// Rewrites legacy V1 or V2 WAL files as V3.
    pub async fn rewrite_as_v3(
        &self,
        replayed_entries: &[(u64, WalEntry, u64)],
    ) -> Result<()> {
        let integrity_key = self.get_integrity_key()?;

        let flusher_tx = {
            let guard = match self.flusher_tx.read() {
                Ok(g) => g,
                Err(e) => e.into_inner(),
            };
            guard.clone()
        };

        // FIX(2026-10-07): Skip rewrite_as_v3 when flusher_tx is None (read-only Wal handle)
        let tx = match flusher_tx {
            Some(tx) => tx,
            None => {
                return Ok(());
            }
        };

        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        tx.send(WalCommand::Rewrite {
            replayed_entries: replayed_entries.to_vec(),
            integrity_key,
            ack: ack_tx,
        })
        .await
        .map_err(|_| ContextraError::Storage("WAL flusher channel closed".into()))?;

        ack_rx
            .await
            .map_err(|_| ContextraError::Storage("WAL flusher dropped".into()))??;

        Ok(())
    }

    pub async fn truncate(&self, offset: u64, new_last_hmac: [u8; 32]) -> Result<()> {
        let _truncate_guard = self.truncate_lock.lock().await;

        if self.is_sealed() {
            return Err(ContextraError::Storage(format!(
                "Cannot truncate sealed WAL segment {}",
                self.path.display()
            )));
        }

        let flusher_tx = {
            let guard = match self.flusher_tx.read() {
                Ok(g) => g,
                Err(e) => e.into_inner(),
            };
            guard.clone()
        };
        let tx = flusher_tx
            .ok_or_else(|| ContextraError::Storage("WAL flusher actor is not enabled".into()))?;

        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        tx.send(WalCommand::Truncate {
            offset,
            new_last_hmac,
            ack: ack_tx,
        })
        .await
        .map_err(|_| ContextraError::Storage("WAL flusher channel closed".into()))?;

        ack_rx.await.map_err(|_| {
            ContextraError::Storage("WAL flusher dropped ack for truncate command".into())
        })??;

        Ok(())
    }

    pub async fn rotate_and_seal(&self) -> Result<PathBuf> {
        self.sealed.store(true, std::sync::atomic::Ordering::SeqCst);

        let flusher_tx = {
            let mut guard = self
                .flusher_tx
                .write()
                .map_err(|_| ContextraError::Storage("flusher_tx RwLock poisoned".into()))?;
            guard.take()
        };

        let tx = flusher_tx
            .ok_or_else(|| ContextraError::Storage("WAL flusher actor is not enabled".into()))?;

        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        tx.send(WalCommand::Seal { ack: ack_tx })
            .await
            .map_err(|_| ContextraError::Storage("WAL flusher channel closed".into()))?;

        let sealed_path = ack_rx
            .await
            .map_err(|_| ContextraError::Storage("WAL flusher dropped".into()))??;

        Ok(sealed_path)
    }

    pub async fn find_tx_offset(&self, target_tx_id: TxId) -> Result<(u64, [u8; 32])> {
        let metadata = crate::wal::fs::metadata(&self.path)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        let file_size = metadata.len();

        let mut last_offset = 0u64;
        let mut last_hmac = [0u8; 32];
        let mut found_rollback_point = false;

        self.scan_entries_with_callback(file_size, |_seq, entry, offset| {
            let entry_tx = entry.tx_id().inner();
            if target_tx_id.inner() < TxId::INTERNAL_BASE && entry_tx >= TxId::INTERNAL_BASE {
                last_offset = offset;
                last_hmac = entry.checksum;
                return true;
            }

            if entry_tx > target_tx_id.inner() {
                found_rollback_point = true;
                return false;
            }
            last_offset = offset;
            last_hmac = entry.checksum;
            true
        })
        .await?;

        let _ = found_rollback_point;
        Ok((last_offset, last_hmac))
    }

    /// Returns a snapshot of the last HMAC written to the log.
    pub async fn last_hmac_snapshot(&self) -> [u8; 32] {
        *self.last_hmac.lock().await
    }

    /// Aktiviert eine einmalige simulierte Truncate-I/O-Failure für Tests.
    /// Der Flusher gibt beim nächsten `WalCommand::Truncate` einen Fehler zurück,
    /// ohne `set_len()` auszuführen. `size` wird daher nicht aktualisiert.
    ///
    /// Setzt `FAIL_TRUNCATE_ONCE = true`. Wird vom Flusher nach Auslösung zurückgesetzt.
    #[cfg(feature = "fault-injection")]
    pub fn arm_truncate_failure_for_test(&self) {
        self.fault_plan.fail_truncate_once.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(windows)]
pub(crate) fn set_restrictive_file_acl(path: &Path) -> Result<()> {
    contextra_sys::set_restrictive_file_acl(path)
        .map_err(|e| ContextraError::Storage(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_do_scan_entries_with_callback_v3_without_key_manager() {
        let dir = tempdir().expect("tempdir creation failed");
        let file_path = dir.path().join("test_v3.wal");

        {
            let mut file = super::super::fs::File::create(&file_path)
                .await
                .expect("file create failed");
            file.write_all(&WAL_V3_HEADER)
                .await
                .expect("write header failed");
            file.flush().await.expect("flush failed");
        }

        let mut file = super::super::fs::OpenOptions::new()
            .read(true)
            .open(&file_path)
            .await
            .expect("open read failed");

        let fallback_key = [1u8; 32];
        let mut scanned_entries = Vec::new();

        let dummy_legacy_flag = std::sync::atomic::AtomicBool::new(false);
        let res = do_scan_entries_with_callback(
            &mut file,
            4,
            &file_path,
            None,
            Some(fallback_key),
            false,
            WalVersion::V3,
            &dummy_legacy_flag,
            |_seq, entry, _pos| {
                scanned_entries.push(entry);
                true
            },
        )
        .await;

        assert!(res.is_ok());
        assert_eq!(res.expect("scan result"), WalVersion::V3);
        assert!(scanned_entries.is_empty());
    }
}
