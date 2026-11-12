//! Bridge-Implementierung für Cognition auf contextra-db / contextra-engine.

use contextra_types::Result;

/// Brückenadapter zur Anbindung von Collections an Cognition-Pipelines.
pub struct DbDocumentSource<'a, S, V> {
    collection: &'a crate::collection::Collection<S, V>,
}

impl<'a, S, V> DbDocumentSource<'a, S, V> {
    /// Erstellt eine neue Instanz gebunden an eine Collection.
    pub fn new(collection: &'a crate::collection::Collection<S, V>) -> Self {
        Self { collection }
    }
}
