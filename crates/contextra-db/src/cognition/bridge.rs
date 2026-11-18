//! Bridge-Implementierung für Cognition-Pipelines.

use contextra_types::Result;

use contextra_ports::{StorageEngine, VectorIndex};

/// Brückenadapter zur Anbindung von Collections an Cognition-Pipelines.
pub struct DbDocumentSource<'a, S: StorageEngine, V: VectorIndex> {
    collection: &'a crate::collection::Collection<S, V>,
}

impl<'a, S: StorageEngine, V: VectorIndex> DbDocumentSource<'a, S, V> {
    /// Erstellt eine neue Instanz gebunden an eine Collection.
    pub fn new(collection: &'a crate::collection::Collection<S, V>) -> Self {
        Self { collection }
    }
}
