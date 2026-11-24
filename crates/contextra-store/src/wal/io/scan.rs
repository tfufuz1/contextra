//! WAL Scanning und Iteration über persistierte Frames.
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


#[allow(deprecated, clippy::too_many_arguments)]
pub(crate) async fn do_scan_entries_with_callback<F>(
    file: &mut crate::wal::fs::File,
    file_size: u64,
    path: &Path,
    _key_manager: Option<&crate::wal::KeyManager>,
    fallback_integrity_key: Option<[u8; 32]>,
    allow_legacy_integrity_key_fallback: bool,
    legacy_key_used: &std::sync::atomic::AtomicBool,
    mut callback: F,
) -> Result<WalVersion>
where
    F: FnMut(u64, WalEntry, u64) -> bool,
{
    file.seek(std::io::SeekFrom::Start(0))
        .await
        .map_err(|e| StoreError::Storage(format!("WAL replay seek failed: {}", e)))?;

    let mut reader = tokio::io::BufReader::new(file);

    let mut entries_count = 0u64;
    let mut pos = 0u64;

    let mut version = WalVersion::V1;
    if file_size == 0 {
        return Ok(version);
    }

    let key_manager = _key_manager;

    let integrity_key = if let Some(km) = key_manager {
        km.integrity_key()?
    } else if let Some(key) = fallback_integrity_key {
        key
    } else {
        return Err(StoreError::Storage("No integrity key available".into()));
    };

    let mut verifier = IntegrityVerifier::new(&integrity_key);
    let mut using_legacy_key = false;
    let _ = (
        path,
        fallback_integrity_key,
        allow_legacy_integrity_key_fallback,
        &mut using_legacy_key,
    );

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
                reader
                    .get_mut()
                    .seek(std::io::SeekFrom::Start(0))
                    .await
                    .map_err(|e| {
                        StoreError::Storage(format!("WAL replay seek failed: {}", e))
                    })?;
                reader = tokio::io::BufReader::new(reader.into_inner());
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
            return Err(StoreError::Storage(format!("WAL read failed: {}", e)));
        }

        let len = u32::from_le_bytes(len_buf) as usize;

        if len > MAX_WAL_ENTRY_SIZE as usize {
            if pos + 4 + len as u64 > file_size {
                if entries_count == 0 && file_size > 64 {
                    return Err(StoreError::wal_corruption(
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
            return Err(StoreError::wal_corruption(
                pos,
                format!("WAL entry too large ({} bytes)", len),
            ));
        }

        if pos + 4 + len as u64 > file_size {
            if entries_count == 0 && file_size > 64 {
                return Err(StoreError::wal_corruption(
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
            return Err(StoreError::Storage(format!("WAL read failed: {}", e)));
        }

        let chunk_start_pos = pos;
        pos += (4 + len) as u64;

        {
            if let (Some(km), WalVersion::V2 | WalVersion::V3) = (key_manager, version) {
                if entry_data_raw.len() < 12 {
                    if pos >= file_size {
                        tracing::warn!("WAL truncated during read at offset {}", chunk_start_pos);
                        break;
                    }
                    return Err(StoreError::Storage(
                        "WAL entry too short for nonce".into(),
                    ));
                }
                let nonce_slice = match entry_data_raw.get(0..12) {
                    Some(s) => s,
                    None => {
                        return Err(StoreError::wal_corruption(
                            chunk_start_pos,
                            "WAL entry too short for nonce",
                        ));
                    }
                };
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(nonce_slice);
                let ciphertext = entry_data_raw.get(12..).ok_or_else(|| {
                    StoreError::wal_corruption(chunk_start_pos, "WAL entry missing ciphertext")
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
                        return Err(StoreError::wal_corruption(
                            chunk_start_pos,
                            format!("Decryption failed: {}", e),
                        ));
                    }
                };

                let mut inner_slice = decrypted_data.as_slice();
                while !inner_slice.is_empty() {
                    if inner_slice.len() < 4 {
                        return Err(StoreError::wal_corruption(
                            chunk_start_pos,
                            "Incomplete inner framing in decrypted batch",
                        ));
                    }
                    let inner_len_bytes: [u8; 4] =
                        match inner_slice.get(0..4).and_then(|s| s.try_into().ok()) {
                            Some(b) => b,
                            None => {
                                return Err(StoreError::wal_corruption(
                                    chunk_start_pos,
                                    "Failed to extract inner WAL entry length",
                                ));
                            }
                        };
                    let inner_len = u32::from_le_bytes(inner_len_bytes) as usize;
                    if inner_slice.len() < 4 + inner_len {
                        return Err(StoreError::wal_corruption(
                            chunk_start_pos,
                            "Truncated inner WAL entry payload in decrypted batch",
                        ));
                    }
                    let inner_entry_bytes = match inner_slice.get(4..4 + inner_len) {
                        Some(b) => b,
                        None => {
                            return Err(StoreError::wal_corruption(
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
                            return Err(StoreError::wal_corruption(
                                chunk_start_pos,
                                format!(
                                    "Deserialization failed in authenticated batch: {parse_err}"
                                ),
                            ));
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

                    #[allow(deprecated)]
                    let verify_res = match version {
                        WalVersion::V3 => verifier.verify_and_update_v3(&snapshot, chunk_start_pos),
                        WalVersion::V2 => verifier.verify_and_update_v2(&snapshot, chunk_start_pos),
                        WalVersion::V1 => {
                            verifier.skip_hmac_verify_legacy(&snapshot);
                            Ok(())
                        }
                    };

                    if let Err(e) = verify_res {
                        if !using_legacy_key && allow_legacy_integrity_key_fallback {
                            let legacy_key = legacy_integrity_key()?;
                            let mut legacy_verifier = IntegrityVerifier::new(&legacy_key);
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
                                    target: "contextra_store::wal::legacy_fallback",
                                    wal_path = %path.display(),
                                    event = "legacy_integrity_key_fallback_used",
                                    "Legacy-WAL-Integritätsschlüssel aktiv für Segment {} — dieses Segment hat keine reale Manipulationssicherheit, da der Rückfallschlüssel öffentlich im Quellcode liegt.",
                                    path.display()
                                );
                                verifier = legacy_verifier;
                                using_legacy_key = true;
                                legacy_key_used.store(true, std::sync::atomic::Ordering::SeqCst);
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
                let entry_data = if let Some(km) = key_manager {
                    if entry_data_raw.len() < 12 {
                        return Err(StoreError::Storage(
                            "WAL entry too short for nonce".into(),
                        ));
                    }
                    let nonce_slice = match entry_data_raw.get(0..12) {
                        Some(s) => s,
                        None => {
                            return Err(StoreError::wal_corruption(
                                chunk_start_pos,
                                "WAL entry too short for nonce",
                            ));
                        }
                    };
                    let mut nonce = [0u8; 12];
                    nonce.copy_from_slice(nonce_slice);
                    let ciphertext = entry_data_raw.get(12..).ok_or_else(|| {
                        StoreError::wal_corruption(
                            chunk_start_pos,
                            "WAL entry missing ciphertext",
                        )
                    })?;
                    decrypted_data = match km.decrypt_auto_nonce(ciphertext, &nonce) {
                        Ok(data) => data,
                        Err(e) => {
                            if version == WalVersion::V1 {
                                return Err(StoreError::Storage(format!(
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
                                    break;
                                }
                                return Err(StoreError::wal_corruption(
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

                let entry = match WalEntry::from_bytes_classified(entry_data) {
                    Ok(e) => e,
                    Err(parse_err) => {
                        if key_manager.is_some() && version != WalVersion::V1 {
                            return Err(StoreError::wal_corruption(
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

                let verify_res = match version {
                    WalVersion::V3 => verifier.verify_and_update_v3(&snapshot, chunk_start_pos),
                    WalVersion::V2 => verifier.verify_and_update_v2(&snapshot, chunk_start_pos),
                    WalVersion::V1 => {
                        verifier.skip_hmac_verify_legacy(&snapshot);
                        Ok(())
                    }
                };

                if let Err(e) = verify_res {
                    if !using_legacy_key && allow_legacy_integrity_key_fallback {
                        let legacy_key = legacy_integrity_key()?;
                        let mut legacy_verifier = IntegrityVerifier::new(&legacy_key);
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
                                target: "contextra_store::wal::legacy_fallback",
                                wal_path = %path.display(),
                                event = "legacy_integrity_key_fallback_used",
                                "Legacy-WAL-Integritätsschlüssel aktiv für Segment {} — dieses Segment hat keine reale Manipulationssicherheit, da der Rückfallschlüssel öffentlich im Quellcode liegt.",
                                path.display()
                            );
                            verifier = legacy_verifier;
                            using_legacy_key = true;
                            legacy_key_used.store(true, std::sync::atomic::Ordering::SeqCst);
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
    }

    Ok(version)
}
