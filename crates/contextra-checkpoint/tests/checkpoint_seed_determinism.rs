// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Integrationstest für Checkpoint-Determinismus (Zweimal gleicher Seed -> bitgleicher Checkpoint).

use contextra_checkpoint::PersistentCheckpointStore;
use contextra_ports::{IdGenerator, StorageEngine};
use contextra_testkit::{InMemoryStorageEngine, ManualClock};
use std::sync::atomic::{AtomicU64, Ordering};

struct SequentialIdGen {
    next: AtomicU64,
}

impl SequentialIdGen {
    fn new(start: u64) -> Self {
        Self {
            next: AtomicU64::new(start),
        }
    }
}

impl IdGenerator for SequentialIdGen {
    fn next_id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst)
    }
}
use contextra_types::TxId;
use std::sync::Arc;

#[tokio::test]
async fn test_same_seed_produces_bit_identical_checkpoint() {
    let seed = 42u64;
    let initial_nanos = 1_000_000_000u64; // 1 second

    // Run 1
    let clock1 = Arc::new(ManualClock::new(initial_nanos));
    let id_gen1 = Arc::new(SequentialIdGen::new(seed));
    let storage1 = Arc::new(InMemoryStorageEngine::new());
    let store1 = PersistentCheckpointStore::new(storage1.clone(), "test_ns")
        .unwrap()
        .with_clock(clock1)
        .with_id_generator(id_gen1);

    let meta1 = store1
        .create_checkpoint("cp_det", "col_1", 1, TxId::new(100), serde_json::json!({"version": 1}))
        .await
        .unwrap();

    // Run 2 (identical clock and id_generator seed)
    let clock2 = Arc::new(ManualClock::new(initial_nanos));
    let id_gen2 = Arc::new(SequentialIdGen::new(seed));
    let storage2 = Arc::new(InMemoryStorageEngine::new());
    let store2 = PersistentCheckpointStore::new(storage2.clone(), "test_ns")
        .unwrap()
        .with_clock(clock2)
        .with_id_generator(id_gen2);

    let meta2 = store2
        .create_checkpoint("cp_det", "col_1", 1, TxId::new(100), serde_json::json!({"version": 1}))
        .await
        .unwrap();

    // Metadata MUST be identical
    assert_eq!(meta1.name, meta2.name);
    assert_eq!(meta1.collection_id, meta2.collection_id);
    assert_eq!(meta1.seq_no, meta2.seq_no);
    assert_eq!(meta1.tx_id, meta2.tx_id);
    assert_eq!(meta1.created_at, meta2.created_at);
    assert_eq!(meta1.metadata, meta2.metadata);

    // Stored bytes MUST be bit-identical
    let key = "test_ns:checkpoint:cp_det";
    let bytes1 = storage1.get(key.as_bytes()).await.unwrap().unwrap();
    let bytes2 = storage2.get(key.as_bytes()).await.unwrap().unwrap();
    assert_eq!(bytes1, bytes2, "Two-stage checkpoint serialization with identical seeds MUST be bit-identical!");
}
