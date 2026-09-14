use memfuse_core::{MemFuseError, Result, TxId};
use memfuse_security::wal_crypto::{IntegrityVerifier, WalEntrySnapshot};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use super::{
    legacy_integrity_key, FlusherMessage, PreparedBatch, Wal, WalEntry, WalOp, WalVersion,
    MAX_WAL_ENTRY_SIZE, WAL_V2_HEADER, WAL_V3_HEADER,
};

#[cfg(feature = "fault-injection")]
use super::{DELAY_APPEND_FOR_TX, DELAY_APPEND_MS, FAIL_APPEND_FOR_TX};

impl Wal {
    // AI-TAG[SMELL][ANALYZED-SAFE] audit-C-3: Exklusiver Mutex-Lock self.file.lock() in append_batch serialisiert Header-Check (write_header) und Dateischreibzugriffe vollständig. Die HMAC-Korrektheit wird NICHT durch die self.file-Mutex-Serialisierung, sondern durch den separaten last_hmac-Mutex in prepare_batch garantiert (siehe last_hmac.lock() in prepare_batch). (ID: AGT-STORE-d73203c0) (TS: 2026-09-10T19:14:58Z) (SESSION: 21a8d3e8)
    pub(crate) async fn append_batch(&self, batch: PreparedBatch) -> Result<()> {
        if self.is_sealed() {
            return Err(MemFuseError::Storage(format!(
                "Cannot append to sealed WAL segment {}",
                self.path.display()
            )));
        }

        let entries = &batch.0;
        if entries.is_empty() {
            return Ok(());
        }

        #[cfg(feature = "fault-injection")]
        {
            let fail_tx = FAIL_APPEND_FOR_TX.load(std::sync::atomic::Ordering::SeqCst);
            if fail_tx != 0 && entries.iter().any(|e| e.tx_id().inner() == fail_tx) {
                FAIL_APPEND_FOR_TX.store(0, std::sync::atomic::Ordering::SeqCst);
                return Err(MemFuseError::Storage(
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
            let mut batch_plaintext = Vec::with_capacity(estimated_size);
            for entry in entries {
                let bytes = entry.to_bytes()?;
                batch_plaintext.extend_from_slice(&bytes);
                last_hmac_val = entry.checksum;
            }

            let km_clone = Arc::clone(km);
            let encrypted_result =
                tokio::task::spawn_blocking(move || km_clone.encrypt_auto_nonce(&batch_plaintext))
                    .await
                    .map_err(|e| {
                        MemFuseError::Storage(format!("WAL encryption task panicked: {e}"))
                    })?;

            let (encrypted, nonce) = encrypted_result?;
            let chunk_len = (12 + encrypted.len()) as u32;

            payload_bytes.extend_from_slice(&chunk_len.to_le_bytes());
            payload_bytes.extend_from_slice(&nonce);
            payload_bytes.extend_from_slice(&encrypted);
        } else {
            for entry in entries {
                let bytes = entry.to_bytes()?;
                payload_bytes.extend_from_slice(&bytes);
                last_hmac_val = entry.checksum;
            }
        }

        let flusher_tx = {
            let guard = self.flusher_tx.read().unwrap_or_else(|e| e.into_inner());
            guard.clone()
        };

        if let Some(tx) = flusher_tx {
            let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
            tx.send(FlusherMessage {
                payload: payload_bytes,
                last_hmac_val,
                ack: ack_tx,
            })
            .map_err(|_| MemFuseError::Storage("WAL flusher channel closed".into()))?;

            ack_rx
                .await
                .map_err(|_| MemFuseError::Storage("WAL flusher dropped".into()))??;
            return Ok(());
        } else {
            // self.file.lock() Call Location 2
            let mut file = self.file.lock().await;
            if self.is_sealed() {
                return Err(MemFuseError::Storage(format!(
                    "Cannot append to sealed WAL segment {}",
                    self.path.display()
                )));
            }

            let write_header = !self
                .header_written
                .load(std::sync::atomic::Ordering::Acquire)
                && self.size.load(std::sync::atomic::Ordering::Acquire) == 0;

            if write_header {
                file.write_all(&WAL_V3_HEADER).await.map_err(|e| {
                    MemFuseError::Storage(format!(
                        "WAL header write failed for {}: {}",
                        self.path.display(),
                        e
                    ))
                })?;
            }

            file.write_all(&payload_bytes).await.map_err(|e| {
                MemFuseError::Storage(format!(
                    "WAL batch write failed for {}: {}",
                    self.path.display(),
                    e
                ))
            })?;
            file.flush().await.map_err(|e| {
                MemFuseError::Storage(format!(
                    "WAL batch flush failed for {}: {}",
                    self.path.display(),
                    e
                ))
            })?;
            file.sync_all().await.map_err(|e| {
                MemFuseError::Storage(format!(
                    "WAL batch fsync failed for {}: {}",
                    self.path.display(),
                    e
                ))
            })?;

            if write_header {
                self.header_written
                    .store(true, std::sync::atomic::Ordering::Release);
            }

            let written_len =
                (if write_header { WAL_V3_HEADER.len() } else { 0 }) + payload_bytes.len();
            self.size
                .fetch_add(written_len as u64, std::sync::atomic::Ordering::SeqCst);
        }

        let mut last_hmac = self.last_hmac.lock().await;
        *last_hmac = last_hmac_val;

        Ok(())
    }

    /// Helper for creating entries bound to this WAL's current chain.
    #[allow(dead_code)]
    #[deprecated(
        note = "Use prepare_batch with a single-element Vec instead — direct use bypasses chain-fork protection"
    )]
    pub(crate) async fn create_entry(&self, op: WalOp, seq_no: u64) -> Result<WalEntry> {
        let last_hmac = self.last_hmac.lock().await;
        let integrity_key = self.get_integrity_key()?;
        WalEntry::try_new(op, seq_no, &integrity_key, *last_hmac)
    }

    /// Scans the WAL entry by entry, executing full HMAC chain validation, CRC checks, and key manager decryption.
    ///
    /// Invokes `callback(seq_no, entry, end_offset)` for each valid entry.
    /// If `callback` returns `false`, scanning halts early.
    pub(crate) async fn scan_entries_with_callback<F>(
        &self,
        file_size: u64,
        mut callback: F,
    ) -> Result<WalVersion>
    where
        F: FnMut(u64, WalEntry, u64) -> bool,
    {
        // self.file.lock() Call Location 3
        let mut file = self.file.lock().await;
        file.seek(std::io::SeekFrom::Start(0))
            .await
            .map_err(|e| MemFuseError::Storage(format!("WAL replay seek failed: {}", e)))?;

        let mut reader = tokio::io::BufReader::new(&mut *file);

        let mut entries_count = 0u64;
        let mut pos = 0u64;

        let mut version = WalVersion::V1;
        if file_size == 0 {
            return Ok(version);
        }

        let integrity_key = self.get_integrity_key()?;
        let mut verifier = IntegrityVerifier::new(&integrity_key);
        let mut using_legacy_key = false;

        if file_size >= 4 {
            let mut header_buf = [0u8; 4];
            if reader.read_exact(&mut header_buf).await.is_ok() {
                if header_buf == WAL_V3_HEADER {
                    version = WalVersion::V3;
                    pos = 4;
                } else if header_buf == WAL_V2_HEADER {
                    version = WalVersion::V2;
                    pos = 4;
                } else {
                    file.seek(std::io::SeekFrom::Start(0))
                        .await
                        .map_err(|e| {
                            MemFuseError::Storage(format!("WAL replay seek failed: {}", e))
                        })?;
                    reader = tokio::io::BufReader::new(&mut *file);
                    pos = 0;
                }
            }
        }

        'scan_loop: while pos < file_size {
            let mut len_buf = [0u8; 4];
            if let Err(e) = reader.read_exact(&mut len_buf).await {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    tracing::warn!(
                        "WAL tail corruption (partial entry length) at offset {}",
                        pos
                    );
                    break;
                }
                return Err(MemFuseError::Storage(format!("WAL read failed: {}", e)));
            }

            let len = u32::from_le_bytes(len_buf) as usize;

            if len > MAX_WAL_ENTRY_SIZE as usize {
                if pos + 4 + len as u64 > file_size {
                    if entries_count == 0 && file_size > 64 {
                        return Err(MemFuseError::wal_corruption(
                            pos,
                            format!(
                                "WAL entry length ({}) exceeds hard limit and file size",
                                len
                            ),
                        ));
                    }
                    tracing::warn!("WAL tail corruption (huge len) at offset {}", pos);
                    break;
                }
                return Err(MemFuseError::wal_corruption(
                    pos,
                    format!("WAL entry too large ({} bytes)", len),
                ));
            }

            if pos + 4 + len as u64 > file_size {
                if entries_count == 0 && file_size > 64 {
                    return Err(MemFuseError::wal_corruption(
                        pos,
                        format!(
                            "WAL entry length ({}) exceeds file size ({}) at start of file",
                            len, file_size
                        ),
                    ));
                }
                tracing::warn!("WAL tail corruption (partial entry) at offset {}", pos);
                break;
            }

            let mut entry_data_raw = vec![0u8; len];
            if let Err(e) = reader.read_exact(&mut entry_data_raw).await {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    tracing::warn!(
                        "WAL tail corruption (truncated entry payload) at offset {}",
                        pos
                    );
                    break;
                }
                return Err(MemFuseError::Storage(format!("WAL read failed: {}", e)));
            }

