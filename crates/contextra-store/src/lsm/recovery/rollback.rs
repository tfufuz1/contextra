//! # Rollback-Intent und SSTable-Rollback-Verarbeitung (Ring 1)
//!
//! ZWECK: Atomares Rollback auf eine Zieltransaktion inklusive Rollback-Intent-Datei (`MFRLBK\0\0`),
//! MemTable-Pruning, WAL-Trunkierung und selektiver Rekonstruktion/Loeschung von SSTables.
//! INVARIANTEN: Rollback-Intent-Format ist byte-identisch: `b"MFRLBK\0\0"` gefolgt von 8 Byte Little-Endian TxId.

use crate::error::{StoreError as ContextraError, StoreResult as Result};
use crate::lsm::engine::LsmStorage;
use crate::lsm::guard::CommitGuard;
use crate::sstable::{SstableBuilder, SstableReader};
use contextra_ids::{TxId, TOMBSTONE_BIT};
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Magische Konstante fuer Rollback-Intent-Dateien.
pub const INTENT_MAGIC: &[u8] = b"MFRLBK\0\0";

/// Mindestanzahl an ueberlebenden Eintraegen, um eine neue SSTable zu bauen.
pub const MIN_ENTRIES_FOR_SSTABLE_REBUILD: usize = 8;

impl LsmStorage {
    /// Fuehrt ein Rollback des Datenbestands auf `target_tx` aus.
    pub async fn rollback_to_tx(&self, target_tx: TxId) -> Result<()> {
        let _commit_lock = self.commit_mutex.lock().await;
        let commit_guard = CommitGuard {
            _lock: &_commit_lock,
        };
        self.rollback_to_tx_locked(target_tx, &commit_guard).await
    }

    /// Interne Implementierung des Rollbacks unter gehaltener Commit-Sperre.
    pub(crate) async fn rollback_to_tx_locked(
        &self,
        target_tx: TxId,
        _guard: &CommitGuard<'_>,
    ) -> Result<()> {
        self.clear_intent_locks_above_tx(target_tx);

        let intent_path = self
            .config
            .path
            .join(format!("rollback-{:016x}.intent", target_tx.inner()));
        {
            let mut intent_bytes = Vec::with_capacity(16);
            intent_bytes.extend_from_slice(INTENT_MAGIC);
            intent_bytes.extend_from_slice(&target_tx.inner().to_le_bytes());
            tokio::fs::write(&intent_path, &intent_bytes)
                .await
                .map_err(|e| {
                    ContextraError::Storage(format!("Failed to write rollback intent file: {e}"))
                })?;
            let parent = self.config.path.clone();
            tokio::task::spawn_blocking(move || {
                // INVARIANT-KONFORM: In spawn_blocking gewrappt; Parent-Directory FSync bei Crash-Recovery / Rollback (kein Hot-Path).
                std::fs::File::open(&parent)
                    .and_then(|f| f.sync_all())
                    .map_err(|e| {
                        ContextraError::Storage(format!(
                            "Failed to fsync dir after intent file: {e}"
                        ))
                    })
            })
            .await
            .map_err(|e| ContextraError::Internal(e.to_string()))??;
        }

        let state = self.state.write().await;
        let wal = self.wal.read().await.clone();

        let (target_offset, target_hmac) = wal.find_tx_offset(target_tx).await?;
        wal.truncate(target_offset, target_hmac).await?;

        fn prune_memtable_above_tx(memtable: &crate::memtable::MemTable, target_tx: u64) {
            let txs_to_rollback: std::collections::BTreeSet<u64> = memtable
                .iter()
                .into_iter()
                .map(|(_, _, _, tx)| tx)
                .filter(|&tx| tx > target_tx)
                .collect();
            for tx_id in txs_to_rollback {
                memtable.rollback(tx_id);
            }
        }

        prune_memtable_above_tx(&state.memtable, target_tx.inner());
        for imm in &state.immutable_memtables {
            prune_memtable_above_tx(imm, target_tx.inner());
        }

        let mut sstables_lock = self.sstables.write().await;
        let mut sst_to_remove = Vec::new();
        let mut spanning_sstables = Vec::new();

        sstables_lock.retain(|sst| {
            let meta = sst.metadata();
            if meta.min_tx_id > target_tx.inner() {
                sst_to_remove.push(sst.file_path().to_path_buf());
                false
            } else if meta.min_tx_id <= target_tx.inner() && meta.max_tx_id > target_tx.inner() {
                spanning_sstables.push(Arc::clone(sst));
                false
            } else {
                true
            }
        });

        for spanning in spanning_sstables {
            let mut surviving_entries = Vec::new();
            let mut stream = spanning.stream().await?;
            while let Some((k, v, seq, tx)) = stream.next_entry().await? {
                if tx <= target_tx.inner() {
                    surviving_entries.push((k, v, seq, tx));
                }
            }

            if surviving_entries.len() >= MIN_ENTRIES_FOR_SSTABLE_REBUILD {
                let count = self.segment_counter.fetch_add(1, Ordering::Relaxed);
                let seq = self.next_seq_no.load(Ordering::Relaxed);
                let new_sst_path =
                    self.config
                        .path
                        .join(format!("sst-{:020}-{:06}.sst", seq, count % 1_000_000));

                let mut builder = SstableBuilder::create_with_key_manager(
                    &new_sst_path,
                    self.key_manager.clone(),
                )
                .await?;

                for (k, v, seq, tx) in surviving_entries {
                    builder.add(&k, &v, seq, tx).await?;
                }
                builder.finish().await?;

                let new_reader = SstableReader::open_with_key_manager(
                    &new_sst_path,
                    Arc::clone(&self.block_cache),
                    self.key_manager.clone(),
                )
                .await?;

                self.manifest
                    .append(&crate::manifest::ManifestEntry::Add {
                        path: new_sst_path.clone(),
                        max_tx: new_reader.metadata().max_tx_id,
                    })
                    .await?;

                sstables_lock.push(Arc::new(new_reader));
            } else if !surviving_entries.is_empty() {
                for (k, v, seq, tx) in surviving_entries {
                    state
                        .memtable
                        .put(bytes::Bytes::from(k), bytes::Bytes::from(v), seq, tx);
                }
            }

            sst_to_remove.push(spanning.file_path().to_path_buf());
        }

        sstables_lock.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);

