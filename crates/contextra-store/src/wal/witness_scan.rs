#![cfg(not(loom))]

//! FILE-CONTEXT: STAND/ZWECK/INVARIANTEN
//! Stand: 2026-10-13
//! Zweck: WAL-Witness Scanner (C4.3) fuer Assurance & Claim Verification (Interface-Vertrag V1).
//! Invarianten:
//! - segments_scanned == segments_total - segments_removed
//! - frames_matching_scope == 0

use contextra_ports::{Clock, SystemClock};
use contextra_types::{ContextraError, Result};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use super::{WalEntry, WalOp, WAL_V2_HEADER, WAL_V3_HEADER};

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
    let timeout_nanos = u64::try_from(inputs.timeout.as_nanos()).unwrap_or(u64::MAX);

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
        let file_bytes = match tokio::fs::read(&path).await {
            Ok(b) => b,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    continue;
                }
                return Err(ContextraError::Storage(format!(
                    "Failed to read WAL segment {}: {}",
                    path.display(),
                    err
                )));
            }
        };

        let mut last_logical_pos = 0u64;
        let scan_res = parse_wal_bytes_independently(&file_bytes, |entry, end_pos| {
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
        });

        if scan_res.is_err() {
            // Unreadable or corrupted segment -> report finding
            frames_matching_scope += 1;
        }

        // Tail inspection past logical endpoint
        if last_logical_pos < file_len {
            let tail_len = file_len - last_logical_pos;
            tail_bytes_raw_scanned += tail_len;
            let tail_start = last_logical_pos as usize;
            if tail_start < file_bytes.len() {
                let tail_bytes = &file_bytes[tail_start..];
                if inputs.scope.matches_raw_bytes(tail_bytes) {
                    frames_matching_scope += 1;
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

    let segments_removed = u64::try_from(
        initial_segment_names
            .iter()
            .filter(|name| !final_segment_names.contains(*name))
            .count(),
    )
    .unwrap_or(u64::MAX);

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

/// Independent frame parser for verifying WAL file contents without relying on production `Wal` handle logic.
pub fn parse_wal_bytes_independently<F>(bytes: &[u8], mut callback: F) -> Result<()>
where
    F: FnMut(WalEntry, u64) -> bool,
{
    if bytes.is_empty() {
        return Ok(());
    }

    let mut pos = 0usize;
    if bytes.len() >= 4 {
        let header = &bytes[..4];
        if header == WAL_V3_HEADER || header == WAL_V2_HEADER {
            pos = 4;
        }
    }

    while pos < bytes.len() {
        if pos + 4 > bytes.len() {
            // Truncated length header at tail
            break;
        }
        let len_bytes: [u8; 4] = match bytes[pos..pos + 4].try_into() {
            Ok(b) => b,
            Err(_) => break,
        };
        let len = u32::from_le_bytes(len_bytes) as usize;
        if len == 0 || pos + 4 + len > bytes.len() {
            // Truncated or invalid frame payload at tail
            break;
        }

        let frame_payload = &bytes[pos + 4..pos + 4 + len];
        pos += 4 + len;

        let entry = match WalEntry::from_bytes_classified(frame_payload) {
            Ok(e) => e,
            Err(_) => {
                // If it's encrypted or corrupt, try searching for raw keys or treat as non-parseable
                break;
            }
        };

        if !callback(entry, pos as u64) {
            break;
        }
    }

    Ok(())
}
