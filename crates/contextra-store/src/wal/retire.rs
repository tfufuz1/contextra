//! FILE-CONTEXT: STAND/ZWECK/INVARIANTEN
//! Stand: 2026-10-21
//! Zweck: Segment-Entsorgung (C5.5) fuer WAL-Checkpointing, Scope Purge und Storage-Housekeeping.
//! Invarianten:
//! - Segmente werden nur entfernt, wenn max_tx <= checkpoint_tx oder im Scope-Purge ueberholte Segmente.
//! - ANKER ZUERST: Manifest-WalCheckpoint{hmac} muss durabel geschrieben werden vor Segment-Modifikation/Entsorgung.
//! - Scrubbing ueber scrub_if_safe (gleiche Disposal-Policy wie Phase 5).
//! - Archived / Backup Segmente erfordern explizite Freigabe (PurgeMode::PurgeWithScrub) sonst Blocked/PolicyViolation Error.

#![cfg(not(loom))]

use contextra_durable_fs::{scrub_if_safe, ScrubOutcome, ScrubPolicy};
use contextra_types::{ContextraError, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::compaction::purge_filter::PurgeScope;
use crate::manifest::{Manifest, ManifestEntry};
use crate::util::fsync_parent_dir;

use super::{
    classify_wal_filename, purge_wal_scope, KeyManager, Wal, WalFileType, WalPurgeReport,
};

/// Purge mode specifying physical cleanup requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WalPurgeMode {
    None,
    Purge,
    PurgeWithScrub,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WalRetireReport {
    pub segments_evaluated: u64,
    pub segments_rotated: u64,
    pub segments_removed: u64,
    pub last_removed_seq: u64,
    pub bytes_removed: u64,
    pub purge_report: WalPurgeReport,
}

#[derive(Debug, Clone)]
pub struct WalDirEntry {
    pub path: PathBuf,
    pub is_dir: bool,
}

#[allow(async_fn_in_trait)]
pub trait WalDirStream {
    async fn next_entry(&mut self) -> Result<Option<WalDirEntry>>;
}

pub struct TokioWalDirStream {
    inner: tokio::fs::ReadDir,
    wal_dir: PathBuf,
}

impl TokioWalDirStream {
    pub fn new(dir: tokio::fs::ReadDir, wal_dir: &Path) -> Self {
        Self {
            inner: dir,
            wal_dir: wal_dir.to_path_buf(),
        }
    }
}

impl WalDirStream for TokioWalDirStream {
    async fn next_entry(&mut self) -> Result<Option<WalDirEntry>> {
        match self.inner.next_entry().await {
            Ok(Some(entry)) => {
                let path = entry.path();
                let is_dir = path.is_dir();
                Ok(Some(WalDirEntry { path, is_dir }))
            }
            Ok(None) => Ok(None),
            Err(err) => Err(ContextraError::Storage(format!(
                "Failed to read WAL directory entry in {}: {}",
                self.wal_dir.display(),
                err
            ))),
        }
    }
}

pub async fn retire_segments_up_to(wal_dir: &Path, checkpoint_tx: u64) -> Result<WalRetireReport> {
    let entries = match tokio::fs::read_dir(wal_dir).await {
        Ok(e) => e,
        Err(err) => {
            return Err(ContextraError::Storage(format!(
                "Failed to read WAL directory {}: {}",
                wal_dir.display(),
                err
            )));
        }
    };

    let stream = TokioWalDirStream::new(entries, wal_dir);
    retire_segments_with_stream(stream, checkpoint_tx, ScrubPolicy::OverwriteZeros).await
}

pub async fn retire_segments_with_stream<S: WalDirStream>(
    mut stream: S,
    checkpoint_tx: u64,
    scrub_policy: ScrubPolicy,
) -> Result<WalRetireReport> {
    let mut report = WalRetireReport::default();
    let mut candidates = Vec::new();

    loop {
        let entry = match stream.next_entry().await? {
            Some(e) => e,
            None => break,
        };

        if entry.is_dir {
            continue;
        }

        let filename = match entry.path.file_name().and_then(|f| f.to_str()) {
            Some(f) => f,
            None => continue,
        };

        let file_type = classify_wal_filename(filename);
        match file_type {
            WalFileType::Active
            | WalFileType::Sealed
            | WalFileType::Archived
            | WalFileType::Backup => {
                candidates.push((entry.path, file_type));
            }
            WalFileType::Sidecar | WalFileType::Unclassified => {}
        }
    }

    for (path, file_type) in candidates {
        report.segments_evaluated += 1;

        let metadata = match tokio::fs::metadata(&path).await {
            Ok(m) => m,
            Err(err) => {
                return Err(ContextraError::Storage(format!(
                    "Failed to stat WAL candidate {}: {}",
                    path.display(),
                    err
                )));
            }
        };
        let file_len = metadata.len();

        let wal = Wal::open_read_only(&path, None).await?;
        let entries_res = wal.replay().await;

        let (entries_list, max_tx_in_seg, max_seq_in_seg) = match entries_res {
            Ok(list) => {
                let mut max_tx = 0u64;
                let mut max_seq = 0u64;
                for (seq, entry, _) in &list {
                    let tx = entry.tx_id().inner();
                    if tx > max_tx {
                        max_tx = tx;
                    }
                    if *seq > max_seq {
                        max_seq = *seq;
                    }
                }
                (list, max_tx, max_seq)
            }
            Err(err) => {
                tracing::warn!(
                    wal_path = %path.display(),
                    error = %err,
                    "Failed replay on WAL segment during retirement check"
                );
                return Err(ContextraError::Storage(format!(
                    "Failed replay on WAL segment {} during retirement check: {}",
                    path.display(),
                    err
                )));
            }
        };

        if entries_list.is_empty() {
            // Empty segment is safe to remove if file exists
            remove_segment_file(&path, scrub_policy).await?;
            report.segments_removed += 1;
            report.bytes_removed += file_len;
            continue;
        }

        if max_tx_in_seg <= checkpoint_tx {
            let target_path = if file_type == WalFileType::Active {
                // Active segment must be rotated and sealed before removal
                let rw_wal = Wal::open(&path).await?;
                let sealed_path = rw_wal.rotate_and_seal().await?;
                report.segments_rotated += 1;
                sealed_path
            } else {
                path
            };

            remove_segment_file(&target_path, scrub_policy).await?;
            report.segments_removed += 1;
            report.bytes_removed += file_len;
            if max_seq_in_seg > report.last_removed_seq {
                report.last_removed_seq = max_seq_in_seg;
            }
        }
    }

    Ok(report)
}

/// Complete high-level WAL retirement function for a given scope and checkpoint state.
///
/// Steps:
/// (1) Rotate/seal active segment if candidate frames exist;
/// (2) **ANKER ZUERST**: Manifest-`WalCheckpoint{hmac}` written durably before modification/removal;
/// (3) Fully obsoleted segments retired via `retire_segments_up_to` with scrubbing (`scrub_if_safe`);
/// (4) Mixed segments scope-purged via `purge_wal_scope` (re-encoding without scope frames and re-chaining HMAC);
/// (5) Archived/Backup classes require explicit `WalPurgeMode::PurgeWithScrub` (returning `PolicyViolation` error otherwise).
pub async fn retire_for_scope(
    wal_dir: &Path,
    manifest: &Manifest,
    scope: &PurgeScope,
    as_of_tx: u64,
    checkpoint_tx: u64,
    key_manager: Option<Arc<KeyManager>>,
    mode: WalPurgeMode,
    scrub_policy: ScrubPolicy,
) -> Result<WalRetireReport> {
    // 5. Check Archived/Backup classes
    let mut dir_entries = tokio::fs::read_dir(wal_dir).await.map_err(|e| {
        ContextraError::Storage(format!("Failed to read WAL dir {}: {}", wal_dir.display(), e))
    })?;

    while let Ok(Some(entry)) = dir_entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
            let file_type = classify_wal_filename(filename);
            if matches!(file_type, WalFileType::Archived | WalFileType::Backup) {
                if mode != WalPurgeMode::PurgeWithScrub {
                    return Err(ContextraError::PolicyViolation(format!(
                        "Refusing to retire archived/backup WAL segment {:?} without explicit PurgeWithScrub mode",
                        path
                    )));
                }
            }
        }
    }

    // Determine current tail HMAC across all WAL segments for anchor
    let mut last_hmac = [0u8; 32];
    let mut active_path_opt: Option<PathBuf> = None;

    let mut entries_scan = tokio::fs::read_dir(wal_dir).await.map_err(|e| {
        ContextraError::Storage(format!("Failed to read WAL dir {}: {}", wal_dir.display(), e))
    })?;

    let mut all_files = Vec::new();
    while let Ok(Some(entry)) = entries_scan.next_entry().await {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
            let ft = classify_wal_filename(filename);
            if matches!(ft, WalFileType::Active | WalFileType::Sealed) {
                all_files.push((path, ft));
            }
        }
    }
    all_files.sort_by(|(p1, _), (p2, _)| p1.cmp(p2));

    for (path, ft) in &all_files {
        if *ft == WalFileType::Active {
            active_path_opt = Some(path.clone());
        }
        if let Ok(wal) = Wal::open_read_only(path, key_manager.clone()).await {
            if let Ok(replayed) = wal.replay().await {
                if let Some((_, last_entry, _)) = replayed.last() {
                    last_hmac = last_entry.checksum;
                }
            }
        }
    }

    // 1. Seal active segment if present
    if let Some(active_path) = active_path_opt {
        if let Ok(rw_wal) = Wal::open_with_key_manager(&active_path, key_manager.clone()).await {
            if let Err(err) = rw_wal.rotate_and_seal().await {
                tracing::warn!(wal_path = %active_path.display(), error = %err, "Failed to rotate and seal active WAL segment during retire_for_scope");
            }
        }
    }

    // 2. ANKER ZUERST: Manifest-WalCheckpoint durable write before any removals
    manifest
        .append(&ManifestEntry::WalCheckpoint { hmac: last_hmac })
        .await?;

    // 3. Fully obsoleted segments retired with scrub
    let mut retire_report = retire_segments_up_to(wal_dir, checkpoint_tx).await?;

    // 4. Mixed segments scope purge
    let purge_report = purge_wal_scope(
        wal_dir,
        scope,
        as_of_tx,
        key_manager,
        #[cfg(any(test, feature = "failpoints", feature = "fault-injection"))]
        None,
    )
    .await?;

    retire_report.purge_report = purge_report;

    let _ = scrub_policy; // INTENTIONAL-DROP: Reserved for custom scrub policy pass-through in phase 2.

    Ok(retire_report)
}