        let mut max_seq = 0u64;
        for sst in sstables_lock.iter() {
            max_seq = max_seq.max(sst.metadata().max_seq & !TOMBSTONE_BIT);
        }
        for (_, _, seq, _) in state.memtable.iter() {
            max_seq = max_seq.max(seq & !TOMBSTONE_BIT);
        }
        for imm in &state.immutable_memtables {
            for (_, _, seq, _) in imm.iter() {
                max_seq = max_seq.max(seq & !TOMBSTONE_BIT);
            }
        }
        drop(sstables_lock);

        let mut manifest_batch = Vec::new();
        for path in sst_to_remove {
            tracing::info!("Removing SSTable during rollback: {:?}", path);
            manifest_batch.push(crate::manifest::ManifestEntry::Remove { path: path.clone() });
            if let Err(e) = tokio::fs::remove_file(&path).await {
                if e.kind() != std::io::ErrorKind::NotFound {
                    tracing::error!(
                        path = ?path,
                        "Orphaned SSTable konnte nicht entfernt werden: {e}. Manuelles Cleanup nötig."
                    );
                }
            }
        }

        manifest_batch.push(crate::manifest::ManifestEntry::RollbackComplete {
            target_tx: target_tx.inner(),
        });

        if let Err(e) = self.manifest.append_batch(&manifest_batch).await {
            tracing::warn!("Failed to write Manifest batch during rollback: {}", e);
        }

        if let Err(e) = tokio::fs::remove_file(&intent_path).await {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    "Could not remove rollback intent file {:?}: {} \
                     (non-fatal — recovery will re-run on next startup)",
                    intent_path,
                    e
                );
            }
        }

        self.next_seq_no.store(max_seq + 1, Ordering::SeqCst);
        self.last_committed_tx
            .store(target_tx.inner(), Ordering::SeqCst);

        {
            let mut locks = self.intent_locks.lock().unwrap_or_else(|e| e.into_inner());
            locks.retain(|_, v| v.inner() <= target_tx.inner());
        }

        tracing::info!(
            "Rollback to TX {} successful. Max seq: {}, WAL offset: {}",
            target_tx.inner(),
            max_seq,
            target_offset
        );

        Ok(())
    }
}
