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
                    file.seek(std::io::SeekFrom::Start(0)).await.map_err(|e| {
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
                        WalVersion::V3 => verifier.verify_and_update_v3(&snapshot, chunk_start_pos),
                        WalVersion::V2 => verifier.verify_and_update_v2(&snapshot, chunk_start_pos),
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
                            let legacy_res =
                                match version {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wal::WalOp;
    #[cfg(feature = "fault-injection")]
    use crate::wal::FAIL_APPEND_FOR_TX;
    use memfuse_core::TxId;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_wal_crash_consistency_write_without_fsync() {
        let dir = tempdir().expect("tempdir"); // expect
        let wal_path = dir.path().join("crash_sim.wal");

        // 1. Open WAL and append an entry
        {
            let wal = Wal::open(&wal_path).await.expect("open wal"); // expect
            let op = WalOp::Put {
                tx_id: TxId::new(100),
                key: b"crash_k".to_vec(),
                value: b"crash_v".to_vec(),
            };
            let (batch, _) = wal
                .prepare_batch(vec![(op, 1)])
                .await
                .expect("create entry"); // expect
            let entry = &batch.entries()[0];

            // Manually simulate a write + flush to OS buffer WITHOUT file.sync_all()
            let mut file = tokio::fs::OpenOptions::new()
                .append(true)
                .open(&wal_path)
                .await
                .expect("open for append"); // expect
            let bytes = entry.to_bytes().expect("to_bytes"); // expect
            file.write_all(&bytes).await.expect("write_all"); // expect
            file.flush().await.expect("flush"); // expect
                                                // File dropped without calling sync_all() (simulating crash before fsync)
            drop(file);
            drop(wal);
        }

        // 2. Re-open WAL and replay
        let wal_reopen = Wal::open(&wal_path).await;
        assert!(wal_reopen.is_ok(), "WAL open after crash should succeed");
        let wal = wal_reopen.unwrap(); // unwrap

        let replay_result = wal.replay().await;
        match replay_result {
            Ok(entries) => {
                // Should either find the entry or empty set, never panic
                if !entries.is_empty() {
                    assert_eq!(entries.len(), 1);
                    assert_eq!(entries[0].1.seq_no, 1);
                }
            }
            Err(e) => {
                panic!("replay() failed unexpectedly with error: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_append_batch_partial_write_atomicity() {
        let dir = tempdir().expect("tempdir"); // expect
        let wal_path = dir.path().join("partial_batch.wal");

        let wal = Wal::open(&wal_path).await.expect("open wal"); // expect
        let ops = vec![
            (
                WalOp::Put {
                    tx_id: TxId::new(1),
                    key: b"b1".to_vec(),
                    value: b"v1".to_vec(),
                },
                1,
            ),
            (
                WalOp::Put {
                    tx_id: TxId::new(1),
                    key: b"b2".to_vec(),
                    value: b"v2".to_vec(),
                },
                2,
            ),
            (
                WalOp::Put {
                    tx_id: TxId::new(1),
                    key: b"b3".to_vec(),
                    value: b"v3".to_vec(),
                },
                3,
            ),
        ];

        let (entries, _) = wal.prepare_batch(ops).await.expect("prepare_batch"); // expect
        assert_eq!(entries.len(), 3);

        // Serialize all 3 entries into a single bytes payload
        let mut batch_bytes = Vec::new();
        for e in entries.entries() {
            batch_bytes.extend_from_slice(&e.to_bytes().expect("to_bytes")); // expect
        }

        // Truncate the batch in the middle of entry 2 (partial write during crash)
        // Each entry is ~101 bytes. Total ~303 bytes.
        // Subtracting 120 bytes leaves ~183 bytes, truncating entry 2 mid-write.
        let truncated_len = batch_bytes.len() - 120;
        let truncated_bytes = &batch_bytes[..truncated_len];

        // Append the truncated bytes directly to the WAL file
        {
            let mut file = tokio::fs::OpenOptions::new()
                .append(true)
                .open(&wal_path)
                .await
                .expect("open for append"); // expect
            file.write_all(truncated_bytes).await.expect("write_all"); // expect
            file.flush().await.expect("flush"); // expect
        }

        // Reopen and replay
        let wal2 = Wal::open(&wal_path).await.expect("reopen"); // expect
        let replay_entries = wal2
            .replay()
            .await
            .expect("replay must succeed without panic"); // expect

        // Replay must recover entry 1 (which was fully written) and cleanly discard the truncated tail
        assert_eq!(replay_entries.len(), 1, "Only entry 1 should be recovered");
        assert_eq!(replay_entries[0].1.seq_no, 1);
    }

    #[tokio::test]
    async fn test_truncate_size_visible_atomically_with_file_state() {
        let dir = tempdir().expect("tempdir");
        let wal_path = dir.path().join("truncate_atomic.wal");

        let wal = Arc::new(Wal::open(&wal_path).await.expect("open wal"));

        let done = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let wal_trunc = wal.clone();
        let done_trunc = done.clone();
        let truncater = tokio::spawn(async move {
            for i in 0..100 {
                // Prepare and append a batch of entries so file grows
                let ops = vec![
                    (
                        WalOp::Put {
                            tx_id: TxId::new(i * 2 + 1),
                            key: b"atomic_key_1".to_vec(),
                            value: b"atomic_val_1".to_vec(),
                        },
                        i * 2 + 1,
                    ),
                    (
                        WalOp::Put {
                            tx_id: TxId::new(i * 2 + 2),
                            key: b"atomic_key_2".to_vec(),
                            value: b"atomic_val_2".to_vec(),
                        },
                        i * 2 + 2,
                    ),
                ];
                let (batch, _) = wal_trunc.prepare_batch(ops).await.expect("prepare_batch");
                wal_trunc.append_batch(batch).await.expect("append_batch");

                // Truncate back to offset 4 (length of WAL_V3_HEADER)
                wal_trunc
                    .truncate(4, [0xAA; 32])
                    .await
                    .expect("truncate failed");
            }
            done_trunc.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        let wal_poll = wal.clone();
        let done_poll = done.clone();
        let poller = tokio::spawn(async move {
            while !done_poll.load(std::sync::atomic::Ordering::SeqCst) {
                if let Ok(meta) = fs::metadata(wal_poll.path()).await {
                    let disk_size = meta.len();
                    let mem_size = wal_poll.size();
                    // In-memory size must never observe stale mem_size > disk_size after truncation
                    assert!(
                        mem_size <= disk_size,
                        "TOCTOU violation: in-memory WAL size ({mem_size}) > physical disk size ({disk_size})"
                    );
                }
                tokio::task::yield_now().await;
            }
        });

        let (res_trunc, res_poll) = tokio::join!(truncater, poller);
        res_trunc.expect("truncater panicked");
        res_poll.expect("poller panicked");
    }

    #[tokio::test]
    async fn test_concurrent_append_batch_header_atomicity() {
        let dir = tempdir().expect("tempdir");
        let wal_path = dir.path().join("concurrent_header.wal");

        let wal = Arc::new(Wal::open(&wal_path).await.expect("open wal"));

        let num_tasks = 8;
        let mut handles = Vec::new();

        for i in 0..num_tasks {
            let wal_clone = wal.clone();
            handles.push(tokio::spawn(async move {
                let op = WalOp::Put {
                    tx_id: TxId::new(i + 1),
                    key: format!("key_{}", i).into_bytes(),
                    value: format!("val_{}", i).into_bytes(),
                };
                let (batch, _) = wal_clone
                    .prepare_batch(vec![(op, i + 1)])
                    .await
                    .expect("prepare_batch");
                wal_clone.append_batch(batch).await.expect("append_batch");
            }));
        }

        for h in handles {
            h.await.expect("join handle");
        }

        let file_bytes = fs::read(&wal_path).await.expect("read wal file");

        // Assert header is present at start
        assert!(
            file_bytes.len() >= 4,
            "WAL file must be at least 4 bytes long"
        );
        assert_eq!(
            &file_bytes[0..4],
            &WAL_V3_HEADER,
            "WAL file must start with WAL_V3_HEADER"
        );

        // Count header occurrences across entire file
        let header_count = file_bytes
            .windows(4)
            .filter(|window| *window == WAL_V3_HEADER)
            .count();
        assert_eq!(
            header_count, 1,
            "WAL_V3_HEADER must appear exactly once at the start of the file, but was found {header_count} times"
        );

        // Reopen and replay to verify no stream corruption
        let wal_reopen = Wal::open(&wal_path).await.expect("reopen wal");
        let replayed = wal_reopen.replay().await.expect("replay must succeed");
        assert_eq!(
            replayed.len(),
            num_tasks as usize,
            "Replay must yield all {} entries",
            num_tasks
        );
    }

    #[tokio::test]
    async fn test_truncate_is_durable_across_simulated_crash() {
        let dir = tempdir().expect("tempdir");
        let wal_path = dir.path().join("truncate_durability.wal");

        let wal = Wal::open(&wal_path).await.expect("open wal");

        // Write several entries so file grows
        for i in 1..=5 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("k{}", i).into_bytes(),
                value: format!("v{}", i).into_bytes(),
            };
            let (batch, _) = wal
                .prepare_batch(vec![(op, i)])
                .await
                .expect("prepare batch");
            wal.append_batch(batch).await.expect("append entry");
        }

        let initial_size = tokio::fs::metadata(&wal_path).await.expect("meta").len();
        assert!(initial_size > 4, "File size should be larger than header");

        // Truncate to offset 4 (HEADER length)
        let new_hmac = [0x77u8; 32];
        wal.truncate(4, new_hmac).await.expect("truncate");

        // Open via a new independent File handle (simulates restart after crash without the original Wal handle)
        let file = tokio::fs::File::open(&wal_path).await.expect("reopen file");
        let metadata = file.metadata().await.expect("metadata");
        assert_eq!(
            metadata.len(),
            4,
            "Physical file length on disk must equal truncated offset 4 after fsync"
        );
    }

    #[tokio::test]
    async fn test_wal_direct_append_batch_fsync_discipline() {
        let dir = tempdir().expect("tempdir");
        let wal_path = dir.path().join("wal_direct.log");

        let wal = Wal::open(&wal_path).await.expect("wal open");

        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"direct_k".to_vec(),
            value: b"direct_v".to_vec(),
        };

        let (batch, _) = wal
            .prepare_batch(vec![(op, 1)])
            .await
            .expect("create entry");

        println!("[STRACE_MARKER_START_DIRECT_APPEND]");
        let append_res = wal.append_batch(batch).await;
        println!("[STRACE_MARKER_END_DIRECT_APPEND]");

        assert!(append_res.is_ok());
    }

    #[tokio::test]
    async fn test_truncate_at_offset_zero_yields_empty_wal() -> Result<()> {
        let dir = tempdir()?;
        let wal_path = dir.path().join("test_truncate_zero.wal");

        let wal = Wal::open(&wal_path).await?;
        for i in 1..=5 {
            let op = WalOp::Put {
                tx_id: TxId::new(i),
                key: format!("k{i}").into_bytes(),
                value: format!("v{i}").into_bytes(),
            };
            let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
            wal.append_batch(batch).await?;
        }

        let meta_before = tokio::fs::metadata(&wal_path).await?;
        assert!(
            meta_before.len() > 0,
            "WAL file should contain written bytes before truncation"
        );

        wal.truncate(0, [0u8; 32]).await?;

        let meta_after = tokio::fs::metadata(&wal_path).await?;
        assert_eq!(
            meta_after.len(),
            0,
            "Physical file length must be exactly 0 bytes after truncate(0)"
        );

        drop(wal);
        let wal_reopened = Wal::open(&wal_path).await?;
        let entries = wal_reopened.replay().await?;
        assert_eq!(
            entries.len(),
            0,
            "Reopened WAL after truncate(0) must yield 0 entries"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_append_batch_concurrent_double_header_write_race() -> Result<()> {
        let dir = tempdir()?;
        let wal_path = dir.path().join("test_concurrent_header.wal");

        let wal = Wal::open(&wal_path).await?;

        let op1 = WalOp::Put {
            tx_id: TxId::new(10),
            key: b"concurrent_key_1".to_vec(),
            value: b"val_1".to_vec(),
        };
        let op2 = WalOp::Put {
            tx_id: TxId::new(11),
            key: b"concurrent_key_2".to_vec(),
            value: b"val_2".to_vec(),
        };

        let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
        let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;

        let handle1 = wal.append_batch(batch1);
        let handle2 = wal.append_batch(batch2);
        let (res1, res2) = tokio::join!(handle1, handle2);

        res1?;
        res2?;

        let file_bytes = tokio::fs::read(&wal_path).await?;

        let header_magic_count = file_bytes
            .windows(4)
            .filter(|win| *win == WAL_V3_HEADER)
            .count();

        assert_eq!(
            header_magic_count, 1,
            "Header magic 'MFW3' must appear EXACTLY ONCE in physical WAL file, found {}",
            header_magic_count
        );

        Ok(())
    }

    #[cfg(feature = "fault-injection")]
    #[tokio::test]
    async fn test_disk_full_mid_append_batch_rollback() -> Result<()> {
        use std::sync::atomic::Ordering;
        let dir = tempdir()?;
        let wal_path = dir.path().join("disk_full_batch.wal");

        let wal = Wal::open(&wal_path).await?;

        let op_base = WalOp::Put {
            tx_id: TxId::new(10),
            key: b"base_k".to_vec(),
            value: b"base_v".to_vec(),
        };
        let (batch_base, _) = wal.prepare_batch(vec![(op_base, 1)]).await?;
        wal.append_batch(batch_base).await?;

        let fail_tx = TxId::new(20);
        let op_fail1 = WalOp::Put {
            tx_id: fail_tx,
            key: b"fail_k1".to_vec(),
            value: b"fail_v1".to_vec(),
        };
        let op_fail2 = WalOp::Put {
            tx_id: fail_tx,
            key: b"fail_k2".to_vec(),
            value: b"fail_v2".to_vec(),
        };

        let (batch, prev_hmac_snapshot) = wal
            .prepare_batch(vec![(op_fail1, 2), (op_fail2, 3)])
            .await?;

        FAIL_APPEND_FOR_TX.store(fail_tx.inner(), Ordering::SeqCst);

        let append_res = wal.append_batch(batch).await;
        assert!(
            append_res.is_err(),
            "append_batch must return Err when fault injection triggers WAL append failure"
        );

        wal.restore_last_hmac(prev_hmac_snapshot).await?;

        drop(wal);
        let wal_reopened = Wal::open(&wal_path).await?;
        let entries = wal_reopened.replay().await?;

        assert_eq!(
            entries.len(),
            1,
            "WAL must contain exactly 1 baseline entry after batch failure rollback"
        );
        assert_eq!(entries[0].1.seq_no, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_wal_rotate_and_seal_readonly_guarantee() {
        let dir = tempfile::tempdir().expect("tempdir");
        let wal_path = dir.path().join("test.wal");

        // WAL öffnen und einen Eintrag schreiben
        let wal = Wal::open(&wal_path).await.expect("open WAL");
        let op = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"hello".to_vec(),
            value: b"world".to_vec(),
        };
        let (batch, _hmac) = wal
            .prepare_batch(vec![(op, 1)])
            .await
            .expect("prepare batch");
        wal.append_batch(batch).await.expect("append batch");

        // rotate_and_seal aufrufen
        let sealed_path = wal.rotate_and_seal().await.expect("rotate_and_seal");

        // Invariante 1: Sealed-Datei existiert am neuen Pfad
        assert!(sealed_path.exists(), "Sealed WAL muss existieren");
        assert!(
            sealed_path.to_str().unwrap().contains(".sealed."),
            "Sealed-Pfad muss '.sealed.' enthalten"
        );

        // Invariante 2: Originalpfad existiert nicht mehr
        assert!(
            !wal_path.exists(),
            "Originaler WAL-Pfad muss nach Rotate verschwunden sein"
        );

        // Invariante 3: Sealed-Datei ist read-only
        let meta = tokio::fs::metadata(&sealed_path).await.expect("metadata");
        assert!(
            meta.permissions().readonly(),
            "Versiegeltes WAL-Segment MUSS read-only sein"
        );

        // Invariante 4: Schreibversuch auf sealed-Datei schlägt fehl
        let write_result = tokio::fs::OpenOptions::new()
            .write(true)
            .open(&sealed_path)
            .await;
        assert!(
            write_result.is_err(),
            "Schreibversuch auf versiegeltes WAL-Segment muss fehlschlagen"
        );
    }

    #[tokio::test]
    async fn test_wal_rotate_seal_crash_mid_rename() {
        let dir = tempfile::tempdir().expect("tempdir");
        let wal_path = dir.path().join("crash_test.wal");

        // 1. Write initial committed WAL entries
        {
            let wal = Wal::open(&wal_path).await.expect("open WAL");
            for i in 1..=3 {
                let op = WalOp::Put {
                    tx_id: TxId::new(i),
                    key: format!("k{i}").into_bytes(),
                    value: format!("v{i}").into_bytes(),
                };
                let (batch, _) = wal
                    .prepare_batch(vec![(op, i)])
                    .await
                    .expect("prepare batch");
                wal.append_batch(batch).await.expect("append batch");
            }
        }

        // 2. Simulate crash state between rename and parent fsync
        let sealed_name = format!("crash_test.wal.sealed.1234567890");
        let sealed_path = dir.path().join(&sealed_name);

        tokio::fs::rename(&wal_path, &sealed_path)
            .await
            .expect("simulate rename before crash");

        // 3. Post-crash inspection & recovery verification
        let wal_exists = wal_path.exists();
        let sealed_exists = sealed_path.exists();

        assert!(
            (wal_exists && !sealed_exists) || (!wal_exists && sealed_exists),
            "WAL segment must be either fully active or fully sealed after crash mid-rename"
        );

        if sealed_exists {
            let sealed_wal = Wal::open(&sealed_path).await.expect("open sealed WAL");
            let replayed = sealed_wal.replay().await.expect("replay sealed WAL");
            assert_eq!(
                replayed.len(),
                3,
                "All 3 entries must be present in sealed WAL"
            );
        }
    }
}
