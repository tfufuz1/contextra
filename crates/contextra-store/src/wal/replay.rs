use contextra_core::{ContextraError, Result};
#[cfg(feature = "wal-integrity")]
use contextra_crypto::wal_crypto::{IntegrityVerifier, WalEntrySnapshot};
use std::path::PathBuf;

use super::{
    legacy_integrity_key, Wal, WalEntry, WalOp, WalParseError, WalVersion, MAX_WAL_ENTRY_SIZE,
    WAL_V2_HEADER, WAL_V3_HEADER,
};

pub type WalSeq = u64;

pub trait ReplayProgressSink: Send + Sync {
    fn on_entry_replayed(&self, seq: WalSeq, entries_total_estimate: Option<u64>);
}

pub struct NoopReplayProgressSink;
impl ReplayProgressSink for NoopReplayProgressSink {
    fn on_entry_replayed(&self, _seq: WalSeq, _entries_total_estimate: Option<u64>) {}
}

impl Wal {
    pub(crate) fn handle_wal_entry_parse_error(
        e: WalParseError,
        chunk_start_pos: u64,
        pos: u64,
        file_size: u64,
    ) -> Option<ContextraError> {
        match e {
            // TODO(Implementer): [P01 / F-02 & F-04 / HIGH & MEDIUM / JULES-P01-02]
            // Differenzierung zwischen CRC32-Fehlern am physischen Dateiende (Torn Write) und Bitrot in Dateimitte (I-3):
            // Aktuell wird jeder CrcMismatch bedingungslos als harter `wal_corruption`-Fehler gewertet.
            // Tritt der CrcMismatch jedoch am physischen Dateiende auf (`pos >= file_size`), liegt ein partieller
            // Tail Write (Crash mitten im Schreiben) vor, der sauber abgeschnitten (truncation warning) und
            // bis zum letzten validen HMAC-Frame recovered werden muss.
            // Nur CRC-Fehler VOR dem physischen Dateiende dürfen als harte Bitrot-Korruption (`wal_corruption`) ablehnen!
            WalParseError::CrcMismatch { stored, computed } => {
                Some(ContextraError::wal_corruption(
                    chunk_start_pos,
                    format!(
                        "CRC validation failed: stored={:#010x}, computed={:#010x}",
                        stored, computed
                    ),
                ))
            }
            _ => {
                if pos >= file_size {
                    tracing::warn!(
                        "WAL truncation at tail (offset {}), partial entry: {}",
                        chunk_start_pos,
                        e
                    );
                    None
                } else {
                    Some(ContextraError::wal_corruption(
                        chunk_start_pos,
                        format!("Deserialization failed: {}", e),
                    ))
                }
            }
        }
    }

    /// Replays the WAL using memory mapping (`memmap2`) for zero-copy entry decoding.
    /// Falls back to stream replay if mmap or parsing fails.
    pub async fn replay(&self) -> Result<Vec<(u64, WalEntry, u64)>> {
        self.replay_with_sink(&NoopReplayProgressSink).await
    }

    /// Replays the WAL while notifying the provided `ReplayProgressSink` of progress.
    pub async fn replay_with_sink<S: ReplayProgressSink>(
        &self,
        sink: &S,
    ) -> Result<Vec<(u64, WalEntry, u64)>> {
        match self.replay_mmap_with_sink(sink).await {
            Ok((entries, _)) => Ok(entries),
            Err(e) => {
                tracing::warn!("WAL mmap replay failed ({e}), falling back to stream reader");
                self.replay_stream().await
            }
        }
    }

