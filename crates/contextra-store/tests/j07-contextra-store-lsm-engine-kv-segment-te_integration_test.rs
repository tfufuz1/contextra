//! Integration test file verifying reachability and correctness of 22 audit-flagged symbols
//! in `contextra-store`.
//!
//! Symbol list (22):
//! 1. `delete_segment`
//! 2. `generate_deletion_proof`
//! 3. `read_segment`
//! 4. `write_segment`
//! 5. `with_adaptive_planner`
//! 6. `has_pending_legacy_wal_migration`
//! 7. `migrate_legacy_wal_keys`
//! 8. `open_with_merge_operator`
//! 9. `pressure_receiver`
//! 10. `spawn_tracked`
//! 11. `force_flush`
//! 12. `clear_circuit_breaker`
//! 13. `dropped_count_for`
//! 14. `is_any_circuit_breaker_open`
//! 15. `point_lookup_metrics`
//! 16. `scan_range_into`
//! 17. `shard_entry_counts`
//! 18. `tx_range`
//! 19. `collection_prefix`
//! 20. `decode_tenant_id`
//! 21. `encode_chunk_key`
//! 22. `encode_graph_key`

use bytes::Bytes;
use contextra_core::{
    CollectionId, ContextraError, DocId, ResourceBudget, ResourceTracker, SnapshotRegistry,
    StorageEngine, TenantId, TxId,
};
use contextra_crypto::{crypto::KeyManager, kv_shredding::KeyRegistry};
use contextra_ports::SystemClock;
use contextra_store::{
    compaction::{
        adaptive::{AdaptiveCompactionPlan, AdaptiveCompactionPlanner, WorkloadMetricsSnapshot},
        CompactionConfig, CompactionEngine, MergeOperator,
    },
    kv::{KvDeleteMode, KvSegmentConfig, KvSegmentManager},
    lsm::{
        AsyncObserverAdapter, CommittedBatch, LsmConfig, LsmStorage, ObserverRegistry, WalObserver,
    },
    memtable::MemTable,
    sstable::BlockCache,
    tenant_codec::TenantKeyCodec,
};
use std::{collections::BTreeMap, ops::Bound, sync::Arc, time::Duration};
use tempfile::TempDir;

struct TestMergeOperator;

impl MergeOperator for TestMergeOperator {
    fn merge(&self, existing_val: &[u8], new_val: &[u8]) -> Result<Vec<u8>, ContextraError> {
        let mut combined = existing_val.to_vec();
        combined.extend_from_slice(b"+");
        combined.extend_from_slice(new_val);
        Ok(combined)
    }
}

struct DummyAdaptivePlanner;

impl AdaptiveCompactionPlanner for DummyAdaptivePlanner {
    fn plan_compaction(
        &self,
        _stats: &contextra_core::StorageStats,
        _metrics: &WorkloadMetricsSnapshot,
        _sstables: &[Arc<contextra_store::sstable::SstableReader>],
        _min_active_seqno: u64,
    ) -> Result<Option<AdaptiveCompactionPlan>, ContextraError> {
        Ok(None)
    }
}

struct SlowWalObserver;

impl WalObserver for SlowWalObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[tokio::test]
async fn test_kv_segment_manager_symbols() {
    let registry = Arc::new(KeyRegistry::new());
    let salt = [0u8; 32];
    let master_km = Arc::new(KeyManager::try_new("passphrase-j07", &salt).unwrap());

    let config = KvSegmentConfig {
        delete_mode: KvDeleteMode::CryptoShred,
    };
    let manager = KvSegmentManager::new(config, Arc::clone(&registry), Some(master_km));

    let group_id = 1001u64;
    let plaintext = b"sensitive payload";

    // Symbol 4: write_segment
    let payload = manager
        .write_segment(group_id, plaintext)
        .expect("write_segment should succeed");
    assert!(payload.is_encrypted);

    // Symbol 3: read_segment
    let read_back = manager
        .read_segment(&payload)
        .expect("read_segment should decrypt successfully");
    assert_eq!(read_back, plaintext);

    // Symbol 2: generate_deletion_proof (before delete)
    let proof_before = manager
        .generate_deletion_proof(group_id)
        .expect("generate_deletion_proof should succeed");
    assert!(!proof_before, "Key is active, proof_before should be false");

    // Symbol 1: delete_segment
    let deleted = manager.delete_segment(group_id);
    assert!(deleted, "delete_segment should revoke key");

    // Symbol 2: generate_deletion_proof (after delete)
    let proof_after = manager
        .generate_deletion_proof(group_id)
        .expect("generate_deletion_proof should succeed");
    assert!(proof_after, "Key revoked, proof_after should be true");

    // Decryption fails after revocation
    assert!(manager.read_segment(&payload).is_err());
}

#[test]
fn test_compaction_engine_with_adaptive_planner_symbol() {
    let snapshot_registry = Arc::new(SnapshotRegistry::new());
    let block_cache = Arc::new(BlockCache::new(1024 * 1024));
    let budget = Arc::new(ResourceTracker::new(ResourceBudget {
        memory_limit: 10 * 1024 * 1024,
    }));
    let config = CompactionConfig::default();

    let engine = CompactionEngine::new(config, snapshot_registry, block_cache, None, budget, None);

    // Symbol 5: with_adaptive_planner
    let planner: Arc<dyn AdaptiveCompactionPlanner> = Arc::new(DummyAdaptivePlanner);
    let _engine_with_planner = engine.with_adaptive_planner(planner);
}

