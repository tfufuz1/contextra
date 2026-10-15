#![allow(clippy::cast_possible_truncation)]

//! FILE-CONTEXT: STAND/ZWECK/INVARIANTEN
//! Stand: 2026-10-13
//! Zweck: WAL-Witness Scanner (C4.3) fuer Assurance & Claim Verification (Interface-Vertrag V1).
//! Invarianten:
//! - segments_scanned == segments_total - segments_removed
//! - frames_matching_scope == 0

use contextra_core::{ContextraError, Result};
use contextra_ports::{Clock, SystemClock};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{Wal, WalOp};

pub const ASSURANCE_LEVEL: u8 = 4;
pub const CLAIM_TEXT_ID: u8 = 5;
pub const LAYER_ID_WAL_ALL_SEGMENTS: u8 = 4;

#[derive(Debug, Clone, Default)]
pub struct WalScopeSpec {
    pub key_prefixes: Vec<Vec<u8>>,
    pub raw_needles: Vec<Vec<u8>>,
}

impl WalScopeSpec {
    pub fn matches_key(&self, key: &[u8]) -> bool {
        if self.key_prefixes.is_empty() {
            return true;
        }
        self.key_prefixes
            .iter()
            .any(|prefix| key.starts_with(prefix))
    }

    pub fn matches_raw_bytes(&self, bytes: &[u8]) -> bool {
        if self.raw_needles.is_empty() {
            return false;
        }
        self.raw_needles.iter().any(|needle| {
            !needle.is_empty() && bytes.windows(needle.len()).any(|w| w == needle.as_slice())
        })
    }
}

#[derive(Debug, Clone)]
pub struct WalScanInputs {
    pub wal_dir: PathBuf,
    pub as_of_tx: u64,
    pub encrypted_mode: bool,
    pub timeout: Duration,
    pub scope: WalScopeSpec,
    pub key_identifiers_claimed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalScanEvidence {
    pub segments_total: u64,
    pub segments_scanned: u64,
    pub segments_removed: u64,
    pub tail_bytes_raw_scanned: u64,
    pub frames_matching_scope: u64,
    pub post_cutoff_hits: u64,
    pub last_removed_seq: u64,
    pub unclassified_files_found: u64,
}

impl WalScanEvidence {
    pub fn layer_id(&self) -> u8 {
        LAYER_ID_WAL_ALL_SEGMENTS
    }

    pub fn assurance_level(&self) -> u8 {
        ASSURANCE_LEVEL
    }

    pub fn claim_text_id(&self) -> u8 {
        CLAIM_TEXT_ID
    }

    pub fn invariant_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        if self.segments_scanned != self.segments_total.saturating_sub(self.segments_removed) {
            violations.push(format!(
                "Invariant violation: segments_scanned ({}) != segments_total ({}) - segments_removed ({})",
                self.segments_scanned, self.segments_total, self.segments_removed
            ));
        }
        if self.frames_matching_scope != 0 {
            violations.push(format!(
                "Invariant violation: frames_matching_scope ({}) != 0",
                self.frames_matching_scope
            ));
        }
        if self.post_cutoff_hits != 0 {
            violations.push(format!(
                "Post-cutoff write hits present ({}); scope key modified after deletion cutoff",
                self.post_cutoff_hits
            ));
        }
        if self.unclassified_files_found != 0 {
            violations.push(format!(
                "Unclassified files present ({}); claim invalidated",
                self.unclassified_files_found
            ));
        }
        violations
    }

