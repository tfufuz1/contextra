//! Test helpers and in-memory test doubles for `contextra-graph` (Klasse A, B2).
//!
//! Replaces internal dev-dependency on `contextra-testkit` with self-contained
//! implementations compatible with `contextra_ports::StorageEngine`.

#[cfg(feature = "types-compat")]
use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
#[cfg(feature = "types-compat")]
use contextra_types::{Result, TxId};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Simple in-memory fault-injecting VFS double for crash simulation tests.
#[derive(Debug, Default, Clone)]
pub struct FaultVfs {
    files: Arc<RwLock<BTreeMap<std::path::PathBuf, Vec<u8>>>>,
    failing: Arc<parking_lot::Mutex<bool>>,
}

/// Fault configuration for crash simulation.
#[derive(Debug, Default, Clone)]
pub struct FaultConfig {
    /// If true, all file operations fail.
    pub fail_all: bool,
}

impl FaultVfs {
    /// Creates a new `FaultVfs`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the fault injection config.
    pub fn set_config(&self, config: FaultConfig) {
        *self.failing.lock() = config.fail_all;
    }

    /// Triggers an immediate crash state.
    pub fn trigger_crash(&self) {
        *self.failing.lock() = true;
    }

    /// Resets fault counters and failure state.
    pub fn reset_counters(&self) {
        *self.failing.lock() = false;
    }

    /// Writes data to a virtual path.
    pub fn write_file(
        &self,
        path: &std::path::Path,
        data: &[u8],
    ) -> std::result::Result<(), String> {
        if *self.failing.lock() {
            return Err("Simulated I/O fault".to_string());
        }
        self.files.write().insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    /// Reads data from a virtual path.
    pub fn read_file(&self, path: &std::path::Path) -> std::result::Result<Vec<u8>, String> {
        if *self.failing.lock() {
            return Err("Simulated I/O fault".to_string());
        }
        self.files
            .read()
            .get(path)
            .cloned()
            .ok_or_else(|| "Not found".to_string())
    }

    /// Flushes virtual files to disk.
    pub fn sync(&self) -> std::result::Result<(), String> {
        if *self.failing.lock() {
            return Err("Simulated fsync fault".to_string());
        }
        Ok(())
    }
}

/// In-memory storage engine double implementing `contextra_ports::StorageEngine`.
#[derive(Debug, Default, Clone)]
pub struct InMemoryStorageEngine {
    data: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>,
    last_tx: Arc<AtomicU64>,
}

impl InMemoryStorageEngine {
    /// Creates a new empty `InMemoryStorageEngine`.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "types-compat")]
impl StorageEngine for InMemoryStorageEngine {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        let key = key.to_vec();
        Box::pin(async move {
            let res = self.data.read().get(&key).cloned().map(bytes::Bytes::from);
            Ok(res)
        })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        _seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        self.get(key)
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        let key = key.to_vec();
        let value = value.to_vec();
        self.last_tx.fetch_max(tx_id.inner(), Ordering::SeqCst);
        Box::pin(async move {
            self.data.write().insert(key, value);
            Ok(())
        })
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        let key = key.to_vec();
        self.last_tx.fetch_max(tx_id.inner(), Ordering::SeqCst);
        Box::pin(async move {
            self.data.write().remove(&key);
            Ok(())
        })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        let prefix = prefix.to_vec();
        Box::pin(async move {
            let guard = self.data.read();
            let matches = guard
                .range(prefix.clone()..)
                .take_while(|(k, _)| k.starts_with(&prefix))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Ok(matches)
        })
    }

    fn scan<'a>(
        &'a self,
        _start: std::ops::Bound<&'a [u8]>,
        _end: std::ops::Bound<&'a [u8]>,
        _limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.last_tx.fetch_max(tx_id.inner(), Ordering::SeqCst);
        Box::pin(async move { Ok(()) })
    }

    fn rollback<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        Box::pin(async move {
            Ok(StorageStats {
                num_segments: 0,
                total_size_bytes: 0,
                memtable_size_bytes: 0,
            })
        })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        let val = self.last_tx.load(Ordering::SeqCst);
        Box::pin(async move { Ok(val) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        let val = self.last_tx.load(Ordering::SeqCst);
        Box::pin(async move { Ok(TxId::new(val)) })
    }

    fn pin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn unpin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
}
