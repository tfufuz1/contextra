//! Adapter zur Anbindung von contextra-db Collections an das ContextStore-Port.

use crate::ports::ContextStore;
use async_trait::async_trait;
use contextra_types::Result;

/// Adapter für eine persistente contextra-db Collection.
pub struct DbCollectionStore<'a> {
    collection: &'a contextra_db::Collection,
}

impl<'a> DbCollectionStore<'a> {
    /// Erstellt eine neue Adapter-Instanz.
    pub fn new(collection: &'a contextra_db::Collection) -> Self {
        Self { collection }
    }
}

#[async_trait]
impl<'a> ContextStore for DbCollectionStore<'a> {
    async fn put_state(&self, key: &str, value: &[u8]) -> Result<()> {
        self.collection.put_kv(key, value).await
    }

    async fn get_state(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.collection.get_kv(key).await
    }

    async fn delete_state(&self, key: &str) -> Result<bool> {
        self.collection.delete_kv(key).await
    }
}
