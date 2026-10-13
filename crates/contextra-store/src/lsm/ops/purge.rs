// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Implementation von LsmStorage::purge_keys (Compaction-basierte Bereinigung).
// INVARIANTEN: Active snapshot pin check, Disk space pre-flight probe, atomic manifest transitions, Zero-panic.

use crate::compaction::purge_filter::{FreeSpaceProbe, PurgeBlocked, PurgeReport, PurgeScope};
use crate::lsm::engine::LsmStorage;
use crate::manifest::ManifestEntry;
use crate::sstable::{SstableBuilder, SstableReader};
use contextra_core::StorageEngine;
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
    ) -> std::result::Result<PurgeReport, PurgeBlocked> {
        // 1. Active snapshot pin check
        let min_active_seq = self.snapshot_registry.min_active_seqno();
        if min_active_seq <= as_of_tx {
            return Err(PurgeBlocked::ActiveSnapshot {
                min_seq: min_active_seq,
            });
        }

        // 2. Active Memtable Flush to SSTable first
        StorageEngine::flush(self)
            .await
            .map_err(|_| PurgeBlocked::CompactionBusy)?;

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

        // 3. Disk space probe check
        let needed_space = total_bytes.max(1024 * 1024);
        let free_space = space_probe.and_then(|p| p.free_space_bytes()).unwrap_or(0);

        if free_space < needed_space {
            return Err(PurgeBlocked::InsufficientDiskSpace {
                needed: needed_space,
                free: free_space,
            });
        }

        let mut sstables_to_rewrite = Vec::new();

        // Identify SSTables that contain keys matching scope
        for sst in &snapshot_sstables {
            let mut stream = sst
                .stream()
                .await
                .map_err(|_| PurgeBlocked::CompactionBusy)?;
            let mut matches = false;
            while let Ok(Some((key, _val, _seq, _tx))) = stream.next_entry().await {
                if scope.matches_key(&key) {
                    matches = true;
                    break;
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
                .map_err(|_| PurgeBlocked::CompactionBusy)?;

            let mut valid_entries = Vec::new();

            while let Ok(Some((key, val, seq, tx))) = stream.next_entry().await {
                if scope.matches_key(&key) {
                    records_purged += 1;
                    bytes_reclaimed += (key.len() + val.len()) as u64;
                } else {
                    valid_entries.push((key, val, seq, tx));
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
                        .map_err(|_| PurgeBlocked::CompactionBusy)?;

                builder.set_format_version(old_sst.format_version);

                for (k, v, seq, tx) in &valid_entries {
                    builder
                        .add(k, v, *seq, *tx)
                        .await
                        .map_err(|_| PurgeBlocked::CompactionBusy)?;
                }

                let meta = builder
                    .finish()
                    .await
                    .map_err(|_| PurgeBlocked::CompactionBusy)?;

                // Atomic rename tmp -> final
                tokio::fs::rename(&tmp_path, &final_path)
                    .await
                    .map_err(|_| PurgeBlocked::CompactionBusy)?;

                if let Some(parent) = final_path.parent() {
                    let _ = crate::util::fsync_parent_dir(parent).await;
                }

                let new_reader = SstableReader::open_with_key_manager(
                    &final_path,
                    Arc::clone(&self.block_cache),
                    self.key_manager.clone(),
                )
                .await
                .map_err(|_| PurgeBlocked::CompactionBusy)?;

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
            .map_err(|_| PurgeBlocked::CompactionBusy)?;

        // 5. In-Memory SSTable Set Update
        {
            let mut sst_guard = self.sstables.write().await;
            let mut updated_sstables: Vec<Arc<SstableReader>> = Vec::new();

            for sst in sst_guard.iter() {
                if !replaced_old_paths.iter().any(|p| p == sst.file_path()) {
                    updated_sstables.push(Arc::clone(sst));
                }
            }

            updated_sstables.extend(new_readers);
            *sst_guard = updated_sstables;
        }

        Ok(PurgeReport {
            records_purged,
            bytes_reclaimed,
            sstables_replaced,
        })
    }
}
