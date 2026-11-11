//! Lokale Schnittstellen und Standardimplementierungen für den Tier-2 Spill-Speicher.

use crate::error::Result;
use crate::types::TxId;
use bytes::Bytes;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Dyn-kompatibler lokaler Future-Typ ohne externe Laufzeitabhängigkeit.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// ANCHOR[contextra-kvcache:SPILL-001]
/// Lokales Persistenz-Interface für die Tier-2-Auslagerung von KV-Segmenten.
pub trait KvSpillStorage: Send + Sync {
    /// Liest einen Wert unter dem angegebenen Schlüssel.
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>;

    /// Schreibt ein Schlüssel-Wert-Paar im Kontext einer Transaktion (`tx_id`).
    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    /// Bestätigt die Transaktion mit der ID `tx_id`.
    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
}

// ANCHOR[contextra-kvcache:SPILL-002]
/// Deterministischer In-Memory-Spill-Speicher als Adoptions-Baustein für Standalone-Setups.
#[derive(Debug, Default)]
pub struct InMemoryKvSpillStorage {
    data: Arc<RwLock<BTreeMap<Vec<u8>, Bytes>>>,
    staging: Arc<RwLock<BTreeMap<TxId, Vec<(Vec<u8>, Bytes)>>>>,
}

impl InMemoryKvSpillStorage {
    /// Erstellt eine neue Instanz des In-Memory Spill-Speichers.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gibt die Anzahl gespeicherter persistenter Einträge zurück.
    pub fn len(&self) -> usize {
        self.data.read().len()
    }

    /// Prüft, ob der Speicher leer ist.
    pub fn is_empty(&self) -> bool {
        self.data.read().is_empty()
    }
}

impl KvSpillStorage for InMemoryKvSpillStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        let key_vec = key.to_vec();
        let data = Arc::clone(&self.data);
        Box::pin(async move {
            let guard = data.read();
            Ok(guard.get(&key_vec).cloned())
        })
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        let key_vec = key.to_vec();
        let val_bytes = Bytes::copy_from_slice(value);
        let staging = Arc::clone(&self.staging);
        Box::pin(async move {
            let mut guard = staging.write();
            guard.entry(tx_id).or_default().push((key_vec, val_bytes));
            Ok(())
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        let staging = Arc::clone(&self.staging);
        let data = Arc::clone(&self.data);
        Box::pin(async move {
            let staged = {
                let mut guard = staging.write();
                guard.remove(&tx_id)
            };
            if let Some(entries) = staged {
                let mut store = data.write();
                for (k, v) in entries {
                    if v.is_empty() {
                        store.remove(&k);
                    } else {
                        store.insert(k, v);
                    }
                }
            }
            Ok(())
        })
    }
}
