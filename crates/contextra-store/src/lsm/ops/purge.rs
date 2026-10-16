// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Implementation von LsmStorage::purge_keys (Compaction-basierte Bereinigung).
// INVARIANTEN: Active snapshot pin check, Disk space pre-flight probe, atomic manifest transitions, Zero-panic.

use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeReport, PurgeScope};
use crate::lsm::engine::LsmStorage;
use crate::manifest::ManifestEntry;
use crate::sstable::{SstableBuilder, SstableReader};
use contextra_ports::erasure::PurgeError;
use contextra_ports::StorageEngine;
use contextra_types::TOMBSTONE_BIT;
use std::sync::Arc;

impl LsmStorage {
    /// Führt Compaction-basierte physikalische Schlüssel- und Präfix-Bereinigung durch.
    ///
    /// Identifiziert SSTables, die Schlüssel aus `PurgeScope` enthalten,
    /// schreibt diese ohne die Purge-Schlüssel neu (oder entfernt sie, falls leer),
    /// führt einen atomaren Manifest-Swap durch und aktualisiert die In-Memory-SSTable-Liste.
    pub async fn purge_keys(
        &self,
        scope: PurgeScope,
        as_of_tx: u64,
        space_probe: Option<&dyn FreeSpaceProbe>,
    ) -> std::result::Result<PurgeReport, PurgeError> {
        // 1. Active snapshot pin check
        let min_active_seq = self.snapshot_registry.min_active_seqno();
        if min_active_seq <= as_of_tx {
            return Err(PurgeError::SnapshotPinned {
                oldest_pin_age: as_of_tx.saturating_sub(min_active_seq),
            });
        }

        // 2. Active Memtable Flush to SSTable first
        StorageEngine::flush(self)
            .await
            .map_err(|e| PurgeError::ProbeFailed(format!("Flush failed: {e}")))?;

        // Snapshot existing sstables list under read lock
        let snapshot_sstables = {
            let sst_guard = self.sstables.read().await;
            sst_guard.clone()
        };

        // Select candidate SSTables and verify disk space strictly over candidates (P-6)
        let sstables_to_rewrite = match crate::compaction::purge_select::select_purge_candidates(
            &snapshot_sstables,
            &scope,
            space_probe,
        ) {
            Ok(candidates) => candidates,
            Err(crate::compaction::purge_filter::PurgeBlocked::InsufficientDiskSpace { needed, free }) => {
                return Err(PurgeError::InsufficientDiskSpace {
                    needed,
                    available: Some(free),
                });
            }
            Err(crate::compaction::purge_filter::PurgeBlocked::ProbeFailed(msg)) => {
                return Err(PurgeError::ProbeFailed(msg));
            }
            Err(crate::compaction::purge_filter::PurgeBlocked::ActiveSnapshot { min_seq }) => {
                return Err(PurgeError::SnapshotPinned {
                    oldest_pin_age: as_of_tx.saturating_sub(min_seq),
                });
            }
            Err(crate::compaction::purge_filter::PurgeBlocked::CompactionBusy) => {
                return Err(PurgeError::CompactionBusy);
            }
            Err(other) => {
                return Err(PurgeError::ProbeFailed(format!("{:?}", other)));
            }
        };

        if sstables_to_rewrite.is_empty() {
            return Ok(PurgeReport {
                records_purged: 0,
                bytes_reclaimed: 0,
                sstables_replaced: 0,
                obsolete_files: Vec::new(),
            });
        }

        let db_path = &self.config.path;
        let mut records_purged = 0u64;
        let mut bytes_reclaimed = 0u64;
        let mut sstables_replaced = 0usize;

        let mut manifest_entries = Vec::new();
        let mut replacement_map: std::collections::HashMap<std::path::PathBuf, Option<Arc<SstableReader>>> =
            std::collections::HashMap::new();
        let mut obsolete_files = Vec::new();

        for old_sst in &sstables_to_rewrite {
            let old_path = old_sst.file_path().to_path_buf();
            let mut stream = old_sst
                .stream()
                .await
                .map_err(|e| PurgeError::ReadError {
                    file: old_sst.file_path().to_string_lossy().to_string(),
                    offset: 0,
                    source_details: e.to_string(),
                })?;

            let seq = self
                .next_seq_no
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let count = self
                .segment_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let target_sst_name = format!("sst-purge-{:020}-{:06}.sst", seq, count % 1_000_000);
            let tmp_path = db_path.join(format!("{}.tmp", target_sst_name));
            let final_path = db_path.join(&target_sst_name);

            let mut builder =
                SstableBuilder::create_with_key_manager(&tmp_path, self.key_manager.clone())
                    .await
                    .map_err(|e| PurgeError::WriteError {
                        file: tmp_path.to_string_lossy().to_string(),
                        source_details: e.to_string(),
                    })?;

            builder.set_format_version(old_sst.format_version);
            let mut entries_written = 0u64;

            loop {
                match stream.next_entry().await {
                    Ok(Some((key, val, seq, tx))) => {
                        let raw_seq = seq & !TOMBSTONE_BIT;
                        if scope.matches_key(&key) && raw_seq <= as_of_tx {
                            records_purged += 1;
                            bytes_reclaimed += (key.len() + val.len()) as u64;
                        } else {
                            builder
                                .add(&key, &val, seq, tx)
                                .await
                                .map_err(|e| PurgeError::WriteError {
                                    file: tmp_path.to_string_lossy().to_string(),
                                    source_details: e.to_string(),
                                })?;
                            entries_written += 1;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        drop(builder);
                        let _ = tokio::fs::remove_file(&tmp_path).await;
                        return Err(PurgeError::ReadError {
                            file: old_sst.file_path().to_string_lossy().to_string(),
                            offset: 0,
                            source_details: e.to_string(),
                        });
                    }
                }
            }

            sstables_replaced += 1;
            obsolete_files.push(old_path.clone());

            if entries_written == 0 {
                // SSTable became completely empty post-purge
                drop(builder);
                let _ = tokio::fs::remove_file(&tmp_path).await;
                manifest_entries.push(ManifestEntry::Remove {
                    path: old_path.clone(),
                });
                replacement_map.insert(old_path, None);
            } else {
                let meta = builder
                    .finish()
                    .await
                    .map_err(|e| PurgeError::WriteError {
                        file: tmp_path.to_string_lossy().to_string(),
                        source_details: e.to_string(),
                    })?;

                // Atomic rename tmp -> final
                tokio::fs::rename(&tmp_path, &final_path)
                    .await
                    .map_err(|e| PurgeError::WriteError {
                        file: final_path.to_string_lossy().to_string(),
                        source_details: e.to_string(),
                    })?;

                if let Some(parent) = final_path.parent() {
                    crate::util::fsync_parent_dir(parent)
                        .await
                        .map_err(|e| PurgeError::WriteError {
                            file: final_path.to_string_lossy().to_string(),
                            source_details: format!("Parent fsync failed: {e}"),
                        })?;
                }

                let new_reader = SstableReader::open_with_key_manager(
                    &final_path,
                    Arc::clone(&self.block_cache),
                    self.key_manager.clone(),
                )
                .await
                .map_err(|e| PurgeError::ReadError {
                    file: final_path.to_string_lossy().to_string(),
                    offset: 0,
                    source_details: e.to_string(),
                })?;

                manifest_entries.push(ManifestEntry::Replace {
                    removed: vec![old_path.clone()],
                    added: final_path,
                    added_max_tx: meta.max_tx_id,
                    rank: 0,
                });
                replacement_map.insert(old_path, Some(Arc::new(new_reader)));
            }
        }

        // 4. Atomic Manifest Batch Write
        self.manifest
            .append_batch(&manifest_entries)
            .await
            .map_err(|e| PurgeError::Manifest(e.to_string()))?;

        // 5. In-Memory SSTable Set Update — position-preserving replacement (L6 / B-07 fix)
        //
        // Replace each old SSTable in-place at its original position via explicit old_path keying.
        {
            let mut sst_guard = self.sstables.write().await;
            let updated_sstables: Vec<Arc<SstableReader>> = sst_guard
                .iter()
                .filter_map(|sst| {
                    let path = sst.file_path();
                    if let Some(replacement) = replacement_map.remove(path) {
                        replacement
                    } else {
                        Some(Arc::clone(sst))
                    }
                })
                .collect();

            *sst_guard = updated_sstables;
        }

        Ok(PurgeReport {
            records_purged,
            bytes_reclaimed,
            sstables_replaced,
            obsolete_files,
        })
    }
}
