//! # Wartungsoperationen (Ring 1)
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Behandelt Transaktions-Rollback, Checkpoint Pin/Unpin, Metadaten und Speicherstatistiken
//! // INVARIANTEN: Fail-Closed, INV-STORE-1

use super::super::engine::LsmStorage;
use crate::error::StoreResult as Result;
use crate::stats::StorageStats;
use contextra_ids::TxId;
use std::sync::atomic::Ordering;

/// Verwirft uncommittete Schreiboperationen einer Transaktion im Staging-Puffer und gibt Intent-Locks frei.
pub(crate) async fn rollback(storage: &LsmStorage, tx_id: TxId) -> Result<()> {
    storage.tx_buffer.discard_kv(tx_id);
    storage.cleanup_intent_locks_for_tx(tx_id);
    Ok(())
}

/// Fuehrt ein Rollback bis zu einer Ziel-Transaktions-ID durch.
#[allow(dead_code)]
pub(crate) async fn rollback_to_tx(storage: &LsmStorage, tx_id: TxId) -> Result<()> {
    LsmStorage::rollback_to_tx(storage, tx_id).await
}

/// Pinnt einen Sequenzstand gegen Frueh-Kompaktion.
pub(crate) async fn pin_checkpoint(storage: &LsmStorage, seq_no: u64) -> Result<()> {
    storage.snapshot_registry.pin(seq_no);
    Ok(())
}

/// Gibt einen gepinnten Sequenzstand wieder frei.
pub(crate) async fn unpin_checkpoint(storage: &LsmStorage, seq_no: u64) -> Result<()> {
    storage.snapshot_registry.unpin(seq_no);
    Ok(())
}

/// Berechnet aktuelle physische Speicherstatistiken (SSTable- und MemTable-Groessen).
pub(crate) async fn stats(storage: &LsmStorage) -> Result<StorageStats> {
    let state = storage.state.read().await;
    let sstables = storage.sstables.read().await;
    let num_segments = sstables.len();
    let mut total_size_bytes = 0;
    for sst in sstables.iter() {
        total_size_bytes += sst.metadata().file_size;
    }

    let mut memtable_size_bytes = state.memtable.size() as u64;
    for m in &state.immutable_memtables {
        memtable_size_bytes += m.size() as u64;
    }

    Ok(StorageStats {
        num_segments,
        total_size_bytes,
        memtable_size_bytes,
    })
}

/// Liefert die zuletzt vergebene Sequenznummer.
pub(crate) async fn last_seq_no(storage: &LsmStorage) -> Result<u64> {
    Ok(storage.next_seq_no.load(Ordering::SeqCst).saturating_sub(1))
}

/// Liefert den aktuellen Stand der committeten Transaktionen.
pub(crate) async fn last_tx_id(storage: &LsmStorage) -> Result<TxId> {
    Ok(TxId::new(storage.last_committed_tx.load(Ordering::SeqCst)))
}
