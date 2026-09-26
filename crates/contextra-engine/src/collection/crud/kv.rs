use super::internal::validate_doc_id;
use crate::collection::Collection;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::Result;

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Stores a non-vector key-value entry directly in LSM storage without touching vector, text, or graph indices.
    #[tracing::instrument(level = "trace", skip(self, value))]
    pub async fn put_kv(&self, id: &str, value: &serde_json::Value) -> Result<()> {
        validate_doc_id(id)?;
        let _guard = self.kv_locks.lock_for(id).await;
        drop(_guard);
        let tx = self.allocate_tx()?;
        let user_key = self.namespaced_key(id.as_bytes(), 0);
        let data = serde_json::to_vec(value)?;
        self.storage.put(tx, &user_key, &data).await?;
        self.storage.commit(tx).await?;
        Ok(())
    }

    /// Stores a non-vector key-value entry directly in LSM storage only if the key does not already exist.
    /// Returns `ContextraError::Conflict` if the key is already present.
    #[tracing::instrument(level = "trace", skip(self, value))]
    pub async fn put_kv_if_absent(&self, id: &str, value: &serde_json::Value) -> Result<()> {
        validate_doc_id(id)?;
        let _guard = self.kv_locks.lock_for(id).await;
        drop(_guard);
        let tx = self.allocate_tx()?;
        let user_key = self.namespaced_key(id.as_bytes(), 0);
        let data = serde_json::to_vec(value)?;
        let written = self.storage.put_if_absent(tx, &user_key, &data).await?;
        if !written {
            if let Err(rollback_err) = self.storage.rollback(tx).await {
                tracing::error!(
                    tx_id = ?tx,
                    key = %id,
                    error = %rollback_err,
                    "put_kv_if_absent: rollback after failed put_if_absent also failed — transaction may be left in an inconsistent state"
                );
            }
            return Err(contextra_types::ContextraError::Conflict(format!(
                "Key '{}' already exists in collection KV store",
                id
            )));
        }
        self.storage.commit(tx).await?;
        Ok(())
    }

    /// Retrieves a key-value entry directly from LSM storage.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_kv(&self, id: &str) -> Result<Option<serde_json::Value>> {
        validate_doc_id(id)?;
        let key = self.namespaced_key(id.as_bytes(), 0);
        if let Some(data) = self.storage.get(&key).await? {
            let val: serde_json::Value = serde_json::from_slice(&data)?;
            return Ok(Some(val));
        }
        Ok(None)
    }
}