    pub fn is_valid(&self) -> bool {
        self.invariant_violations().is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalFileType {
    Active,
    Sealed,
    Archived,
    Backup,
    Sidecar,
    Unclassified,
}

pub fn classify_wal_filename(filename: &str) -> WalFileType {
    if matches!(
        filename,
        "MANIFEST"
            | "SALT"
            | "LOCK"
            | "purge.intent"
            | "pending_delete.journal"
            | ".wal_integrity_key"
            | "integrity.key"
            | "erasure.ledger"
            | "ledger.head"
    ) || filename.ends_with(".rekeyed")
        || filename.ends_with(".uuid")
        || filename.ends_with(".ledger")
        || filename.ends_with(".head")
        || filename.ends_with(".key")
    {
        WalFileType::Sidecar
    } else if filename.contains(".sealed.") || filename.ends_with(".sealed") {
        WalFileType::Sealed
    } else if filename.contains(".archived") || filename.ends_with(".archived") {
        WalFileType::Archived
    } else if filename.ends_with(".bak") || filename.contains(".bak.") {
        WalFileType::Backup
    } else if filename.ends_with(".wal")
        || filename.ends_with(".log")
        || filename.starts_with("wal")
    {
        WalFileType::Active
    } else {
        WalFileType::Unclassified
    }
}

pub async fn scan_wal_directory(inputs: &WalScanInputs) -> Result<WalScanEvidence> {
    scan_wal_directory_with_clock(inputs, &SystemClock::new()).await
}

pub async fn scan_wal_directory_with_clock(
    inputs: &WalScanInputs,
    clock: &dyn Clock,
) -> Result<WalScanEvidence> {
    let start_nanos = clock.monotonic_nanos();
    let timeout_nanos = inputs.timeout.as_nanos() as u64;

    let mut entries = match tokio::fs::read_dir(&inputs.wal_dir).await {
        Ok(e) => e,
        Err(err) => {
            return Err(ContextraError::Storage(format!(
                "Failed to read WAL directory {}: {}",
                inputs.wal_dir.display(),
                err
            )));
        }
    };

    let mut segments_total = 0u64;
    let mut segments_scanned = 0u64;
    let mut tail_bytes_raw_scanned = 0u64;
    let mut frames_matching_scope = 0u64;
    let mut post_cutoff_hits = 0u64;
    let mut unclassified_files_found = 0u64;

    let mut initial_segment_names = HashSet::new();
    let mut paths_to_scan = Vec::new();

    while let Some(entry_res) = entries.next_entry().await.transpose() {
        let entry = entry_res.map_err(|e| {
            ContextraError::Storage(format!(
                "Failed to read directory entry in {}: {}",
                inputs.wal_dir.display(),
                e
            ))
        })?;

        if clock.monotonic_nanos().saturating_sub(start_nanos) > timeout_nanos {
            return Err(ContextraError::Storage("WAL scan timed out".into()));
        }

        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let filename = match path.file_name().and_then(|f| f.to_str()) {
            Some(f) => f,
            None => continue,
        };

        let file_type = classify_wal_filename(filename);
        match file_type {
            WalFileType::Sidecar => {}
            WalFileType::Unclassified => {
                unclassified_files_found += 1;
            }
            WalFileType::Active
            | WalFileType::Sealed
            | WalFileType::Archived
            | WalFileType::Backup => {
                segments_total += 1;
                initial_segment_names.insert(filename.to_string());
                paths_to_scan.push(path);
            }
        }
    }

    for path in paths_to_scan {
        if clock.monotonic_nanos().saturating_sub(start_nanos) > timeout_nanos {
            return Err(ContextraError::Storage("WAL scan timed out".into()));
        }

        let metadata = match tokio::fs::metadata(&path).await {
            Ok(m) => m,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    // Segment was removed prior to scanning
                    continue;
                }
                return Err(ContextraError::Storage(format!(
                    "Failed to stat WAL segment {}: {}",
                    path.display(),
                    err
                )));
            }
        };

        let file_len = metadata.len();
        let wal = match Wal::open_read_only(&path, None).await {
            Ok(w) => w,
            Err(ContextraError::Storage(msg))
                if msg.contains("No such file") || msg.contains("not found") =>
            {
                // Segment was removed prior to open
                continue;
            }
            Err(err) => return Err(err),
        };
        let mut last_logical_pos = 0u64;

        let scan_res = wal
            .scan_entries_with_callback(file_len, |_seq, entry, end_pos| {
                last_logical_pos = end_pos;
                let tx = entry.tx_id().inner();
                let is_hit = match &entry.op {
                    WalOp::Put { key, .. } => inputs.scope.matches_key(key),
                    WalOp::Delete { key, .. } => {
                        inputs.key_identifiers_claimed && inputs.scope.matches_key(key)
                    }
                    WalOp::TxEnd { .. } => false,
                };

                if is_hit {
                    frames_matching_scope += 1;
                    if tx > inputs.as_of_tx {
                        post_cutoff_hits += 1;
                    }
                }
                true
            })
            .await;

        if scan_res.is_err() {
            // Unreadable or corrupted segment -> report finding
            frames_matching_scope += 1;
        }

        // Tail inspection past logical endpoint
        if last_logical_pos < file_len {
            let tail_len = file_len - last_logical_pos;
            tail_bytes_raw_scanned += tail_len;
            if inputs.key_identifiers_claimed || !inputs.scope.raw_needles.is_empty() {
                if let Ok(tail_bytes) = read_file_tail(&path, last_logical_pos, tail_len).await {
                    if inputs.scope.matches_raw_bytes(&tail_bytes) {
                        frames_matching_scope += 1;
                    }
                }
            }
        }

        segments_scanned += 1;
    }

    // Directory check after scan to count removed segments
    let mut final_entries = match tokio::fs::read_dir(&inputs.wal_dir).await {
        Ok(e) => e,
        Err(err) => {
            return Err(ContextraError::Storage(format!(
                "Failed to read WAL directory during final verification {}: {}",
                inputs.wal_dir.display(),
                err
            )));
        }
    };

    let mut final_segment_names = HashSet::new();
    while let Some(entry_res) = final_entries.next_entry().await.transpose() {
        let entry = entry_res.map_err(|e| {
            ContextraError::Storage(format!(
                "Failed to read directory entry during final verification in {}: {}",
                inputs.wal_dir.display(),
                e
            ))
        })?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
            let file_type = classify_wal_filename(filename);
            if matches!(
                file_type,
                WalFileType::Active
                    | WalFileType::Sealed
                    | WalFileType::Archived
                    | WalFileType::Backup
            ) {
                final_segment_names.insert(filename.to_string());
            }
        }
    }

    let segments_removed = initial_segment_names
        .iter()
        .filter(|name| !final_segment_names.contains(*name))
        .count() as u64;

    Ok(WalScanEvidence {
        segments_total,
        segments_scanned,
        segments_removed,
        tail_bytes_raw_scanned,
        frames_matching_scope,
        post_cutoff_hits,
        last_removed_seq: 0,
        unclassified_files_found,
    })
}

async fn read_file_tail(path: &Path, start_pos: u64, tail_len: u64) -> Result<Vec<u8>> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    let mut file = super::fs::OpenOptions::new()
        .read(true)
        .open(path)
        .await
        .map_err(|e| ContextraError::Storage(format!("Failed to open for tail read: {}", e)))?;

    file.seek(std::io::SeekFrom::Start(start_pos))
        .await
        .map_err(|e| ContextraError::Storage(format!("Failed to seek for tail read: {}", e)))?;

    let buf_size = usize::try_from(tail_len).unwrap_or(0);
    let mut buf = vec![0u8; buf_size];
    file.read_exact(&mut buf)
        .await
        .map_err(|e| ContextraError::Storage(format!("Failed to read tail bytes: {}", e)))?;

    Ok(buf)
}
