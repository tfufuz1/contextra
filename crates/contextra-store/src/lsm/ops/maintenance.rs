use super::super::engine::LsmStorage;
use contextra_core::{Result, StorageStats, TxId};
use std::sync::atomic::Ordering;

pub(super) async fn rollback(storage: &LsmStorage, tx_id: TxId) -> Result<()> {
    storage.tx_buffer.discard_kv(tx_id);
    storage.cleanup_intent_locks_for_tx(tx_id);
    Ok(())
}

pub(super) async fn rollback_to_tx(storage: &LsmStorage, tx_id: TxId) -> Result<()> {
    LsmStorage::rollback_to_tx(storage, tx_id).await
}

pub(super) async fn pin_checkpoint(storage: &LsmStorage, seq_no: u64) -> Result<()> {
    storage.snapshot_registry.pin(seq_no);
    Ok(())
}

pub(super) async fn unpin_checkpoint(storage: &LsmStorage, seq_no: u64) -> Result<()> {
    storage.snapshot_registry.unpin(seq_no);
    Ok(())
}

pub(super) async fn stats(storage: &LsmStorage) -> Result<StorageStats> {
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

pub(super) async fn last_seq_no(storage: &LsmStorage) -> Result<u64> {
    Ok(storage.next_seq_no.load(Ordering::SeqCst).saturating_sub(1))
}

pub(super) async fn last_tx_id(storage: &LsmStorage) -> Result<TxId> {
    Ok(TxId::new(storage.last_committed_tx.load(Ordering::SeqCst)))
}
