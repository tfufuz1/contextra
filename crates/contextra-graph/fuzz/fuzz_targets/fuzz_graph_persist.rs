//! # Fuzz-Target für CsrGraph Persistenz- und Deserialisierungs-Robustheit
//!
//! **Ring & Klasse:** Ring 0 (Fuzzing / REQ-QA-003)
//!
//! ## Zweck
//! Speist zufällige Byte-Arrays in die Deserialisierungs-Pfade von `CsrGraph::load_from_storage`
//! sowie die bincode-Payloads (`Entity`, `PersistedEdgePayload`) ein.
//! Garantiert: Keine Panics, keine unbegrenzten Allokationen, und bei Erfolg Roundtrip-Integrität.

#![no_main]

use arbitrary::Arbitrary;
use bytes::Bytes;
use contextra_graph::csr::{CsrGraph, PersistedEdgePayload};
use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
use contextra_types::{Entity, Result as ContextraResult, TxId};
use libfuzzer_sys::fuzz_target;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::ops::Bound;
use std::sync::Arc;

/// InMemory StorageEngine für isoliertes Fuzzing ohne Dateisystem-I/O.
#[derive(Clone)]
struct FuzzStorage {
    data: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl FuzzStorage {
    fn new(entries: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        let mut map = BTreeMap::new();
        // Begrenze Einträge auf maximal 100 zur Verhinderung von OOM beim Fuzzing
        for (k, v) in entries.into_iter().take(100) {
            // Begrenze Schlüssellänge und Wertlänge
            if k.len() <= 512 && v.len() <= 65536 {
                map.insert(k, v);
            }
        }
        Self {
            data: Arc::new(RwLock::new(map)),
        }
    }
}

impl StorageEngine for FuzzStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, ContextraResult<Option<Bytes>>> {
        Box::pin(async move {
            let guard = self.data.read();
            Ok(guard.get(key).map(|v| Bytes::copy_from_slice(v)))
        })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        _seq: u64,
    ) -> BoxFuture<'a, ContextraResult<Option<Bytes>>> {
        self.get(key)
    }

    fn put<'a>(
        &'a self,
        _tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move {
            let mut guard = self.data.write();
            guard.insert(key.to_vec(), value.to_vec());
            Ok(())
        })
    }

    fn put_if_absent<'a>(
        &'a self,
        _tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, ContextraResult<bool>> {
        Box::pin(async move {
            let mut guard = self.data.write();
            if guard.contains_key(key) {
                Ok(false)
            } else {
                guard.insert(key.to_vec(), value.to_vec());
                Ok(true)
            }
        })
    }

    fn delete<'a>(&'a self, _tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move {
            let mut guard = self.data.write();
            guard.remove(key);
            Ok(())
        })
    }

    fn commit<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, ContextraResult<StorageStats>> {
        Box::pin(async move {
            Ok(StorageStats {
                num_segments: 1,
                total_size_bytes: 0,
                memtable_size_bytes: 0,
            })
        })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, ContextraResult<u64>> {
        Box::pin(async move { Ok(0) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, ContextraResult<TxId>> {
        Box::pin(async move { Ok(TxId::new(1)) })
    }

    fn pin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn unpin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, ContextraResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, ContextraResult<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let guard = self.data.read();
            let mut results = Vec::new();
            for (key, val) in guard.range(prefix.to_vec()..) {
                if key.starts_with(prefix) {
                    results.push((key.clone(), val.clone()));
                } else {
                    break;
                }
            }
            Ok(results)
        })
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        _seq_no: u64,
    ) -> BoxFuture<'a, ContextraResult<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.scan_prefix(prefix)
    }

    fn scan<'a>(
        &'a self,
        _start: Bound<&'a [u8]>,
        _end: Bound<&'a [u8]>,
        _limit: Option<usize>,
    ) -> BoxFuture<'a, ContextraResult<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }
}

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    raw_entries: Vec<(Vec<u8>, Vec<u8>)>,
    raw_entity_bytes: Vec<u8>,
    raw_edge_bytes: Vec<u8>,
}

fuzz_target!(|input: FuzzInput| {
    // 1. Direct bincode deserialization robustness for Entity
    if let Ok(entity) = bincode::deserialize::<Entity>(&input.raw_entity_bytes) {
        // Bei erfolgreicher Deserialisierung: Roundtrip muss gelingen
        if let Ok(serialized) = bincode::serialize(&entity) {
            let roundtripped: Result<Entity, _> = bincode::deserialize(&serialized);
            assert!(
                roundtripped.is_ok(),
                "Roundtrip deserialization of valid entity must succeed"
            );
        }
    }

    // 2. Direct bincode deserialization robustness for PersistedEdgePayload
    if let Ok(payload) = bincode::deserialize::<PersistedEdgePayload>(&input.raw_edge_bytes) {
        if let Ok(serialized) = bincode::serialize(&payload) {
            let roundtripped: Result<PersistedEdgePayload, _> = bincode::deserialize(&serialized);
            assert!(
                roundtripped.is_ok(),
                "Roundtrip deserialization of valid payload must succeed"
            );
        }
    }

    // 3. Whole-graph storage reconstruction via CsrGraph::load_from_storage
    let storage = FuzzStorage::new(input.raw_entries);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();

    if let Ok(runtime) = rt {
        runtime.block_on(async {
            // Must never panic on arbitrary malformed bytes in storage
            let _ = CsrGraph::load_from_storage(&storage).await;
        });
    }
});
