//! # Brücke: `contextra_ports::StorageEngine` als lokale `SnapshotSource`-Naht
//!
//! Der Kern kennt nur den lokalen Trait [`crate::snapshot_source::SnapshotSource`]
//! und dessen Fehlertyp [`CheckpointError`]. Diese Datei verbindet ihn mit dem
//! Workspace-Speicher-Port. Sie ist nur mit Feature `workspace` übersetzbar.

use crate::error::{CheckpointError, CheckpointResult};
use crate::snapshot_source::{BoxFuture, SnapshotSource};
use bytes::Bytes;
use contextra_ids::TxId;
use contextra_ports::StorageEngine;
use contextra_types::ContextraError;

/// Bildet Workspace-Fehler auf den lokalen Fehlertyp ab.
///
/// I/O bleibt I/O. Die übrigen Varianten werden Kategorien zugeordnet, sonst `Internal`.
fn map_err(err: ContextraError) -> CheckpointError {
    match err {
        ContextraError::Io(e) => CheckpointError::Io(e),
        ContextraError::Serialization(msg) => CheckpointError::Serialization(msg),
        ContextraError::Transaction(msg) => CheckpointError::Transaction(msg),
        ContextraError::InvalidInput(msg) => CheckpointError::InvalidInput(msg),
        ContextraError::CheckpointNotFound => CheckpointError::CheckpointNotFound,
        other => CheckpointError::Internal(other.to_string()),
    }
}

impl<T: StorageEngine + ?Sized> SnapshotSource for T {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, CheckpointResult<Option<Bytes>>> {
        Box::pin(async move { StorageEngine::get(self, key).await.map_err(map_err) })
    }

    fn put<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            StorageEngine::put(self, tx_id, key, value)
                .await
                .map_err(map_err)
        })
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            StorageEngine::delete(self, tx_id, key)
                .await
                .map_err(map_err)
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move { StorageEngine::commit(self, tx_id).await.map_err(map_err) })
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move { StorageEngine::rollback(self, tx_id).await.map_err(map_err) })
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            StorageEngine::rollback_to_tx(self, tx_id)
                .await
                .map_err(map_err)
        })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move { StorageEngine::flush(self).await.map_err(map_err) })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, CheckpointResult<u64>> {
        Box::pin(async move { StorageEngine::last_seq_no(self).await.map_err(map_err) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, CheckpointResult<TxId>> {
        Box::pin(async move { StorageEngine::last_tx_id(self).await.map_err(map_err) })
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            StorageEngine::pin_checkpoint(self, seq_no)
                .await
                .map_err(map_err)
        })
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            StorageEngine::unpin_checkpoint(self, seq_no)
                .await
                .map_err(map_err)
        })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, CheckpointResult<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            StorageEngine::scan_prefix(self, prefix)
                .await
                .map_err(map_err)
        })
    }
}
