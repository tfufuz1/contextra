//! FILE-CONTEXT: STAND/ZWECK/INVARIANTEN
//! Stand: 2026-10-17
//! Zweck: WAL-Scope Purge (LN-3.1) - Selektive Bereinigung von gelöschten Schlüsseln/Werten aus WAL-Segmenten.
//! Invarianten:
//! - Schreiben neuer Segmente ohne Frames des Scopes (tx <= as_of_tx).
//! - HMAC-Hash-Kette der verbleibenden Frames wird neu berechnet.
//! - Atomarer Commit: neue Datei fsync -> rename -> Parent Dir fsync -> altes Segment durabel entfernen/scrubben.
//! - Tail hinter dem logischen Ende wird verworfen/genullt.
//! - Failpoint-/Crash-Sicherheit über durablen Intent-Marker (`wal_purge.intent`).

use contextra_durable_fs::atomic_replace;
use contextra_types::{ContextraError, Result};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

use crate::compaction::purge_filter::PurgeScope;
use crate::util::fsync_parent_dir;
use crate::wal::{
    classify_wal_filename, KeyManager, Wal, WalEntry, WalFileType, WalOp, WAL_V3_HEADER,
};

pub static WAL_PURGE_FAILPOINT: AtomicU32 = AtomicU32::new(0);

pub const FAILPOINT_BEFORE_REWRITE: u32 = 1;
pub const FAILPOINT_AFTER_REWRITE: u32 = 2;
pub const FAILPOINT_AFTER_RENAME: u32 = 3;
pub const FAILPOINT_AFTER_OLD_REMOVAL: u32 = 4;

const WAL_INTENT_MAGIC: &[u8; 4] = b"WPRG";
const WAL_INTENT_VERSION: u8 = 1;

/// Durable intent representing an active WAL purge operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrxWalPurgeIntent {
    pub purge_id: [u8; 16],
    pub scope_hash: [u8; 32],
    pub as_of_tx: u64,
}

impl OrxWalPurgeIntent {
    pub fn new(purge_id: [u8; 16], scope: &PurgeScope, as_of_tx: u64) -> Self {
        let mut hasher = blake3::Hasher::new();
        match scope {
            PurgeScope::Keys(keys) => {
                hasher.update(b"KEYS:");
                for key in keys {
                    hasher.update(&(key.len() as u64).to_le_bytes());
                    hasher.update(key);
                }
            }
            PurgeScope::Prefixes(prefixes) => {
                hasher.update(b"PREFIXES:");
                for prefix in prefixes {
                    hasher.update(&(prefix.len() as u64).to_le_bytes());
                    hasher.update(prefix);
                }
            }
        }
        let scope_hash = *hasher.finalize().as_bytes();
        Self {
            purge_id,
            scope_hash,
            as_of_tx,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(16 + 32 + 8);
        buf.extend_from_slice(&self.purge_id);
        buf.extend_from_slice(&self.scope_hash);
        buf.extend_from_slice(&self.as_of_tx.to_le_bytes());
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 + 32 + 8 {
            return Err(ContextraError::Storage(
                "WAL purge intent binary truncated".into(),
            ));
        }
        let mut purge_id = [0u8; 16];
        purge_id.copy_from_slice(&bytes[..16]);
        let mut scope_hash = [0u8; 32];
        scope_hash.copy_from_slice(&bytes[16..48]);
        let as_of_tx_bytes: [u8; 8] = bytes[48..56]
            .try_into()
            .map_err(|_| ContextraError::Storage("Invalid as_of_tx length".into()))?;
        let as_of_tx = u64::from_le_bytes(as_of_tx_bytes);
        Ok(Self {
            purge_id,
            scope_hash,
            as_of_tx,
        })
    }

    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let payload = self.to_bytes();
        let checksum = crc32fast::hash(&payload);

        let mut buffer = Vec::with_capacity(4 + 1 + 4 + 4 + payload.len());
        buffer.extend_from_slice(WAL_INTENT_MAGIC);
        buffer.push(WAL_INTENT_VERSION);
        buffer.extend_from_slice(&checksum.to_le_bytes());
        buffer.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&payload);

