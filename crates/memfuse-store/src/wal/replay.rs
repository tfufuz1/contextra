use memfuse_core::{MemFuseError, Result};
use memfuse_crypto::wal_crypto::{IntegrityVerifier, WalEntrySnapshot};
use std::path::PathBuf;

use super::{
    legacy_integrity_key, Wal, WalEntry, WalOp, WalVersion, MAX_WAL_ENTRY_SIZE, WAL_V2_HEADER,
    WAL_V3_HEADER,
};

impl Wal {
    pub(crate) fn handle_wal_entry_parse_error(
        e: MemFuseError,
        chunk_start_pos: u64,
        pos: u64,
        file_size: u64,
    ) -> Option<MemFuseError> {
        let err_msg = format!("{}", e);
        let is_crc_error = err_msg.contains("CRC mismatch");

        if pos >= file_size && !is_crc_error {
            tracing::warn!(
                "WAL truncation at tail (offset {}), partial entry: {}",
                chunk_start_pos,
                e
            );
            None
        } else {
            let reason = if is_crc_error {
                format!("CRC validation failed: {}", e)
            } else {
                format!("Deserialization failed: {}", e)
            };
            Some(MemFuseError::wal_corruption(chunk_start_pos, reason))
        }
    }

    /// Replays the WAL using memory mapping (`memmap2`) for zero-copy entry decoding.
    /// Falls back to stream replay if mmap or parsing fails.
    pub async fn replay(&self) -> Result<Vec<(u64, WalEntry, u64)>> {
        let (entries, _) = self.replay_mmap().await?;
        Ok(entries)
    }

    /// Replays the WAL using the stream reader (`BufReader`).
    pub async fn replay_stream(&self) -> Result<Vec<(u64, WalEntry, u64)>> {
        let metadata = crate::wal::fs::metadata(&self.path)
            .await
            .map_err(|e| MemFuseError::Storage(e.to_string()))?;
        let mut entries = Vec::new();
        let version = self
            .scan_entries_with_callback(metadata.len(), |seq, entry, pos| {
                entries.push((seq, entry, pos));
                true
            })
            .await?;
        let _ = version;
        Ok(entries)
    }

    /// Replays the WAL and returns all entries with seq_no > since_seq_no.
    pub async fn replay_from(&self, since_seq_no: u64) -> Result<Vec<WalEntry>> {
        let all = self.replay().await?;
        Ok(all
            .into_iter()
            .filter(|(seq, _, _)| *seq > since_seq_no)
            .map(|(_, entry, _)| entry)
            .collect())
    }

