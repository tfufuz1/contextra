//! FILE-CONTEXT: STAND/ZWECK/INVARIANTEN
//! Stand: 2026-10-11
//! Zweck: Segment-Entsorgung (C5.5) fuer WAL-Checkpointing und Storage-Housekeeping.
//! Invarianten:
//! - Segmente werden nur entfernt, wenn max_tx <= checkpoint_tx.
//! - Entfernung erfolgt ueber contextra_durable_fs::durable_remove in spawn_blocking.

#![cfg(not(loom))]

use contextra_types::{ContextraError, Result};
use std::path::{Path, PathBuf};

use super::{classify_wal_filename, Wal, WalFileType};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WalRetireReport {
    pub segments_evaluated: u64,
    pub segments_rotated: u64,
    pub segments_removed: u64,
    pub last_removed_seq: u64,
    pub bytes_removed: u64,
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
    retire_segments_with_stream(stream, checkpoint_tx).await
}

pub async fn retire_segments_with_stream<S: WalDirStream>(
    mut stream: S,
    checkpoint_tx: u64,
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
            remove_segment_file(&path).await?;
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

            remove_segment_file(&target_path).await?;
            report.segments_removed += 1;
            report.bytes_removed += file_len;
            if max_seq_in_seg > report.last_removed_seq {
                report.last_removed_seq = max_seq_in_seg;
            }
        }
    }

    Ok(report)
}

async fn remove_segment_file(path: &Path) -> Result<()> {
    let p = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        contextra_durable_fs::durable_remove(&p).map_err(|e| {
            ContextraError::Storage(format!(
                "Failed to durably remove WAL segment {}: {}",
                p.display(),
                e
            ))
        })
    })
    .await
    .map_err(|e| ContextraError::Storage(format!("spawn_blocking join error: {}", e)))?
}