        atomic_replace(path, &buffer).map_err(|e| {
            ContextraError::Storage(format!("Failed to write WAL purge intent: {}", e))
        })
    }

    pub fn read_from_file(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let bytes = std::fs::read(path).map_err(|e| {
            ContextraError::Storage(format!("Failed to read WAL purge intent: {}", e))
        })?;

        if bytes.len() < 13 {
            return Err(ContextraError::Storage(
                "WAL purge intent header too short".into(),
            ));
        }

        if &bytes[0..4] != WAL_INTENT_MAGIC {
            return Err(ContextraError::Storage(
                "Invalid WAL purge intent magic header".into(),
            ));
        }

        if bytes[4] != WAL_INTENT_VERSION {
            return Err(ContextraError::Storage(format!(
                "Unsupported WAL purge intent version: {}",
                bytes[4]
            )));
        }

        let checksum_bytes: [u8; 4] = bytes[5..9]
            .try_into()
            .map_err(|_| ContextraError::Storage("Invalid checksum length".into()))?;
        let expected_checksum = u32::from_le_bytes(checksum_bytes);

        let len_bytes: [u8; 4] = bytes[9..13]
            .try_into()
            .map_err(|_| ContextraError::Storage("Invalid payload len length".into()))?;
        let payload_len = u32::from_le_bytes(len_bytes) as usize;

        if bytes.len() < 13 + payload_len {
            return Err(ContextraError::Storage(
                "WAL purge intent payload truncated".into(),
            ));
        }

        let payload = &bytes[13..13 + payload_len];
        if crc32fast::hash(payload) != expected_checksum {
            return Err(ContextraError::Storage(
                "WAL purge intent CRC32 checksum mismatch".into(),
            ));
        }

        let intent = Self::from_bytes(payload)?;
        Ok(Some(intent))
    }
}

/// Report summarizing results of a WAL scope purge operation.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WalPurgeReport {
    pub segments_evaluated: u64,
    pub segments_rewritten: u64,
    pub frames_purged: u64,
    pub tail_bytes_cleaned: u64,
}