    #[allow(clippy::type_complexity)]
    pub(crate) async fn replay_with_size_and_version(
        &self,
        file_size: u64,
    ) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)> {
        self.replay_mmap_with_size_and_version(file_size).await
    }

    #[allow(clippy::type_complexity)]
    async fn replay_mmap_with_size_and_version(
        &self,
        file_size: u64,
    ) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)> {
        match self.replay_mmap().await {
            Ok(res) => Ok(res),
            Err(e) => {
                tracing::warn!(
                    "WAL mmap replay failed for {:?} ({}), falling back to stream reader",
                    self.path,
                    e
                );
                let mut entries = Vec::new();
                let version = self
                    .scan_entries_with_callback(file_size, |seq, entry, pos| {
                        entries.push((seq, entry, pos));
                        true
                    })
                    .await?;
                Ok((entries, version))
            }
        }
    }

    /// Replays the WAL using zero-copy memory mapping (`mmap`).
    /// Returns all valid entries with sequence numbers, entries, end offsets, and detected WAL version.
    #[allow(unsafe_code, clippy::type_complexity)]
    pub async fn replay_mmap(&self) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)> {
        let std_file = std::fs::File::open(&self.path)
            .map_err(|e| MemFuseError::Storage(format!("Failed to open WAL for mmap: {e}")))?;
        let metadata = std_file
            .metadata()
            .map_err(|e| MemFuseError::Storage(format!("Failed to stat WAL for mmap: {e}")))?;
        let file_size = metadata.len();

        if file_size == 0 {
            return Ok((Vec::new(), WalVersion::V1));
        }

        let mmap = unsafe {
            // SAFETY:
            // 1. Invariant: `std_file` is a valid open read-only file descriptor to the WAL file.
            // 2. Guarantor: `std::fs::File::open` opened read-only; the mmap buffer is read-only.
            // 3. Call-site verified: `mmap` is held read-only locally within `replay_mmap` for entry parsing.
            // 4. ADR reference: ADR-017 (Zero-Copy Mmap for WAL Replay).
            memmap2::Mmap::map(&std_file)
                .map_err(|e| MemFuseError::Storage(format!("WAL mmap failed: {e}")))?
        };

        self.parse_mmap_slice(&mmap, file_size)
    }

    #[allow(clippy::type_complexity)]
    fn parse_mmap_slice(
        &self,
        mmap: &[u8],
        file_size: u64,
    ) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)> {
        let mut entries = Vec::new();
        let mut entries_count = 0u64;
        let mut pos = 0u64;
        let mut version = WalVersion::V1;

        if file_size == 0 {
            return Ok((entries, version));
        }

        let integrity_key = self.get_integrity_key()?;
        let mut verifier = IntegrityVerifier::new(&integrity_key);
        let mut using_legacy_key = false;

        // Detect version from header
        if file_size >= 4 {
            let header_bytes = &mmap[0..4];
            if header_bytes == WAL_V3_HEADER {
                version = WalVersion::V3;
                pos = 4;
            } else if header_bytes == WAL_V2_HEADER {
                version = WalVersion::V2;
                pos = 4;
            } else {
                pos = 0;
            }
        }

        while pos < file_size {
            if pos + 4 > file_size {
                tracing::warn!(
                    "WAL tail corruption (partial entry length) at offset {}",
                    pos
                );
                break;
            }

            let len_bytes: [u8; 4] = mmap[pos as usize..pos as usize + 4].try_into().unwrap();
            let len = u32::from_le_bytes(len_bytes) as usize;

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

            let chunk_start_pos = pos;
            let entry_data_raw = &mmap[pos as usize + 4..pos as usize + 4 + len];
            pos += 4 + len as u64;

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

                let mut slice = decrypted_data.as_slice();
                while !slice.is_empty() {
                    if slice.len() < 4 {
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
                    let inner_len_bytes: [u8; 4] = match slice[0..4].try_into() {
                        Ok(b) => b,
                        Err(_) => {
                            return Err(MemFuseError::wal_corruption(
                                chunk_start_pos,
                                "Failed to extract inner WAL entry length",
                            ));
                        }
                    };
                    let inner_len = u32::from_le_bytes(inner_len_bytes) as usize;
                    if slice.len() < 4 + inner_len {
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
                    let inner_entry_bytes = &slice[4..4 + inner_len];
                    slice = &slice[4 + inner_len..];

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
                    entries.push((seq, entry, pos));
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
                    entry_data_raw
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
                entries.push((seq, entry, pos));
            }
        }

        Ok((entries, version))
    }

    /// Scans the WAL using read-only memory mapping (`memmap2`) for zero-copy entry parsing.
    ///
    /// If mmap mapping or slice scanning encounters any error, it logs a warning (`tracing::warn!`)
    /// and automatically falls back to stream-based scanning (`scan_entries_with_callback`).
    #[allow(unsafe_code)]
    pub async fn scan_entries_mmap<F>(&self, file_size: u64, mut callback: F) -> Result<WalVersion>
    where
        F: FnMut(u64, WalEntry, u64) -> bool,
    {
        if file_size == 0 {
            return Ok(WalVersion::V1);
        }

        #[allow(clippy::type_complexity)]
        let mmap_res = (|| -> Result<(WalVersion, Vec<(u64, WalEntry, u64)>)> {
            let std_file = std::fs::File::open(&self.path).map_err(|e| {
                MemFuseError::Storage(format!("Failed to open WAL file for mmap: {e}"))
            })?;
            // SAFETY: Invariant: `std_file` is a valid open file descriptor opened in read-only mode for replay.
            // Guarantor: std::fs::File::open returned Ok above.
            // UB Prevention: Read-only mapping prevents data races. If file is truncated or removed,
            // active mmap buffer remains valid for the duration of slice scanning.
            let mmap = unsafe { memmap2::Mmap::map(&std_file) }.map_err(|e| {
                MemFuseError::Storage(format!(
                    "Failed to mmap WAL file {}: {e}",
                    self.path.display()
                ))
            })?;

            let slice_len = (file_size as usize).min(mmap.len());
            let slice = &mmap[..slice_len];
            let mut buffered_entries = Vec::new();
            let version = self.scan_entries_from_slice(slice, file_size, |seq, entry, pos| {
                buffered_entries.push((seq, entry, pos));
                true
            })?;
            Ok((version, buffered_entries))
        })();

        match mmap_res {
            Ok((version, buffered_entries)) => {
                for (seq, entry, pos) in buffered_entries {
                    if !callback(seq, entry, pos) {
                        break;
                    }
                }
                Ok(version)
            }
            Err(e) => {
                tracing::warn!(
                    "mmap replay failed for WAL {:?}: {}; falling back to stream reader",
                    self.path,
                    e
                );
                self.scan_entries_with_callback(file_size, callback).await
            }
        }
    }

    fn verify_entry_snapshot(
        &self,
        snapshot: &WalEntrySnapshot,
        version: WalVersion,
        chunk_start_pos: u64,
        verifier: &mut IntegrityVerifier,
        using_legacy_key: &mut bool,
    ) -> Result<()> {
        let verify_res = match version {
            WalVersion::V3 => verifier.verify_and_update_v3(snapshot, chunk_start_pos),
            WalVersion::V2 => verifier.verify_and_update_v2(snapshot, chunk_start_pos),
            WalVersion::V1 => {
                verifier.skip_hmac_verify_legacy(snapshot);
                Ok(())
            }
        };

        if let Err(e) = verify_res {
            if !*using_legacy_key && self.allow_legacy_integrity_key_fallback {
                let mut legacy_verifier = IntegrityVerifier::new(&legacy_integrity_key());
                legacy_verifier.set_last_hmac(verifier.last_hmac_snapshot());
                let legacy_res = match version {
                    WalVersion::V3 => {
                        legacy_verifier.verify_and_update_v3(snapshot, chunk_start_pos)
                    }
                    WalVersion::V2 => {
                        legacy_verifier.verify_and_update_v2(snapshot, chunk_start_pos)
                    }
                    WalVersion::V1 => {
                        legacy_verifier.skip_hmac_verify_legacy(snapshot);
                        Ok(())
                    }
                };
                if legacy_res.is_ok() {
                    tracing::warn!(
                        "WAL nutzt veralteten Integritätsschlüssel — Datenbank sollte neu initialisiert werden"
                    );
                    *verifier = legacy_verifier;
                    *using_legacy_key = true;
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

    fn scan_entries_from_slice<F>(
        &self,
        slice: &[u8],
        file_size: u64,
        mut callback: F,
    ) -> Result<WalVersion>
    where
        F: FnMut(u64, WalEntry, u64) -> bool,
    {
        let mut entries_count = 0u64;
        let mut pos = 0u64;
        let slice_len = slice.len() as u64;

        let mut version = WalVersion::V1;
        if file_size == 0 || slice_len == 0 {
            return Ok(version);
        }

        let integrity_key = self.get_integrity_key()?;
        let mut verifier = IntegrityVerifier::new(&integrity_key);
        let mut using_legacy_key = false;

        // Detect version from header
        if file_size >= 4 && slice_len >= 4 {
            let header_bytes: [u8; 4] = slice[0..4].try_into().unwrap();
            if header_bytes == WAL_V3_HEADER {
                version = WalVersion::V3;
                pos = 4;
            } else if header_bytes == WAL_V2_HEADER {
                version = WalVersion::V2;
                pos = 4;
            } else {
                pos = 0;
            }
        }

        'scan_loop: loop {
            if pos >= slice_len {
                break;
            }

            if pos + 4 > slice_len {
                tracing::warn!(
                    "WAL tail corruption (partial length header) at offset {}",
                    pos
                );
                break;
            }

            let len_bytes: [u8; 4] = slice[pos as usize..pos as usize + 4].try_into().unwrap();
            let len = u32::from_le_bytes(len_bytes) as usize;

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

            if pos + 4 + len as u64 > file_size || (pos + 4 + len as u64) as usize > slice.len() {
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

            let entry_data_raw = &slice[pos as usize + 4..pos as usize + 4 + len];
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

                    self.verify_entry_snapshot(
                        &snapshot,
                        version,
                        chunk_start_pos,
                        &mut verifier,
                        &mut using_legacy_key,
                    )?;

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
                    entry_data_raw
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

                self.verify_entry_snapshot(
                    &snapshot,
                    version,
                    chunk_start_pos,
                    &mut verifier,
                    &mut using_legacy_key,
                )?;

                entries_count += 1;
                let seq = entry.seq_no;
                if !callback(seq, entry, pos) {
                    break 'scan_loop;
                }
            }
        }

        Ok(version)
    }
}

pub(crate) async fn recover_from_bak_if_present(wal_path: &std::path::Path) -> Result<bool> {
    for suffix in &["v1.bak", "v2.bak"] {
        let bak_path = PathBuf::from(format!("{}.{}", wal_path.display(), suffix));
        if !crate::wal::fs::try_exists(&bak_path).await.unwrap_or(false) {
            continue;
        }
        let wal_len = crate::wal::fs::metadata(wal_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0);
        let bak_len = match crate::wal::fs::metadata(&bak_path).await {
            Ok(m) => m.len(),
            Err(_) => continue,
        };
        if wal_len == 0 && bak_len > 0 {
            // Reguläre WAL ist leer, aber ein nicht-leeres Backup existiert:
            // sehr wahrscheinlich ein Crash zwischen set_len(0) und dem Schreiben
            // des neuen V3-Contents. Backup wiederherstellen.
            match crate::wal::fs::rename(&bak_path, wal_path).await {
                Ok(()) => {}
                Err(_) => {
                    // Cross-Device-Fallback: copy + remove statt rename.
                    crate::wal::fs::copy(&bak_path, wal_path)
                        .await
                        .map_err(|e| {
                            MemFuseError::Storage(format!("WAL backup recovery copy failed: {e}"))
                        })?;
                    let _ = crate::wal::fs::remove_file(&bak_path).await;
                }
            }
            if let Ok(f) = crate::wal::fs::OpenOptions::new()
                .write(true)
                .open(wal_path)
                .await
            {
                f.sync_all().await.map_err(|e| {
                    MemFuseError::Storage(format!("WAL recovery sync_all failed: {e}"))
                })?;
            }
            return Ok(true);
        }
    }
    Ok(false)
}
