//! Adapter zur Anbindung von contextra-db Collections an das ContextStore-Port.
//!
//! Die Collection speichert JSON-Werte, der Port arbeitet mit rohen Bytes. Bytes werden
//! als JSON-Array von `u8` abgebildet. Diese Kodierung ist verlustfrei und ohne Zusatz-
//! abhängigkeit umkehrbar.

use crate::ports::ContextStore;
use contextra_ports::BoxFuture;
use contextra_types::{ContextraError, Result};

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

impl<'a> ContextStore for DbCollectionStore<'a> {
    fn put_state<'b>(&'b self, key: &'b str, value: &'b [u8]) -> BoxFuture<'b, Result<()>> {
        Box::pin(async move {
            let encoded = serde_json::to_value(value)
                .map_err(|e| ContextraError::Serialization(e.to_string()))?;
            self.collection.put_kv(key, &encoded).await
        })
    }

    fn get_state<'b>(&'b self, key: &'b str) -> BoxFuture<'b, Result<Option<Vec<u8>>>> {
        Box::pin(async move {
            match self.collection.get_kv(key).await? {
                None => Ok(None),
                Some(value) => serde_json::from_value::<Vec<u8>>(value)
                    .map(Some)
                    .map_err(|e| ContextraError::Serialization(e.to_string())),
            }
        })
    }

    fn delete_state<'b>(&'b self, key: &'b str) -> BoxFuture<'b, Result<bool>> {
        Box::pin(async move {
            // Die Collection meldet keinen Löschstatus. Vorhandensein wird vorab geprüft.
            let existed = self.collection.get_kv(key).await?.is_some();
            self.collection.delete(key).await?;
            Ok(existed)
        })
    }
}