/// Purges WAL entries matching `scope` with `tx <= as_of_tx` across all segments in `wal_dir`.
pub async fn purge_wal_scope(
    wal_dir: &Path,
    scope: &PurgeScope,
    as_of_tx: u64,
    key_manager: Option<Arc<KeyManager>>,
) -> Result<WalPurgeReport> {
    resume_wal_scope_purge(wal_dir).await?;

    let intent_path = wal_dir.join("wal_purge.intent");
    let mut rng_bytes = [0u8; 16];
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut rng_bytes);

    let intent = OrxWalPurgeIntent::new(rng_bytes, scope, as_of_tx);
    intent.write_to_file(&intent_path)?;
    fsync_parent_dir(&intent_path).await?;

    let mut report = WalPurgeReport::default();

    if check_and_clear_failpoint(FAILPOINT_BEFORE_REWRITE) {
        return Err(ContextraError::Storage(
            "Simulated failpoint: FAILPOINT_BEFORE_REWRITE".into(),
        ));
    }

    let mut dir_entries = match tokio::fs::read_dir(wal_dir).await {
        Ok(e) => e,
        Err(e) => {
            return Err(ContextraError::Storage(format!(
                "Failed to read WAL directory {}: {}",
                wal_dir.display(),
                e
            )));
        }
    };

    let mut candidates = Vec::new();
    while let Ok(Some(entry)) = dir_entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let filename = match path.file_name().and_then(|f| f.to_str()) {
            Some(f) => f,
            None => continue,
        };

        let file_type = classify_wal_filename(filename);
        if matches!(
            file_type,
            WalFileType::Active | WalFileType::Sealed | WalFileType::Archived | WalFileType::Backup
        ) {
            candidates.push(path);
        }
    }

    for path in candidates {
        report.segments_evaluated += 1;

        let wal = Wal::open_read_only(&path, key_manager.clone()).await?;
        let entries = match wal.replay().await {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!(
                    wal_path = %path.display(),
                    error = %err,
                    "Failed to replay WAL segment during scope purge evaluation"
                );
                continue;
            }
        };

        let metadata = tokio::fs::metadata(&path).await.map_err(|e| {
            ContextraError::Storage(format!("Failed to stat segment {}: {}", path.display(), e))
        })?;
        let physical_file_len = metadata.len();

        let mut last_logical_pos = 0u64;
        let mut has_matching_frames = false;
        let mut matching_count = 0u64;

        for (_, entry, end_pos) in &entries {
            last_logical_pos = *end_pos;
            let tx = entry.tx_id().inner();
            if tx <= as_of_tx {
                let matches = match &entry.op {
                    WalOp::Put { key, .. } => scope.matches_key(key),
                    WalOp::Delete { key, .. } => scope.matches_key(key),
                    WalOp::TxEnd { .. } => false,
                };
                if matches {
                    has_matching_frames = true;
                    matching_count += 1;
                }
            }
        }

        let tail_bytes = physical_file_len.saturating_sub(last_logical_pos);
        if !has_matching_frames && tail_bytes == 0 {
            continue;
        }

        // Rewrite required
        let tmp_path = path.with_extension("purge_tmp");

        let integrity_key = wal.integrity_key_for_test()?;

        let mut kept_entries = Vec::new();
        for (_, entry, _) in &entries {
            let tx = entry.tx_id().inner();
            let should_drop = if tx <= as_of_tx {
                match &entry.op {
                    WalOp::Put { key, .. } => scope.matches_key(key),
                    WalOp::Delete { key, .. } => scope.matches_key(key),
                    WalOp::TxEnd { .. } => false,
                }
            } else {
                false
            };

            if !should_drop {
                kept_entries.push(entry.clone());
            }
        }

        // Re-encode kept entries with fresh HMAC chain and monotonic sequence numbers
        let mut v3_entries = Vec::with_capacity(kept_entries.len());
        let mut prev_hmac = [0u8; 32];
        let mut current_seq = kept_entries.first().map_or(1, |e| e.seq_no);
        for entry in kept_entries {
            let v3_entry = WalEntry::try_new(entry.op, current_seq, &integrity_key, prev_hmac)?;
            prev_hmac = v3_entry.checksum;
            v3_entries.push(v3_entry);
            current_seq += 1;
        }

        let mut total_bytes = Vec::new();
        total_bytes.extend_from_slice(&WAL_V3_HEADER);

        if let Some(km) = &key_manager {
            let mut current_batch = Vec::new();
            for entry in &v3_entries {
                let entry_bytes = entry.to_bytes()?;
                current_batch.extend_from_slice(&entry_bytes);
            }
            if !current_batch.is_empty() {
                let (encrypted, nonce) = km.encrypt_auto_nonce(&current_batch)?;
                let chunk_len = u32::try_from(12 + encrypted.len())
                    .map_err(|e| ContextraError::Storage(e.to_string()))?;
                total_bytes.extend_from_slice(&chunk_len.to_le_bytes());
                total_bytes.extend_from_slice(&nonce);
                total_bytes.extend_from_slice(&encrypted);
            }
        } else {
            for entry in &v3_entries {
                let bytes = entry.to_bytes()?;
                total_bytes.extend_from_slice(&bytes);
            }
        }

        let mut tmp_file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .await
            .map_err(|e| {
                ContextraError::Storage(format!(
                    "Failed to create WAL purge tmp file {}: {}",
                    tmp_path.display(),
                    e
                ))
            })?;

        tmp_file.write_all(&total_bytes).await.map_err(|e| {
            ContextraError::Storage(format!("Failed to write WAL purge tmp payload: {}", e))
        })?;

        tmp_file.sync_all().await.map_err(|e| {
            ContextraError::Storage(format!("Failed to sync WAL purge tmp file: {}", e))
        })?;
        drop(tmp_file);

        if check_and_clear_failpoint(FAILPOINT_AFTER_REWRITE) {
            return Err(ContextraError::Storage(
                "Simulated failpoint: FAILPOINT_AFTER_REWRITE".into(),
            ));
        }

        // Atomic replace: rename tmp -> original
        tokio::fs::rename(&tmp_path, &path).await.map_err(|e| {
            ContextraError::Storage(format!(
                "Failed to rename WAL purge tmp file from {} to {}: {}",
                tmp_path.display(),
                path.display(),
                e
            ))
        })?;

        fsync_parent_dir(&path).await?;

        if check_and_clear_failpoint(FAILPOINT_AFTER_RENAME) {
            return Err(ContextraError::Storage(
                "Simulated failpoint: FAILPOINT_AFTER_RENAME".into(),
            ));
        }

        report.segments_rewritten += 1;
        report.frames_purged += matching_count;
        report.tail_bytes_cleaned += tail_bytes;
    }

    if check_and_clear_failpoint(FAILPOINT_AFTER_OLD_REMOVAL) {
        return Err(ContextraError::Storage(
            "Simulated failpoint: FAILPOINT_AFTER_OLD_REMOVAL".into(),
        ));
    }

    if intent_path.exists() {
        let _ = tokio::fs::remove_file(&intent_path).await;
        fsync_parent_dir(&intent_path).await?;
    }

    Ok(report)
}

/// Resumes/recovers any interrupted WAL scope purge operation by cleaning up orphan `.purge_tmp` files.
pub async fn resume_wal_scope_purge(wal_dir: &Path) -> Result<()> {
    let intent_path = wal_dir.join("wal_purge.intent");
    let mut dir_entries = match tokio::fs::read_dir(wal_dir).await {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    while let Ok(Some(entry)) = dir_entries.next_entry().await {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "purge_tmp") {
            let _ = tokio::fs::remove_file(&path).await;
        }
    }

    if intent_path.exists() {
        let _ = tokio::fs::remove_file(&intent_path).await;
        fsync_parent_dir(&intent_path).await?;
    }

    Ok(())
}

fn check_and_clear_failpoint(fp: u32) -> bool {
    WAL_PURGE_FAILPOINT
        .compare_exchange(fp, 0, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
}