#[tokio::test]
async fn test_lsm_storage_and_observer_symbols() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let lsm_config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    let clock = Arc::new(SystemClock::new());

    // Symbol 6: has_pending_legacy_wal_migration
    // Symbol 7: migrate_legacy_wal_keys
    let migrated = LsmStorage::migrate_legacy_wal_keys(
        &lsm_config,
        Arc::clone(&clock) as Arc<dyn contextra_ports::Clock>,
    )
    .await
    .expect("migrate_legacy_wal_keys should succeed on empty dir");
    assert_eq!(migrated, 0);

    // Symbol 8: open_with_merge_operator
    let merge_op: Arc<dyn MergeOperator> = Arc::new(TestMergeOperator);
    let storage = LsmStorage::open_with_merge_operator(lsm_config, merge_op)
        .await
        .expect("open_with_merge_operator should succeed");

    let has_legacy = storage
        .has_pending_legacy_wal_migration()
        .await
        .expect("has_pending_legacy_wal_migration should succeed");
    assert!(!has_legacy);

    // Symbol 9: pressure_receiver
    let _pressure_rx = storage.pressure_receiver();

    // Symbol 10: spawn_tracked
    let (tx, rx) = tokio::sync::oneshot::channel();
    storage.spawn_tracked(async move {
        let _ = tx.send(true);
    });
    assert!(rx.await.unwrap());

    // Symbol 11: force_flush
    storage
        .force_flush()
        .await
        .expect("force_flush should succeed");

    // Symbol 15: point_lookup_metrics
    let tx_id = TxId(101);
    storage.put(tx_id, b"key1", b"val1").await.unwrap();
    storage.commit(tx_id).await.unwrap();
    storage.force_flush().await.unwrap();

    let (evaluated_sstables, _bloom, _range, _blocks, found) =
        storage.point_lookup_metrics(b"key1").await;
    assert!(found);
    assert!(evaluated_sstables >= 1);

    storage.close().await.unwrap();
}

#[tokio::test]
async fn test_observer_registry_symbols() {
    let registry = ObserverRegistry::new();
    registry.set_max_observer_latency(Duration::from_millis(10));

    let obs: Arc<dyn WalObserver> = Arc::new(SlowWalObserver);
    let (adapted, _handle) = AsyncObserverAdapter::new(Arc::clone(&obs), 2);
    let adapter_arc: Arc<dyn WalObserver> = Arc::new(adapted);
    registry.register_observer(Arc::clone(&adapter_arc));

    // Symbol 14: is_any_circuit_breaker_open (initially false)
    assert!(!registry.is_any_circuit_breaker_open());

    // Symbol 13: dropped_count_for
    let dropped = registry.dropped_count_for(&adapter_arc);
    assert_eq!(dropped, 0);

    // Symbol 12: clear_circuit_breaker
    registry.clear_circuit_breaker(&adapter_arc);
    assert!(!registry.is_any_circuit_breaker_open());
}

#[test]
fn test_memtable_symbols() {
    let mt = MemTable::new();

    // Symbol 18: tx_range (empty memtable initialized to (u64::MAX, 0))
    let range = mt.tx_range();
    assert_eq!(range, (u64::MAX, 0));

    // Symbol 17: shard_entry_counts
    let counts = mt.shard_entry_counts();
    assert_eq!(counts.len(), 16);

    mt.put(Bytes::from("k1"), Bytes::from("v1"), 10, 1);

    let range_after = mt.tx_range();
    assert_eq!(range_after, (1, 1));

    // Symbol 16: scan_range_into
    let mut target = BTreeMap::new();
    mt.scan_range_into(
        Bound::Included(b"k1".as_slice()),
        Bound::Included(b"k2".as_slice()),
        100,
        TxId(10),
        &mut target,
    );
    assert_eq!(target.len(), 1);
    assert_eq!(
        target.get(&Bytes::from("k1")),
        Some(&(Bytes::from("v1"), 10))
    );
}

#[test]
fn test_tenant_key_codec_symbols() {
    let tenant_id = TenantId::try_new(42).unwrap();
    let codec = TenantKeyCodec::new(tenant_id);
    let col = CollectionId(10);

    // Symbol 19: collection_prefix
    let col_prefix = codec.collection_prefix(&col);
    assert_eq!(col_prefix, b"t:42:10:");

    // Symbol 21: encode_chunk_key
    let chunk_key = codec.encode_chunk_key(&col, DocId(99));
    assert_eq!(chunk_key, b"t:42:10:chunk:99");

    // Symbol 22: encode_graph_key
    let graph_key = codec.encode_graph_key(&col, 555);
    assert_eq!(graph_key, b"t:42:10:graph:555");

    // Symbol 20: decode_tenant_id
    let decoded = TenantKeyCodec::decode_tenant_id(&chunk_key);
    assert_eq!(decoded, Some(tenant_id));

    let invalid_decoded = TenantKeyCodec::decode_tenant_id(b"invalid_key");
    assert_eq!(invalid_decoded, None);
}
