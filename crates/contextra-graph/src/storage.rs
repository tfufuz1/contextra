//! Lightweight, standalone storage abstraction for `contextra-graph` persistence.

use std::future::Future;
use std::pin::Pin;

/// Boxed future for async storage methods.
pub type StorageFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Standalone key-value storage interface for persisting graph data.
pub trait GraphStorage: Send + Sync {
    /// Inserts or updates a key-value pair.
    fn put<'a>(&'a self, key: &'a [u8], value: &'a [u8]) -> StorageFuture<'a, Result<(), String>>;

    /// Retrieves the value associated with a key.
    fn get<'a>(&'a self, key: &'a [u8]) -> StorageFuture<'a, Result<Option<Vec<u8>>, String>>;

    /// Deletes a key-value pair.
    fn delete<'a>(&'a self, key: &'a [u8]) -> StorageFuture<'a, Result<(), String>>;

    /// Scans key-value pairs matching a given prefix.
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> StorageFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>, String>>;
}

/// Standalone in-memory storage implementation for standalone testing and benchmarks.
#[derive(Debug, Default, Clone)]
pub struct InMemoryGraphStorage {
    map: std::sync::Arc<parking_lot::RwLock<std::collections::BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl InMemoryGraphStorage {
    /// Creates a new empty `InMemoryGraphStorage`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl GraphStorage for InMemoryGraphStorage {
    fn put<'a>(&'a self, key: &'a [u8], value: &'a [u8]) -> StorageFuture<'a, Result<(), String>> {
        let key = key.to_vec();
        let value = value.to_vec();
        Box::pin(async move {
            self.map.write().insert(key, value);
            Ok(())
        })
    }

    fn get<'a>(&'a self, key: &'a [u8]) -> StorageFuture<'a, Result<Option<Vec<u8>>, String>> {
        let key = key.to_vec();
        Box::pin(async move {
            let res = self.map.read().get(&key).cloned();
            Ok(res)
        })
    }

    fn delete<'a>(&'a self, key: &'a [u8]) -> StorageFuture<'a, Result<(), String>> {
        let key = key.to_vec();
        Box::pin(async move {
            self.map.write().remove(&key);
            Ok(())
        })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> StorageFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>, String>> {
        let prefix = prefix.to_vec();
        Box::pin(async move {
            let guard = self.map.read();
            let matches = guard
                .range(prefix.clone()..)
                .take_while(|(k, _)| k.starts_with(&prefix))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Ok(matches)
        })
    }
}