            let chunk_start_pos = pos;
            pos += (4 + len) as u64;

            if matches!(version, WalVersion::V2 | WalVersion::V3) && self.key_manager.is_some() {
                let km = match self.key_manager.as_ref() {
                    Some(km) => km,
                    None => unreachable!(),
                };
                if entry_data_raw.len() < 12 {
                    if pos >= file_size {
                        tracing::warn!("WAL truncated during read at offset {}", chunk_start_pos);
                        break;
                    }
                    return Err(MemFuseError::Storage(
                        "WAL entry too short for nonce".into(),
                    ));
                }
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&entry_data_raw[0..12]);
                let decrypted_data = match km.decrypt_auto_nonce(&entry_data_raw[12..], &nonce) {
                    Ok(data) => data,
                    Err(e) => {
                        if pos >= file_size {
                            tracing::warn!(
                                "WAL truncation at tail (offset {}), decryption failed: {}",
                                chunk_start_pos,
                                e
                            );
                            break;
                        }
                        return Err(MemFuseError::wal_corruption(
                            chunk_start_pos,
                            format!("Decryption failed: {}", e),
                        ));
                    }
                };

                let mut inner_slice = decrypted_data.as_slice();
                while !inner_slice.is_empty() {
                    if inner_slice.len() < 4 {
                        if pos >= file_size {
                            tracing::warn!(
                                "WAL truncation at tail (offset {}), incomplete inner framing",
                                chunk_start_pos
                            );
                            break;
                        }
                        return Err(MemFuseError::wal_corruption(
                            chunk_start_pos,
                            "Truncated inner WAL entry length in batch",
                        ));
                    }
                    let inner_len_bytes: [u8; 4] = match inner_slice[0..4].try_into() {
                        Ok(b) => b,
                        Err(_) => {
                            return Err(MemFuseError::wal_corruption(
                                chunk_start_pos,
                                "Failed to extract inner WAL entry length",
                            ));
                        }
                    };
                    let inner_len = u32::from_le_bytes(inner_len_bytes) as usize;
                    if inner_slice.len() < 4 + inner_len {
                        if pos >= file_size {
                            tracing::warn!(
                                "WAL truncation at tail (offset {}), incomplete inner payload",
                                chunk_start_pos
                            );
                            break;
                        }
                        return Err(MemFuseError::wal_corruption(
                            chunk_start_pos,
                            "Truncated inner WAL entry in batch",
                        ));
                    }
                    let inner_entry_bytes = &inner_slice[4..4 + inner_len];
                    inner_slice = &inner_slice[4 + inner_len..];

                    let entry = match WalEntry::from_bytes(inner_entry_bytes) {
                        Ok(e) => e,
                        Err(e) => {
                            let err_msg = format!("{}", e);
                            let is_crc_error = err_msg.contains("CRC mismatch");

                            if pos >= file_size && !is_crc_error {
                                tracing::warn!(
                                    "WAL truncation at tail (offset {}), partial entry: {}",
                                    chunk_start_pos,
                                    e
                                );
                                break;
                            } else {
                                let reason = if is_crc_error {
                                    format!("CRC validation failed: {e}")
                                } else {
                                    format!("Deserialization failed: {e}")
                                };
                                return Err(MemFuseError::wal_corruption(chunk_start_pos, reason));
                            }
                        }
                    };

                    let (op_type, key, value) = match &entry.op {
                        WalOp::Put { key, value, .. } => (0u8, key.clone(), value.clone()),
                        WalOp::Delete { key, .. } => (1u8, key.clone(), Vec::new()),
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
                        WalVersion::V3 => {
                            verifier.verify_and_update_v3(&snapshot, chunk_start_pos)
                        }
                        WalVersion::V2 => {
                            verifier.verify_and_update_v2(&snapshot, chunk_start_pos)
                        }
                        WalVersion::V1 => {
                            verifier.skip_hmac_verify_legacy(&snapshot);
                            Ok(())
                        }
                    };

                    if let Err(e) = verify_res {
                        if !using_legacy_key && self.allow_legacy_integrity_key_fallback {
                            let mut legacy_verifier =
                                IntegrityVerifier::new(&legacy_integrity_key());
                            legacy_verifier.set_last_hmac(verifier.last_hmac_snapshot());
                            let legacy_res = match version {
                                WalVersion::V3 => legacy_verifier
                                    .verify_and_update_v3(&snapshot, chunk_start_pos),
                                WalVersion::V2 => legacy_verifier
                                    .verify_and_update_v2(&snapshot, chunk_start_pos),
                                WalVersion::V1 => {
                                    legacy_verifier.skip_hmac_verify_legacy(&snapshot);
                                    Ok(())
                                }
                            };
                            if legacy_res.is_ok() {
                                tracing::warn!(
                                    "WAL nutzt veralteten Integritätsschlüssel — Datenbank sollte neu initialisiert werden"
                                );
                                verifier = legacy_verifier;
                                using_legacy_key = true;
                            } else {
                                return Err(e.into());
                            }
                        } else {
                            return Err(e.into());
                        }
                    }

                    entries_count += 1;
                    let seq = entry.seq_no;
                    if !callback(seq, entry, pos) {
                        break 'scan_loop;
                    }
                }
            } else {
                let decrypted_data;
                let entry_data = if let Some(km) = &self.key_manager {
                    if entry_data_raw.len() < 12 {
                        return Err(MemFuseError::Storage(
                            "WAL entry too short for nonce".into(),
                        ));
                    }
                    let mut nonce = [0u8; 12];
                    nonce.copy_from_slice(&entry_data_raw[0..12]);
                    decrypted_data = match km.decrypt_auto_nonce(&entry_data_raw[12..], &nonce) {
                        Ok(data) => data,
                        Err(e) => {
                            if version == WalVersion::V1 {
                                return Err(MemFuseError::Storage(format!(
                                    "WAL entry at {} claims V1/plaintext format while KeyManager is active for {} \
                                     (decryption failed: {}) — refusing potential downgrade attack. \
                                     Set allow_legacy_integrity_key_fallback / min_wal_version appropriately if \
                                     this WAL genuinely predates encryption and requires migration.",
                                    chunk_start_pos,
                                    self.path.display(),
                                    e
                                )));
                            } else {
                                if pos >= file_size {
                                    tracing::warn!(
                                        "WAL truncation at tail (offset {}), decryption failed: {}",
                                        chunk_start_pos,
                                        e
                                    );
                                    break;
                                }
                                return Err(MemFuseError::wal_corruption(
                                    chunk_start_pos,
                                    format!("Decryption failed: {}", e),
                                ));
                            }
                        }
                    };
                    &decrypted_data
                } else {
                    &entry_data_raw
                };

                let entry = match WalEntry::from_bytes(entry_data) {
                    Ok(e) => e,
                    Err(e) => {
                        if let Some(err) =
                            Self::handle_wal_entry_parse_error(e, chunk_start_pos, pos, file_size)
                        {
                            return Err(err);
                        }
                        break;
                    }
                };

                let (op_type, key, value) = match &entry.op {
                    WalOp::Put { key, value, .. } => (0u8, key.clone(), value.clone()),
                    WalOp::Delete { key, .. } => (1u8, key.clone(), Vec::new()),
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
                    WalVersion::V3 => verifier.verify_and_update_v3(&snapshot, chunk_start_pos),
                    WalVersion::V2 => verifier.verify_and_update_v2(&snapshot, chunk_start_pos),
                    WalVersion::V1 => {
                        verifier.skip_hmac_verify_legacy(&snapshot);
                        Ok(())
                    }
                };

                if let Err(e) = verify_res {
                    if !using_legacy_key && self.allow_legacy_integrity_key_fallback {
                        let mut legacy_verifier = IntegrityVerifier::new(&legacy_integrity_key());
                        legacy_verifier.set_last_hmac(verifier.last_hmac_snapshot());
                        let legacy_res = match version {
                            WalVersion::V3 => {
                                legacy_verifier.verify_and_update_v3(&snapshot, chunk_start_pos)
                            }
                            WalVersion::V2 => {
                                legacy_verifier.verify_and_update_v2(&snapshot, chunk_start_pos)
                            }
                            WalVersion::V1 => {
                                legacy_verifier.skip_hmac_verify_legacy(&snapshot);
                                Ok(())
                            }
                        };
                        if legacy_res.is_ok() {
                            tracing::warn!(
                                "WAL nutzt veralteten Integritätsschlüssel — Datenbank sollte neu initialisiert werden"
                            );
                            verifier = legacy_verifier;
                            using_legacy_key = true;
                        } else {
                            return Err(e.into());
                        }
                    } else {
                        return Err(e.into());
                    }
                }

                entries_count += 1;
                let seq = entry.seq_no;
                if !callback(seq, entry, pos) {
                    break 'scan_loop;
                }
            }
        }

        Ok(version)
    }

    /// Rewrites legacy V1 or V2 WAL files as V3.
    pub(crate) async fn rewrite_as_v3(
        &self,
        replayed_entries: &[(u64, WalEntry, u64)],
    ) -> Result<()> {
        let integrity_key = self.get_integrity_key()?;
        let mut v3_entries = Vec::with_capacity(replayed_entries.len());
        let mut prev_hmac = [0u8; 32];

        for (_, entry, _) in replayed_entries {
            let v3_entry =
                WalEntry::try_new(entry.op.clone(), entry.seq_no, &integrity_key, prev_hmac)?;
            prev_hmac = v3_entry.checksum;
            v3_entries.push(v3_entry);
        }

        // self.file.lock() Call Location 4
        let mut file = self.file.lock().await;
        file.seek(std::io::SeekFrom::Start(0)).await.map_err(|e| {
            MemFuseError::Storage(format!("WAL seek failed during migration: {}", e))
        })?;
        file.set_len(0).await.map_err(|e| {
            MemFuseError::Storage(format!("WAL truncate failed during migration: {}", e))
        })?;

        let mut total_bytes = Vec::new();
        total_bytes.extend_from_slice(&WAL_V3_HEADER);

        let mut last_hmac_val = [0u8; 32];
        if let Some(km) = &self.key_manager {
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

        file.write_all(&total_bytes)
            .await
            .map_err(|e| MemFuseError::Storage(format!("WAL migration write failed: {}", e)))?;
        file.flush()
            .await
            .map_err(|e| MemFuseError::Storage(format!("WAL migration flush failed: {}", e)))?;
        file.sync_all()
            .await
            .map_err(|e| MemFuseError::Storage(format!("WAL migration fsync failed: {}", e)))?;

        self.size.store(
            total_bytes.len() as u64,
            std::sync::atomic::Ordering::SeqCst,
        );
        self.header_written
            .store(true, std::sync::atomic::Ordering::Release);
        let mut last_hmac = self.last_hmac.lock().await;
        *last_hmac = last_hmac_val;

        Ok(())
    }

    pub async fn truncate(&self, offset: u64, new_last_hmac: [u8; 32]) -> Result<()> {
        // self.file.lock() Call Location 5
        let mut file = self.file.lock().await;
        if self.is_sealed() {
            return Err(MemFuseError::Storage(format!(
                "Cannot truncate sealed WAL segment {}",
                self.path.display()
            )));
        }

        self.size.store(offset, std::sync::atomic::Ordering::SeqCst);
        if offset < 4 {
            self.header_written
                .store(false, std::sync::atomic::Ordering::Release);
        }

        file.set_len(offset)
            .await
            .map_err(|e| MemFuseError::Storage(format!("WAL truncate failed: {e}")))?;

        file.sync_all()
            .await
            .map_err(|e| MemFuseError::Storage(format!("WAL truncate fsync failed: {e}")))?;

        file.seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(|e| MemFuseError::Storage(format!("WAL seek after truncate failed: {e}")))?;

        {
            let mut last_hmac_guard = self.last_hmac.lock().await;
            *last_hmac_guard = new_last_hmac;
        }

        drop(file);

        Ok(())
    }

    pub async fn rotate_and_seal(&self) -> Result<PathBuf> {
        self.sealed.store(true, std::sync::atomic::Ordering::SeqCst);

        let old_tx = {
            let mut guard = self
                .flusher_tx
                .write()
                .map_err(|_| MemFuseError::Storage("flusher_tx RwLock poisoned".into()))?;
            guard.take()
        };
        drop(old_tx);

        let _hmac_guard = self.last_hmac.lock().await;

        // self.file.lock() Call Location 6
        {
            let file = self.file.lock().await;
            file.sync_all().await.map_err(|e| {
                MemFuseError::Storage(format!(
                    "WAL fsync vor rotate_and_seal fehlgeschlagen: {}",
                    e
                ))
            })?;
        }

        let micros = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();
        let sealed_name = format!(
            "{}.sealed.{}",
            self.path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("wal"),
            micros
        );
        let sealed_path = self.path.with_file_name(sealed_name);

        tokio::fs::rename(&self.path, &sealed_path)
            .await
            .map_err(|e| {
                MemFuseError::Storage(format!(
                    "WAL rotate_and_seal rename {} → {} fehlgeschlagen: {}",
                    self.path.display(),
                    sealed_path.display(),
                    e
                ))
            })?;

        crate::util::fsync_parent_dir(&sealed_path).await?;

        let mut perms = tokio::fs::metadata(&sealed_path)
            .await
            .map_err(|e| {
                MemFuseError::Storage(format!("WAL metadata nach seal fehlgeschlagen: {}", e))
            })?
            .permissions();
        perms.set_readonly(true);
        tokio::fs::set_permissions(&sealed_path, perms)
            .await
            .map_err(|e| {
                MemFuseError::Storage(format!("WAL set_readonly fehlgeschlagen: {}", e))
            })?;

        Ok(sealed_path)
    }

    pub async fn find_tx_offset(&self, target_tx_id: TxId) -> Result<(u64, [u8; 32])> {
        let metadata = tokio::fs::metadata(&self.path)
            .await
            .map_err(|e| MemFuseError::Storage(e.to_string()))?;
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
}

#[cfg(windows)]
#[allow(unsafe_code)]
pub(crate) fn set_restrictive_file_acl(path: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_SUCCESS, GENERIC_ALL, HANDLE,
    };
    use windows_sys::Win32::Security::Authorization::{SetNamedSecurityInfoW, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::{
        AddAccessAllowedAce, GetLengthSid, GetTokenInformation, InitializeAcl, TokenUser,
        ACCESS_ALLOWED_ACE, ACL, ACL_REVISION, DACL_SECURITY_INFORMATION,
        PROTECTED_DACL_SECURITY_INFORMATION, TOKEN_QUERY, TOKEN_USER,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token_handle: HANDLE = std::ptr::null_mut();
    let res = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) };
    if res == 0 {
        let err = unsafe { GetLastError() };
        return Err(MemFuseError::Storage(format!(
            "Failed to open process token for ACL restriction: Win32 error {}",
            err
        )));
    }

    struct TokenGuard(HANDLE);
    impl Drop for TokenGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { CloseHandle(self.0) };
            }
        }
    }
    let _guard = TokenGuard(token_handle);

    let mut len = 0u32;
    unsafe {
        GetTokenInformation(token_handle, TokenUser, null_mut(), 0, &mut len);
    }

    if len == 0 {
        return Err(MemFuseError::Storage(
            "GetTokenInformation returned 0 buffer length for TokenUser".into(),
        ));
    }

    let mut buffer = vec![0u8; len as usize];
    let res = unsafe {
        GetTokenInformation(
            token_handle,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            len,
            &mut len,
        )
    };
    if res == 0 {
        let err = unsafe { GetLastError() };
        return Err(MemFuseError::Storage(format!(
            "Failed to retrieve process owner SID: Win32 error {}",
            err
        )));
    }

    let token_user = buffer.as_ptr() as *const TOKEN_USER;
    let owner_sid = unsafe { (*token_user).User.Sid };
    if owner_sid.is_null() {
        return Err(MemFuseError::Storage(
            "Retrieved null owner SID from process token".into(),
        ));
    }

    let sid_len = unsafe { GetLengthSid(owner_sid) };
    let acl_size =
        std::mem::size_of::<ACL>() + std::mem::size_of::<ACCESS_ALLOWED_ACE>() + sid_len as usize;

    let mut acl_buf = vec![0u8; acl_size];
    let p_acl = acl_buf.as_mut_ptr() as *mut ACL;

    if unsafe { InitializeAcl(p_acl, acl_size as u32, ACL_REVISION) } == 0 {
        let err = unsafe { GetLastError() };
        return Err(MemFuseError::Storage(format!(
            "Failed to initialize ACL: Win32 error {}",
            err
        )));
    }

    if unsafe { AddAccessAllowedAce(p_acl, ACL_REVISION, GENERIC_ALL, owner_sid) } == 0 {
        let err = unsafe { GetLastError() };
        return Err(MemFuseError::Storage(format!(
            "Failed to add ACE to ACL: Win32 error {}",
            err
        )));
    }

    let path_wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let status = unsafe {
        SetNamedSecurityInfoW(
            path_wide.as_ptr() as *mut _,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            p_acl,
            null_mut(),
        )
    };

    if status != ERROR_SUCCESS {
        return Err(MemFuseError::Storage(format!(
            "SetNamedSecurityInfoW failed for {} with Win32 error code {}",
            path.display(),
            status
        )));
    }

    Ok(())
}

#[cfg(not(windows))]
pub(crate) fn set_restrictive_file_acl(path: &Path) -> Result<()> {
    let _ = path;
    Ok(())
}
