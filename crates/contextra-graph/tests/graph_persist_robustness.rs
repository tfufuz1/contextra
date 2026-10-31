//! Robustness tests for CsrGraph persistence deserialization (`graph_persist.rs`).
//!
//! Tests deterministic garbage, truncation, and bit-flip inputs on stable toolchain:
//! - Malformed entity records in storage
//! - Truncated edge payloads and invalid UTF-8 edge keys
//! - Bit flips across valid serialized entity & edge payloads
//! - Recovery and graceful handling (no panics, fail-closed / skip corrupt records).

use bytes::Bytes;
use contextra_graph::csr::{CsrGraph, PersistedEdgePayload};
use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
use contextra_types::{Entity, EntityId, Result as ContextraResult, TxId};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::ops::Bound;
use std::sync::Arc;

#[derive(Clone, Default)]
struct MockStorage {
    data: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl MockStorage {
    fn new() -> Self {
        Self::default()
    }

    fn insert(&self, key: &[u8], value: &[u8]) {
        self.data.write().insert(key.to_vec(), value.to_vec());
    }
}

impl StorageEngine for MockStorage {
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
            self.data.write().insert(key.to_vec(), value.to_vec());
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
            self.data.write().remove(key);
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

#[tokio::test]
async fn test_load_from_storage_handles_corrupt_entity_bincode() {
    let storage = MockStorage::new();

    // Insert a valid entity
    let entity = Entity::new(EntityId::new(1), "Alice", "Person");
    let valid_bytes = bincode::serialize(&entity).unwrap();
    storage.insert(b"__graph:entity:1", &valid_bytes);

    // Insert a corrupted entity (random garbage bytes)
    storage.insert(b"__graph:entity:2", b"\xff\xff\xff\x00\x12\x34\xaa\xbb");

    // Loading should fail gracefully with an Error (fail-closed on corrupt entity) and NOT panic
    let result = CsrGraph::load_from_storage(&storage).await;
    assert!(
        result.is_err(),
        "Corrupt entity must return an error without panicking"
    );
}

#[tokio::test]
async fn test_load_from_storage_handles_corrupt_edge_key() {
    let storage = MockStorage::new();

    // 1. Valid entity
    let e1 = Entity::new(EntityId::new(1), "E1", "Node");
    let e2 = Entity::new(EntityId::new(2), "E2", "Node");
    storage.insert(b"__graph:entity:1", &bincode::serialize(&e1).unwrap());
    storage.insert(b"__graph:entity:2", &bincode::serialize(&e2).unwrap());

    // 2. Corrupt edge key (non-UTF8 or missing colon)
    let payload = PersistedEdgePayload {
        weight: 1.0,
        tx_valid_from: None,
        tx_valid_to: None,
        business_valid_from: None,
        business_valid_to: None,
        source_doc_id: None,
    };
    let payload_bytes = bincode::serialize(&payload).unwrap();

    // Missing colon separator
    storage.insert(b"__graph:edge:1and2withoutcolon", &payload_bytes);

    // Invalid UTF-8 key
    storage.insert(b"__graph:edge:\xff\xfe:2", &payload_bytes);

    // Invalid edge key should either return an error or skip invalid key gracefully without panicking
    let _ = CsrGraph::load_from_storage(&storage).await;
}

#[tokio::test]
async fn test_load_from_storage_handles_truncated_and_legacy_edge_payloads() {
    let storage = MockStorage::new();

    let e1 = Entity::new(EntityId::new(10), "N10", "Node");
    let e2 = Entity::new(EntityId::new(20), "N20", "Node");
    storage.insert(b"__graph:entity:10", &bincode::serialize(&e1).unwrap());
    storage.insert(b"__graph:entity:20", &bincode::serialize(&e2).unwrap());

    // 1. Truncated edge payload (e.g. only 2 bytes instead of struct)
    storage.insert(b"__graph:edge:10:20", b"\x01\x02");

    // Should return Err on completely malformed edge payload without panicking
    let res = CsrGraph::load_from_storage(&storage).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_bit_flip_fuzz_simulation_on_serialized_payloads() {
    let entity = Entity::new(EntityId::new(50), "ReliabilityTest", "Component");
    let serialized = bincode::serialize(&entity).unwrap();

    // Flip each bit in the serialized payload
    for byte_idx in 0..serialized.len() {
        for bit in 0..8 {
            let mut mutated = serialized.clone();
            mutated[byte_idx] ^= 1 << bit;

            // Deserialization must either succeed (yielding an Entity) or return an Err, never panic
            let _ = bincode::deserialize::<Entity>(&mutated);
        }
    }

    let payload = PersistedEdgePayload {
        weight: 2.5,
        tx_valid_from: Some(TxId::new(100)),
        tx_valid_to: None,
        business_valid_from: Some(1700000000),
        business_valid_to: None,
        source_doc_id: None,
    };
    let edge_bytes = bincode::serialize(&payload).unwrap();

    for byte_idx in 0..edge_bytes.len() {
        for bit in 0..8 {
            let mut mutated = edge_bytes.clone();
            mutated[byte_idx] ^= 1 << bit;

            let _ = bincode::deserialize::<PersistedEdgePayload>(&mutated);
        }
    }
}
