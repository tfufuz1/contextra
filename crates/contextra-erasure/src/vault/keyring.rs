//! InMemoryKeyring für Dateischlüssel nach SPEC D-85, PO-06.

use contextra_ports::keys::{FileId, FileKey, FileKeyring, FileKind};
use contextra_ports::BoxFuture;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

/// In-Memory-Implementierung von `FileKeyring` für Tests und Validierung.
#[derive(Clone, Default)]
pub struct InMemoryKeyring {
    keys: Arc<RwLock<BTreeMap<FileId, Vec<u8>>>>,
}

impl InMemoryKeyring {
    /// Erstellt einen neuen `InMemoryKeyring`.
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl FileKeyring for InMemoryKeyring {
    fn create(&self, _kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey, String>> {
        Box::pin(async move {
            let mut key_bytes = vec![0u8; 32];
            let hash = blake3::hash(&id.as_u64().to_le_bytes());
            key_bytes.copy_from_slice(hash.as_bytes());

            let mut lock = self.keys.write();
            lock.insert(id, key_bytes.clone());
            Ok(FileKey::new(key_bytes, id))
        })
    }

    fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey, String>> {
        Box::pin(async move {
            let lock = self.keys.read();
            match lock.get(&id) {
                Some(bytes) => Ok(FileKey::new(bytes.clone(), id)),
                None => Err("FileKey destroyed or missing (erased)".into()),
            }
        })
    }

    fn destroy(&self, id: FileId) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            let mut lock = self.keys.write();
            if let Some(mut existing) = lock.remove(&id) {
                existing.fill(0);
            }
            Ok(())
        })
    }
}
