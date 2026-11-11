//! # Implementierung der ports-Traits für Checkpoint-Komponenten
//!
//! Kapselt die Implementierung von `contextra_ports::{Checkpoint, CheckpointCoordinator}`
//! und entlastet den Kern von direkten Trait-Kopplungen.

use crate::meta::CheckpointMeta;
use crate::store::PersistentCheckpointStore;
use contextra_ports::BoxFuture;
use contextra_types::{Result, TxId, WorkflowState};

impl<S: contextra_ports::StorageEngine> contextra_ports::CheckpointCoordinator
    for PersistentCheckpointStore<S>
{
    type Meta = CheckpointMeta;

    async fn create_named_checkpoint(
        &self,
        name: &str,
        collection_id: &str,
        seq_no: u64,
        tx_id: TxId,
        metadata: serde_json::Value,
    ) -> Result<Self::Meta> {
        self.create_checkpoint(name, collection_id, seq_no, tx_id, metadata)
            .await
            .map_err(Into::into)
    }

    async fn restore_named_checkpoint(&self, name: &str) -> Result<Self::Meta> {
        self.restore_checkpoint(name).await.map_err(Into::into)
    }

    async fn drop_named_checkpoint(&self, name: &str) -> Result<()> {
        self.drop_checkpoint(name).await.map_err(Into::into)
    }

    async fn list_named_checkpoints(&self) -> Result<Vec<Self::Meta>> {
        self.list_checkpoints().await.map_err(Into::into)
    }
}

impl<S: contextra_ports::StorageEngine> contextra_ports::Checkpoint
    for PersistentCheckpointStore<S>
{
    fn take_snapshot<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<WorkflowState>> {
        Box::pin(async move {
            let seq_no = self.storage().last_seq_no().await?;
            Ok(WorkflowState {
                tx,
                graph_hash: *blake3::hash(format!("seq-{}", seq_no).as_bytes()).as_bytes(),
            })
        })
    }

    fn restore<'a>(&'a self, state: &'a WorkflowState) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.storage().rollback_to_tx(state.tx).await?;
            self.list_checkpoints().await?;
            Ok(())
        })
    }
}
