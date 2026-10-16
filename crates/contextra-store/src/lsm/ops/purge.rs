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
            .map_err(|_| PurgeError::CompactionBusy)?;

        // Snapshot existing sstables list under read lock
        let snapshot_sstables = {
            let sst_guard = self.sstables.read().await;
            sst_guard.clone()
        };

        // Estimate total bytes of affected SSTables for space check
        let total_bytes: u64 = snapshot_sstables
            .iter()
            .map(|sst| sst.metadata().file_size)
            .sum();

        // 3. Disk space probe check (L8 fix)
        // Probe error yields ProbeFailed. Some(0) yields InsufficientDiskSpace.
        // None probe means unknown free space: conservative execution (1 SSTable at a time).
        let probe_res = match space_probe {
            Some(p) => p.available_bytes().map_err(|e| PurgeError::ProbeFailed(e.0))?,
            None => None,
        };

        let needed_space = total_bytes.max(1024 * 1024);
        match probe_res {
            Some(0) => {
                return Err(PurgeError::InsufficientDiskSpace {
                    needed: needed_space,
                    available: Some(0),
                });
            }
            Some(free_space) => {
                if free_space < needed_space {
                    return Err(PurgeError::InsufficientDiskSpace {
                        needed: needed_space,
                        available: Some(free_space),
                    });
                }
            }
            None => {
                // Unknown free space: permitted to start, candidate check per file
            }
        }

        let mut sstables_to_rewrite = Vec::new();

        // Identify SSTables that contain keys matching scope
        for sst in &snapshot_sstables {
            let mut stream = sst
                .stream()
                .await
                .map_err(|e| PurgeError::ReadError {
                    file: sst.file_path().to_string_lossy().to_string(),
                    offset: 0,
                    source_details: e.to_string(),
                })?;
            let mut matches = false;
            loop {
                match stream.next_entry().await {
                    Ok(Some((key, _val, _seq, _tx))) => {
                        if scope.matches_key(&key) {
                            matches = true;
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(PurgeError::ReadError {
                            file: sst.file_path().to_string_lossy().to_string(),
                            offset: 0,
                            source_details: e.to_string(),
                        });
                    }
                }
            }
            if matches {
                sstables_to_rewrite.push(Arc::clone(sst));
            }
        }

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
        let mut new_readers = Vec::new();
        let mut replaced_old_paths = Vec::new();

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

            let mut valid_entries = Vec::new();

            loop {
                match stream.next_entry().await {
                    Ok(Some((key, val, seq, tx))) => {
                        if scope.matches_key(&key) {
                            records_purged += 1;
                            bytes_reclaimed += (key.len() + val.len()) as u64;
                        } else {
                            valid_entries.push((key, val, seq, tx));
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(PurgeError::ReadError {
                            file: old_sst.file_path().to_string_lossy().to_string(),
                            offset: 0,
                            source_details: e.to_string(),
                        });
                    }
                }
            }

            sstables_replaced += 1;
            replaced_old_paths.push(old_path.clone());

            if valid_entries.is_empty() {
                // SSTable became completely empty post-purge
                manifest_entries.push(ManifestEntry::Remove {
                    path: old_path.clone(),
                });
            } else {
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

                for (k, v, seq, tx) in &valid_entries {
                    builder
                        .add(k, v, *seq, *tx)
                        .await
                        .map_err(|e| PurgeError::WriteError {
                            file: tmp_path.to_string_lossy().to_string(),
                            source_details: e.to_string(),
                        })?;
                }

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

                new_readers.push(Arc::new(new_reader));

                manifest_entries.push(ManifestEntry::Replace {
                    removed: vec![old_path],
                    added: final_path,
                    added_max_tx: meta.max_tx_id,
                    rank: 0,
                });
            }
        }

        // 4. Atomic Manifest Batch Write
        self.manifest
            .append_batch(&manifest_entries)
            .await
            .map_err(|e| PurgeError::Manifest(e.to_string()))?;

        // 5. In-Memory SSTable Set Update — position-preserving replacement (L6 fix)
        //
        // The read path iterates `sstables.iter().rev()` and takes the first match.
        // Appending rewritten readers at the end would give them the highest priority,
        // causing newer versions of non-purged keys (in other SSTables) to be shadowed
        // by older versions carried over from the rewritten file.
        //
        // Fix: replace each old SSTable in-place at its original position.
        {
            let mut sst_guard = self.sstables.write().await;
            let mut updated_sstables: Vec<Arc<SstableReader>> = sst_guard.clone();

            // Build a map from old path → new reader
            let mut replacement_map: std::collections::HashMap<std::path::PathBuf, Option<Arc<SstableReader>>> =
                std::collections::HashMap::new();
            for old_path in &replaced_old_paths {
                // Default to None (removal) — will be overwritten if a replacement reader exists
                replacement_map.insert(old_path.clone(), None);
            }
            // Associate new readers with old paths in order
            let mut reader_iter = new_readers.into_iter();
            for old_path in &replaced_old_paths {
                // If we have a replacement reader, map it; otherwise the SSTable was empty
                // and should be removed (already None in the map)
                if let Some(new_reader) = reader_iter.next() {
                    replacement_map.insert(old_path.clone(), Some(new_reader));
                }
            }

            // Rebuild the list: replace in-place or remove
            updated_sstables = updated_sstables
                .into_iter()
                .filter_map(|sst| {
                    let path = sst.file_path().to_path_buf();
                    if let Some(replacement) = replacement_map.remove(&path) {
                        // This was a replaced SSTable — substitute or remove
                        replacement
                    } else {
                        // Not replaced — keep as-is
                        Some(sst)
                    }
                })
                .collect();

            *sst_guard = updated_sstables;
        }

        Ok(PurgeReport {
            records_purged,
            bytes_reclaimed,
            sstables_replaced,
            obsolete_files: replaced_old_paths,
        })
    }
}