    /// Replays the WAL using the stream reader (`BufReader`).
    pub async fn replay_stream(&self) -> Result<Vec<(u64, WalEntry, u64)>> {
        let metadata = crate::wal::fs::metadata(&self.path)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
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
        _file_size: u64,
    ) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)> {
        match self.replay_mmap().await {
            Ok(res) => Ok(res),
            Err(e) => {
                tracing::warn!(
                    "WAL mmap replay failed for {:?} ({}), falling back to stream reader",
                    self.path,
                    e
                );
                let entries = self.replay_stream().await?;
                Ok((entries, WalVersion::V3))
            }
        }
    }

    /// Replays the WAL using zero-copy memory mapping (`mmap`).
    /// Returns all valid entries with sequence numbers, entries, end offsets, and detected WAL version.
    #[allow(clippy::type_complexity)]
    pub async fn replay_mmap(&self) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)> {
        self.replay_mmap_with_sink(&NoopReplayProgressSink).await
    }

    /// Replays the WAL using zero-copy memory mapping (`mmap`) with progress callback.
    /// Falls back to stream replay if mmap or parsing fails.
    #[allow(clippy::type_complexity)]
    pub async fn replay_mmap_with_sink<S: ReplayProgressSink>(
        &self,
        sink: &S,
    ) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)> {
        let mmap_res = (|| -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)> {
            let std_file = std::fs::File::open(&self.path).map_err(|e| {
                ContextraError::Storage(format!("Failed to open WAL for mmap: {e}"))
            })?;
            let metadata = std_file.metadata().map_err(|e| {
                ContextraError::Storage(format!("Failed to stat WAL for mmap: {e}"))
            })?;
            let file_size = metadata.len();

            if file_size == 0 {
                return Ok((Vec::new(), WalVersion::V1));
            }

            let mmap = contextra_sys::mmap_readonly(&std_file)
                .map_err(|e| ContextraError::Storage(format!("WAL mmap failed: {e}")))?;

            self.parse_mmap_slice_with_sink(&mmap, file_size, sink)
        })();

        match mmap_res {
            Ok(res) => Ok(res),
            Err(e) => {
                tracing::warn!(
                    path = %self.path.display(),
                    error = %e,
                    "WAL mmap replay failed, falling back to stream reader"
                );
                let entries = self.replay_stream().await?;
                for (seq, _, _) in &entries {
                    sink.on_entry_replayed(*seq, None);
                }
                Ok((entries, WalVersion::V3))
            }
        }
    }

    #[allow(dead_code, clippy::type_complexity)]
    pub(crate) fn parse_mmap_slice(
        &self,
        mmap: &[u8],
        file_size: u64,
    ) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)> {
        self.parse_mmap_slice_with_sink(mmap, file_size, &NoopReplayProgressSink)
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn parse_mmap_slice_with_sink<S: ReplayProgressSink>(
        &self,
        mmap: &[u8],
        file_size: u64,
        sink: &S,
    ) -> Result<(Vec<(u64, WalEntry, u64)>, WalVersion)> {
        let slice_len = (file_size as usize).min(mmap.len());
        let slice = mmap
            .get(..slice_len)
            .ok_or_else(|| ContextraError::wal_corruption(0, "Mmap slice bounds exceeded"))?;
        let mut entries = Vec::new();
        let version = self.scan_entries_from_slice(slice, file_size, |seq, entry, pos| {
            sink.on_entry_replayed(seq, None);
            entries.push((seq, entry, pos));
            true
        })?;
        Ok((entries, version))
    }

    /// Scans the WAL using read-only memory mapping (`memmap2`) for zero-copy entry parsing.
    ///
    /// If mmap mapping or slice scanning encounters any error, it logs a warning (`tracing::warn!`)
    /// and automatically falls back to stream-based scanning (`scan_entries_with_callback`).
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
                ContextraError::Storage(format!("Failed to open WAL file for mmap: {e}"))
            })?;
            let mmap = contextra_sys::mmap_readonly(&std_file).map_err(|e| {
                ContextraError::Storage(format!(
                    "Failed to mmap WAL file {}: {e}",
                    self.path.display()
                ))
            })?;

            let slice_len = (file_size as usize).min(mmap.len());
            let slice = mmap
                .get(..slice_len)
                .ok_or_else(|| ContextraError::wal_corruption(0, "Mmap slice bounds exceeded"))?;
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
                    error = ?e,
                    "scan_entries_mmap: HMAC/integrity verify error during mmap scan, falling back to stream"
                );
                self.scan_entries_with_callback(file_size, callback).await
            }
        }
    }

    #[cfg(feature = "wal-integrity")]
    #[allow(deprecated)]
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
            if !*using_legacy_key
                && self
                    .allow_legacy_integrity_key_fallback
                    .load(std::sync::atomic::Ordering::SeqCst)
            {
                let legacy_key = legacy_integrity_key()?;
                let mut legacy_verifier = IntegrityVerifier::new(&legacy_key);
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
                        target: "contextra_store::wal::legacy_fallback",
                        wal_path = %self.path.display(),
                        event = "legacy_integrity_key_fallback_used",
                        "Legacy-WAL-Integritätsschlüssel aktiv für Segment {} — dieses Segment hat keine reale Manipulationssicherheit, da der Rückfallschlüssel öffentlich im Quellcode liegt.",
                        self.path.display()
                    );
                    *verifier = legacy_verifier;
                    *using_legacy_key = true;
                    self.legacy_key_used
                        .store(true, std::sync::atomic::Ordering::SeqCst);
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

        #[cfg(feature = "wal-integrity")]
        let integrity_key = self.get_integrity_key()?;
        #[cfg(feature = "wal-integrity")]
        let mut verifier = IntegrityVerifier::new(&integrity_key);
        let mut using_legacy_key = false;

        // Detect version from header
        if file_size >= 4 && slice_len >= 4 {
            if let Some(header_slice) = slice.get(0..4) {
                if header_slice == WAL_V3_HEADER {
                    version = WalVersion::V3;
                    pos = 4;
                } else if header_slice == WAL_V2_HEADER {
                    version = WalVersion::V2;
                    pos = 4;
                } else {
                    pos = 0;
                }
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

            let len_bytes_slice = match slice.get(pos as usize..pos as usize + 4) {
                Some(s) => s,
                None => {
                    tracing::warn!(
                        "WAL tail corruption (out of bounds length) at offset {}",
                        pos
                    );
                    break;
                }
            };
            let len_bytes: [u8; 4] = match len_bytes_slice.try_into() {
                Ok(b) => b,
                Err(_) => {
                    return Err(ContextraError::wal_corruption(
                        pos,
                        "Failed to parse WAL entry length header",
                    ));
                }
            };
            let len = u32::from_le_bytes(len_bytes) as usize;

            if len > MAX_WAL_ENTRY_SIZE as usize {
                if pos + 4 + len as u64 > file_size {
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
                    break;
                }
                return Err(ContextraError::wal_corruption(
                    pos,
                    format!("WAL entry too large ({} bytes)", len),
                ));
            }

            if pos + 4 + len as u64 > file_size || (pos + 4 + len as u64) as usize > slice.len() {
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
                break;
            }

            let entry_data_raw = match slice.get(pos as usize + 4..pos as usize + 4 + len) {
                Some(data) => data,
                None => {
                    tracing::warn!(
                        "WAL tail corruption (entry slice out of bounds) at offset {}",
                        pos
                    );
                    break;
                }
            };
            let chunk_start_pos = pos;
            pos += (4 + len) as u64;

            #[cfg(feature = "wal-integrity")]
            {
                if matches!(version, WalVersion::V2 | WalVersion::V3) && self.key_manager.is_some()
                {
                    let km = match self.key_manager.as_ref() {
                        Some(km) => km,
                        None => {
                            return Err(ContextraError::Storage("Key manager missing".into()));
                        }
                    };
                    if entry_data_raw.len() < 12 {
                        if pos >= file_size {
                            tracing::warn!(
                                "WAL truncated during read at offset {}",
                                chunk_start_pos
                            );
                            break;
                        }
                        return Err(ContextraError::Storage(
                            "WAL entry too short for nonce".into(),
                        ));
                    }
                    let mut nonce = [0u8; 12];
                    let nonce_slice = match entry_data_raw.get(0..12) {
                        Some(s) => s,
                        None => {
                            return Err(ContextraError::wal_corruption(
                                chunk_start_pos,
                                "Failed to read entry nonce",
                            ));
                        }
                    };
                    nonce.copy_from_slice(nonce_slice);
                    let ciphertext = entry_data_raw.get(12..).ok_or_else(|| {
                        ContextraError::wal_corruption(
                            chunk_start_pos,
                            "WAL entry missing ciphertext",
                        )
                    })?;
                    let decrypted_data = match km.decrypt_auto_nonce(ciphertext, &nonce) {
                        Ok(data) => data,
                        Err(e) => {
                            if pos >= file_size {
                                // Policy: An AEAD decryption failure at physical tail can be caused by zero-filled/truncated write.
                                tracing::error!(
                                    "WAL truncation at tail (offset {}), AEAD decryption failed: {}. Tolerated as incomplete tail write.",
                                    chunk_start_pos,
                                    e
                                );
                                break;
                            }
                            return Err(ContextraError::wal_corruption(
                                chunk_start_pos,
                                format!("Decryption failed: {}", e),
                            ));
                        }
                    };

                    let mut inner_slice = decrypted_data.as_slice();
                    while !inner_slice.is_empty() {
                        if inner_slice.len() < 4 {
                            return Err(ContextraError::wal_corruption(
                                chunk_start_pos,
                                "Incomplete inner framing in decrypted batch",
                            ));
                        }
                        let inner_len_bytes: [u8; 4] =
                            match inner_slice.get(0..4).and_then(|s| s.try_into().ok()) {
                                Some(b) => b,
                                None => {
                                    return Err(ContextraError::wal_corruption(
                                        chunk_start_pos,
                                        "Failed to extract inner WAL entry length",
                                    ));
                                }
                            };
                        let inner_len = u32::from_le_bytes(inner_len_bytes) as usize;
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

                        let entry = match WalEntry::from_bytes_classified(inner_entry_bytes) {
                            Ok(e) => e,
                            Err(parse_err) => {
                                // Policy: Authenticated data is never a tail!
                                return Err(ContextraError::wal_corruption(
                                    chunk_start_pos,
                                    format!("Deserialization failed in authenticated batch: {parse_err}"),
                                ));
                            }
                        };

                        let (op_type, key, value) = match &entry.op {
                            WalOp::Put { key, value, .. } => (0u8, key.clone(), value.clone()),
                            WalOp::Delete { key, .. } => (1u8, key.clone(), Vec::new()),
                            WalOp::TxEnd { committed, .. } => {
                                (2u8, Vec::new(), vec![*committed as u8])
                            }
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
                            return Err(ContextraError::Storage(
                                "WAL entry too short for nonce".into(),
                            ));
                        }
                        let mut nonce = [0u8; 12];
                        let nonce_slice = match entry_data_raw.get(0..12) {
                            Some(s) => s,
                            None => {
                                return Err(ContextraError::Storage(
                                    "WAL entry too short for nonce".into(),
                                ));
                            }
                        };
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
                                        self.path.display(),
                                        e
                                    )));
                                } else {
                                    if pos >= file_size {
                                        tracing::error!(
                                            "WAL truncation at tail (offset {}), AEAD decryption failed: {}. Tolerated as incomplete tail write.",
                                            chunk_start_pos,
                                            e
                                        );
                                        break;
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
                            if self.key_manager.is_some() && version != WalVersion::V1 {
                                return Err(ContextraError::wal_corruption(
                                    chunk_start_pos,
                                    format!("Deserialization failed in authenticated entry: {parse_err}"),
                                ));
                            }
                            if let Some(err) = Self::handle_wal_entry_parse_error(
                                parse_err,
                                chunk_start_pos,
                                pos,
                                file_size,
                            ) {
                                return Err(err);
                            }
                            break;
                        }
                    };

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

            #[cfg(not(feature = "wal-integrity"))]
            {
                let entry = match WalEntry::from_bytes_classified(entry_data_raw) {
                    Ok(e) => e,
                    Err(parse_err) => {
                        if let Some(err) = Self::handle_wal_entry_parse_error(
                            parse_err,
                            chunk_start_pos,
                            pos,
                            file_size,
                        ) {
                            return Err(err);
                        }
                        break;
                    }
                };

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
                            ContextraError::Storage(format!("WAL backup recovery copy failed: {e}"))
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
                    ContextraError::Storage(format!("WAL recovery sync_all failed: {e}"))
                })?;
            }
            // TODO(Implementer): [P01 / F-05 / MEDIUM]
            // Directory Fsync Discipline (Invariante I-6):
            // Nach dem Umbenennen (`rename`) bzw. Löschen (`remove_file`) der `.bak`-Datei fehlt der
            // Verzeichnis-Fsync (`crate::util::fsync_parent_dir(wal_path).await?`), wodurch der Verzeichniseintrag
            // nach Stromausfall verloren gehen kann. Zwingend Verzeichnis synchronisieren.
            return Ok(true);
        }
    }
    Ok(false)
}