async fn remove_segment_file(path: &Path, scrub_policy: ScrubPolicy) -> Result<()> {
    let p = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let outcome = scrub_if_safe(&p, scrub_policy, false, false).map_err(|e| {
            ContextraError::Storage(format!(
                "Failed to scrub and remove WAL segment {}: {}",
                p.display(),
                e
            ))
        })?;
        match outcome {
            ScrubOutcome::Scrubbed(receipt) => {
                tracing::debug!(
                    path = %p.display(),
                    method = ?receipt.method,
                    bytes_scrubbed = receipt.bytes_scrubbed,
                    "WAL segment scrubbed and removed successfully"
                );
            }
            ScrubOutcome::RemovedUnscrubbed => {
                tracing::debug!(
                    path = %p.display(),
                    "WAL segment removed unscrubbed"
                );
            }
            ScrubOutcome::Refused(reason) => {
                tracing::warn!(
                    path = %p.display(),
                    reason = ?reason,
                    "WAL segment scrub refused; falling back to durable remove"
                );
                contextra_durable_fs::durable_remove(&p).map_err(|e| {
                    ContextraError::Storage(format!(
                        "Failed to durably remove WAL segment {}: {}",
                        p.display(),
                        e
                    ))
                })?;
            }
        }
        Ok::<(), ContextraError>(())
    })
    .await
    .map_err(|e| ContextraError::Storage(format!("spawn_blocking join error: {}", e)))??;

    fsync_parent_dir(path).await?;
    Ok(())
}
