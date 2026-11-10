//! Kompatibilitätsbrücke zu `contextra_types` und `contextra_ports` hinter dem Feature `workspace`.

use crate::error::CheckpointError;
use crate::snapshot_source::{BoxFuture, SnapshotSource};
use bytes::Bytes;
use contextra_ids::TxId;
use contextra_ports::StorageEngine;
use contextra_types::ContextraError;

impl From<CheckpointError> for ContextraError {
    fn from(err: CheckpointError) -> Self {
        match err {
            CheckpointError::Internal(msg) => ContextraError::Internal(msg),
            CheckpointError::InvalidInput(msg) => ContextraError::InvalidInput(msg),
            CheckpointError::CheckpointNotFound => ContextraError::CheckpointNotFound,
            CheckpointError::Transaction(msg) => ContextraError::Transaction(msg),
            CheckpointError::Serialization(msg) => ContextraError::Serialization(msg),
            CheckpointError::CrossDeviceLink {
                source_path,
                target_path,
            } => ContextraError::CrossDeviceLink {
                source_path,
                target_path,
            },
            CheckpointError::Io(e) => ContextraError::Io(e),
            CheckpointError::IdWidthMismatch { stored, build } => ContextraError::Internal(format!(
                "ID width mismatch: stored {stored} bits, build expects {build} bits"
            )),
        }
    }
}

impl From<ContextraError> for CheckpointError {
    fn from(err: ContextraError) -> Self {
        match err {
            ContextraError::Internal(msg) => CheckpointError::Internal(msg),
            ContextraError::InvalidInput(msg) => CheckpointError::InvalidInput(msg),
            ContextraError::CheckpointNotFound => CheckpointError::CheckpointNotFound,
            ContextraError::Transaction(msg) => CheckpointError::Transaction(msg),
            ContextraError::Serialization(msg) => CheckpointError::Serialization(msg),
            ContextraError::CrossDeviceLink {
                source_path,
                target_path,
            } => CheckpointError::CrossDeviceLink {
                source_path,
                target_path,
            },
            ContextraError::Io(e) => CheckpointError::Io(e),
            other => CheckpointError::Internal(other.to_string()),
        }
    }
}

impl<T: StorageEngine + ?Sized> SnapshotSource for T {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, crate::error::Result<Option<Bytes>>> {
        Box::pin(async move {
            self.get(key)
                .await
                .map_err(CheckpointError::from)
        })
    }

    fn put<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            let port_tx = contextra_types::TxId::new(tx_id.inner());
            self.put(port_tx, key, value)
                .await
                .map_err(CheckpointError::from)
        })
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            let port_tx = contextra_types::TxId::new(tx_id.inner());
            self.delete(port_tx, key)
                .await
                .map_err(CheckpointError::from)
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            let port_tx = contextra_types::TxId::new(tx_id.inner());
            self.commit(port_tx)
                .await
                .map_err(CheckpointError::from)
        })
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            let port_tx = contextra_types::TxId::new(tx_id.inner());
            self.rollback(port_tx)
                .await
                .map_err(CheckpointError::from)
        })
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            let port_tx = contextra_types::TxId::new(tx_id.inner());
            self.rollback_to_tx(port_tx)
                .await
                .map_err(CheckpointError::from)
        })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.flush()
                .await
                .map_err(CheckpointError::from)
        })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, crate::error::Result<u64>> {
        Box::pin(async move {
            self.last_seq_no()
                .await
                .map_err(CheckpointError::from)
        })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, crate::error::Result<TxId>> {
        Box::pin(async move {
            let port_tx = self.last_tx_id().await.map_err(CheckpointError::from)?;
            Ok(TxId::new(port_tx.inner()))
        })
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.pin_checkpoint(seq_no)
                .await
                .map_err(CheckpointError::from)
        })
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, crate::error::Result<()>> {
        Box::pin(async move {
            self.unpin_checkpoint(seq_no)
                .await
                .map_err(CheckpointError::from)
        })
    }

    #[allow(clippy::type_complexity)]
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, crate::error::Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            self.scan_prefix(prefix)
                .await
                .map_err(CheckpointError::from)
        })
    }
}

use crate::vfs::CheckpointVfs;
use contextra_ports::Vfs;
use std::path::Path;

impl<T: Vfs + ?Sized> CheckpointVfs for T {
    fn write_atomic(&self, path: &Path, data: &[u8]) -> std::io::Result<()> {
        self.write_atomic(path, data)
            .map_err(|e| std::io::Error::other(e.to_string()))
    }

    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        self.read(path)
            .map_err(|e| std::io::Error::other(e.to_string()))
    }
}

use crate::meta::{CheckpointMeta, WorkflowState};
use crate::store::PersistentCheckpointStore;

impl From<WorkflowState> for contextra_types::WorkflowState {
    fn from(s: WorkflowState) -> Self {
        Self {
            tx: contextra_types::TxId::new(s.tx.inner()),
            graph_hash: s.graph_hash,
        }
    }
}

impl From<contextra_types::WorkflowState> for WorkflowState {
    fn from(s: contextra_types::WorkflowState) -> Self {
        Self {
            tx: TxId::new(s.tx.inner()),
            graph_hash: s.graph_hash,
        }
    }
}

impl<S: SnapshotSource> contextra_ports::CheckpointCoordinator for PersistentCheckpointStore<S> {
    type Meta = CheckpointMeta;

    async fn create_named_checkpoint(
        &self,
        name: &str,
        collection_id: &str,
        seq_no: u64,
        tx_id: contextra_types::TxId,
        metadata: serde_json::Value,
    ) -> contextra_types::Result<Self::Meta> {
        self.create_checkpoint(name, collection_id, seq_no, TxId::new(tx_id.inner()), metadata)
            .await
            .map_err(ContextraError::from)
    }

    async fn restore_named_checkpoint(&self, name: &str) -> contextra_types::Result<Self::Meta> {
        self.restore_checkpoint(name)
            .await
            .map_err(ContextraError::from)
    }

    async fn drop_named_checkpoint(&self, name: &str) -> contextra_types::Result<()> {
        self.drop_checkpoint(name)
            .await
            .map_err(ContextraError::from)
    }

    async fn list_named_checkpoints(&self) -> contextra_types::Result<Vec<Self::Meta>> {
        self.list_checkpoints()
            .await
            .map_err(ContextraError::from)
    }
}

impl<S: SnapshotSource> contextra_ports::Checkpoint for PersistentCheckpointStore<S> {
    fn take_snapshot<'a>(
        &'a self,
        tx: contextra_types::TxId,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<contextra_types::WorkflowState>> {
        Box::pin(async move {
            let seq_no = self.storage.last_seq_no().await.map_err(ContextraError::from)?;
            Ok(contextra_types::WorkflowState {
                tx,
                graph_hash: *blake3::hash(format!("seq-{}", seq_no).as_bytes()).as_bytes(),
            })
        })
    }

    fn restore<'a>(
        &'a self,
        state: &'a contextra_types::WorkflowState,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        Box::pin(async move {
            self.storage
                .rollback_to_tx(TxId::new(state.tx.inner()))
                .await
                .map_err(ContextraError::from)?;
            self.list_checkpoints().await.map_err(ContextraError::from)?;
            Ok(())
        })
    }
}
