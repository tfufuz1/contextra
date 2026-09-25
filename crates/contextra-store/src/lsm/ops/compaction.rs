use super::super::engine::LsmStorage;
use super::super::{MemTable, SstableBuilder, SstableReader, Wal};
use contextra_core::{ContextraError, Result, TOMBSTONE_BIT};
use std::sync::atomic::Ordering;
use std::sync::Arc;

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
    // WAL-Erstellung (Datei erstellen, Header schreiben) erfolgt außerhalb des State-Locks,
    // um Tokio-Worker-Thread Blockaden bei Disk-Latenz-Spikes zu verhindern (Fix E).
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

    // ── Phase 2: Atomarer Swap unter Write-Lock ──────────────────────────
    let (to_flush, old_wal_path) = {
        // INVARIANT-LOCK-2 (Atomarer Memtable-Swap):
        // Write-Lock serialisiert diesen Swap atomar gegen alle parallelen commit()-Aufrufe im
        // Single-Commit-Pfad (die ebenfalls state.write() halten). Nach erfolgreichem Swap zeigt
        // `state.memtable` auf einen frischen, leeren Memtable; der alte wird als immutable weitergeführt.
        // Group-Commit-Leader-Pfade halten hier bereits commit_mutex, sodass kein Commit simultan
        // in den neu-swapped Memtable schreibt, bevor dieser korrekt initialisiert ist.
        let mut state = storage.state.write().await;
        if state.memtable.is_empty() && state.immutable_memtables.is_empty() {
            return Ok(());
        }

        let old_wal_path = if !state.memtable.is_empty() {
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
            state.immutable_memtables.push(old_memtable);
            let path = old_wal.path().to_path_buf();
            drop(old_wal);
            Some(path)
        } else {
            None
        };

        let to_flush = state.immutable_memtables.clone();
        (to_flush, old_wal_path)
    }; // write lock freigegeben

    let count = storage.segment_counter.fetch_add(1, Ordering::Relaxed);
    let seq = storage.next_seq_no.load(Ordering::Relaxed);
    let sst_path =
        storage
            .config
            .path
            .join(format!("sst-{:020}-{:06}.sst", seq, count % 1_000_000));

    // ── Phase 3: Expensive I/O & Atomic Transition ──────────────────────────
    let phase3_res: Result<()> = async {
        let mut builder =
            SstableBuilder::create_with_key_manager(&sst_path, storage.key_manager.clone()).await?;

        let mut map = std::collections::BTreeMap::new();
        for mt in &to_flush {
            for (k, v, seq, tx) in mt.iter_latest() {
                map.insert(k, (v, seq, tx));
            }
        }
        for (k, (v, seq, tx)) in map {
            builder.add(&k, &v, seq, tx).await?;
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
            .append(&crate::manifest::ManifestEntry::Add {
                path: sst_path.clone(),
                max_tx: reader.metadata().max_tx_id,
            })
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

        // Best-effort delete of old WAL (non-critical if it fails, as it will be replayed safely)
        if let Some(ref path) = old_wal_path {
            if let Err(e) = tokio::fs::remove_file(path).await {
                tracing::debug!("Could not delete old WAL {:?}: {}", path, e);
            }
        }

        let bytes_freed: u64 = to_flush.iter().map(|mt| mt.size() as u64).sum();
        storage.budget.release_memory(bytes_freed);

        // M-6 FIX: Reset drift counter after successful flush.
        // After a flush, the budget is accurately reflected via release_memory().
        // Drift accumulated during this memtable's lifetime is now irrelevant.
        storage
            .budget_tracking_drift_bytes
            .store(0, std::sync::atomic::Ordering::Relaxed);

        tracing::info!("Flushed memtable to SSTable: {} bytes", bytes_freed);
        Ok(())
    }
    .await;

    if let Err(ref e) = phase3_res {
        // Cleanup on Phase 3 failure:
        // Retain old memtables in state.immutable_memtables for continued read availability.
        // storage.budget.release_memory() is NOT called because memory is still in use.
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
