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
    let (to_flush, old_wal_opt, old_wal_hmac) = {
        let _commit_lock = storage.commit_mutex.lock().await;

        let mut state = storage.state.write().await;
        if state.memtable.is_empty() && state.immutable_memtables.is_empty() {
            return Ok(());
        }

        let (old_wal_opt, old_wal_hmac) = if !state.memtable.is_empty() {
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
            state.immutable_memtables.push(old_memtable);
            (Some(old_wal), hmac)
        } else {
            (None, [0u8; 32])
        };

        let to_flush = state.immutable_memtables.clone();
        (to_flush, old_wal_opt, old_wal_hmac)
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
        storage
            .manifest
            .append_batch(&[
                crate::manifest::ManifestEntry::Add {
                    path: sst_path.clone(),
                    max_tx: reader.metadata().max_tx_id,
                },
                crate::manifest::ManifestEntry::WalCheckpoint { hmac: old_wal_hmac },
            ])
            .await?;
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

        // Delete old WAL only after SST and manifest entry are durable and truncate_lock acquired
        if let Some(ref old_wal) = old_wal_opt {
            let _trunc_guard = old_wal.truncate_lock.lock().await;
            if let Err(e) = tokio::fs::remove_file(old_wal.path()).await {
                tracing::debug!("Could not delete old WAL {:?}: {}", old_wal.path(), e);
            }
        }

        let bytes_freed: u64 = to_flush.iter().map(|mt| mt.size() as u64).sum();
        storage.budget.release_memory(bytes_freed);

        storage
            .budget_tracking_drift_bytes
            .store(0, std::sync::atomic::Ordering::Relaxed);

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
