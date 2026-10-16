use super::super::engine::LsmStorage;
use super::super::{MemTable, SstableBuilder, SstableReader, Wal};
use crate::compaction::retain_key_versions;
use bytes::Bytes;
use contextra_core::{ContextraError, Result, TOMBSTONE_BIT};
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Flushes the active MemTable to an SSTable.
///
/// MUST NOT be called while holding `commit_mutex` (will deadlock).
/// Rotation (WAL swap and MemTable swap) is performed under `commit_mutex` to prevent
/// concurrent sequence allocation races during rotation.
pub(super) async fn flush(storage: &LsmStorage) -> Result<()> {
    let _flush_lock = storage.flush_mutex.lock().await;

    // ── Phase 0: Schnellcheck (Read-Lock, kein I/O) ──────────────────────
    let has_active_memtable = {
        let state = storage.state.read().await;
        if state.memtable.is_empty() && state.immutable_memtables.is_empty() {
            return Ok(());
        }
        !state.memtable.is_empty()
    }; // read lock freigegeben

    // ── Phase 1: Pre-Allokierung der neuen WAL VOR dem Write-Lock ────────
    let new_wal_opt = if has_active_memtable {
        let flush_id = storage.flush_counter.fetch_add(1, Ordering::SeqCst);
        let wal_path = storage
            .config
            .path
            .join(format!("wal-{:020}.log", flush_id));
        let new_wal = Wal::open_with_key_manager(wal_path, storage.key_manager.clone()).await?;
        Some(new_wal)
    } else {
        None
    };

    // ── Phase 2: Atomarer Swap unter commit_mutex und state.write() ──────
    let (to_flush, old_wal_opt) = {
        let _commit_lock = storage.commit_mutex.lock().await;

        let mut state = storage.state.write().await;
        if state.memtable.is_empty() && state.immutable_memtables.is_empty() {
            return Ok(());
        }

        let old_wal_opt = if !state.memtable.is_empty() {
            let new_wal = match new_wal_opt {
                Some(w) => w,
                None => {
                    let flush_id = storage.flush_counter.fetch_add(1, Ordering::SeqCst);
                    let wal_path = storage
                        .config
                        .path
                        .join(format!("wal-{:020}.log", flush_id));
                    Wal::open_with_key_manager(wal_path, storage.key_manager.clone()).await?
                }
            };

            let old_memtable = std::mem::replace(&mut state.memtable, Arc::new(MemTable::new()));
            let old_wal = {
                let mut wal_guard = storage.wal.write().await;
                std::mem::replace(&mut *wal_guard, Arc::new(new_wal))
            };
            old_wal.sealed.store(true, Ordering::SeqCst);
            let hmac = *old_wal.last_hmac.lock().await;
            storage
                .pending_sealed_wals
                .lock()
                .await
                .push((old_wal.path().to_path_buf(), hmac));
            state.immutable_memtables.push(old_memtable);
            Some(old_wal)
        } else {
            None
        };

        let to_flush = state.immutable_memtables.clone();
        (to_flush, old_wal_opt)
    }; // commit_mutex, state.write, and storage.wal released!

    let count = storage.segment_counter.fetch_add(1, Ordering::Relaxed);
    let seq = storage.next_seq_no.load(Ordering::Relaxed);
    let sst_path =
        storage
            .config
            .path
            .join(format!("sst-{:020}-{:06}.sst", seq, count % 1_000_000));

    // ── Phase 3: Expensive I/O & Atomic Transition ──────────────────────────
    let phase3_res: Result<()> = async {
        let floor_seq = storage.snapshot_registry.min_active_seqno();

        let mut key_map: BTreeMap<Bytes, Vec<(Bytes, u64, u64)>> = BTreeMap::new();

        for mt in &to_flush {
            for (k, v, seq, tx) in mt.iter() {
                key_map.entry(k).or_default().push((v, seq, tx));
            }
        }

        let mut builder =
            SstableBuilder::create_with_key_manager(&sst_path, storage.key_manager.clone()).await?;

        for (k, mut versions) in key_map {
            versions.sort_by(|a, b| (b.1 & !TOMBSTONE_BIT).cmp(&(a.1 & !TOMBSTONE_BIT)));
            let retained = retain_key_versions(versions, |(_, seq, _)| *seq, floor_seq);
            for (v, seq, tx) in retained {
                builder.add(&k, &v, seq, tx).await?;
            }
        }

        builder
            .finish()
            .await
            .map_err(|e| ContextraError::Storage(format!("SSTable finish failed: {}", e)))?;

        let reader = SstableReader::open_with_key_manager(
            &sst_path,
            Arc::clone(&storage.block_cache),
            storage.key_manager.clone(),
        )
        .await
        .map_err(|e| ContextraError::Storage(format!("SSTable open after flush failed: {}", e)))?;

        // === SSTABLE MANIFEST INTEGRATION START ===
        let latest_hmac_opt = {
            let pending = storage.pending_sealed_wals.lock().await;
            pending
                .iter()
                .rev()
                .find(|(_, hmac)| *hmac != [0u8; 32])
                .map(|(_, hmac)| *hmac)
        };

        let mut manifest_entries = vec![crate::manifest::ManifestEntry::Add {
            path: sst_path.clone(),
            max_tx: reader.metadata().max_tx_id,
        }];

        if let Some(hmac) = latest_hmac_opt {
            manifest_entries.push(crate::manifest::ManifestEntry::WalCheckpoint { hmac });
        }

        storage.manifest.append_batch(&manifest_entries).await?;
        // === SSTABLE MANIFEST INTEGRATION END ===

        // Atomic transition: remove successfully flushed memtables from immutable memtables and add to SSTables
        let mut state = storage.state.write().await;
        let mut sstables = storage.sstables.write().await;

        state
            .immutable_memtables
            .retain(|mt| !to_flush.iter().any(|tf| Arc::ptr_eq(mt, tf)));

        // last_committed_tx MUSS vor sstables.push() aktualisiert werden — sonst Race-Fenster für parallele Reader, siehe DECISIONS.md ADR-043.
        let sst_max_tx = reader.metadata().max_tx_id;
        storage.advance_visibility(contextra_core::TxId::new(sst_max_tx));

        sstables.push(Arc::new(reader));
        sstables.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);

        debug_assert!(
            sstables
                .windows(2)
                .all(|w| (w[0].metadata().max_seq & !TOMBSTONE_BIT)
                    <= (w[1].metadata().max_seq & !TOMBSTONE_BIT)),
            "SSTable list must be sorted by max_seq in ascending order after flush"
        );

        drop(sstables);
        drop(state);

        // Delete ALL sealed WAL files whose data was flushed into SSTable (H4b)
        {
            let mut pending_guard = storage.pending_sealed_wals.lock().await;
            let active_wal_path = {
                let wal_read = storage.wal.read().await;
                wal_read.path().to_path_buf()
            };

            for (wal_path, _) in pending_guard.drain(..) {
                if wal_path != active_wal_path {
                    let mut trunc_guard_opt = None;
                    if let Some(ref old_wal) = old_wal_opt {
                        if old_wal.path() == wal_path.as_path() {
                            trunc_guard_opt = Some(old_wal.truncate_lock.lock().await);
                        }
                    }

                    if let Err(e) = tokio::fs::remove_file(&wal_path).await {
                        tracing::debug!("Could not delete old WAL {:?}: {}", wal_path, e);
                    } else {
                        let uuid_sidecar =
                            std::path::PathBuf::from(format!("{}.uuid", wal_path.display()));
                        if let Err(e) = tokio::fs::remove_file(&uuid_sidecar).await {
                            if e.kind() != std::io::ErrorKind::NotFound {
                                tracing::debug!(
                                    "Could not delete sidecar {:?}: {}",
                                    uuid_sidecar,
                                    e
                                );
                            }
                        }
                    }
                    drop(trunc_guard_opt);
                }
            }
            let _ = crate::util::fsync_parent_dir(&storage.config.path).await;
        }

        let bytes_freed: u64 = to_flush.iter().map(|mt| mt.size() as u64).sum();
        storage.budget.release_memory(bytes_freed);

        storage
            .budget_tracking_drift_bytes
            .store(0, std::sync::atomic::Ordering::Relaxed);

        let prune_bound = storage
            .tx_buffer
            .min_read_snapshot()
            .unwrap_or(u64::MAX)
            .min(storage.snapshot_registry.min_active_seqno());
        storage.ssi_validator.prune_through(prune_bound);

        tracing::info!("Flushed memtable to SSTable: {} bytes", bytes_freed);
        Ok(())
    }
    .await;

    if let Err(ref e) = phase3_res {
        if sst_path.exists() {
            if let Err(rm_err) = tokio::fs::remove_file(&sst_path).await {
                tracing::warn!(
                    path = ?sst_path,
                    "Failed to remove partial SSTable after flush failure: {rm_err}"
                );
            }
        }

        tracing::warn!(
            "Flush Phase 3 failed; old memtable retained in immutable list for continued read availability. WAL on disk is intact. Error: {e}"
        );
    }

    phase3_res
}
