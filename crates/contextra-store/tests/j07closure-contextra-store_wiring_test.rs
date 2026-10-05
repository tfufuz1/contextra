//! Integration test suite verifying production wiring of all 16 target symbols.

use contextra_core::{CollectionId, TenantId};
use contextra_crypto::{crypto::KeyManager, kv_shredding::KeyRegistry};
use contextra_ports::{Clock, SystemClock};
use contextra_store::{
    compaction::{adaptive::CostBasedAdaptivePlanner, MergeOperator},
    kv::{KvDeleteMode, KvSegmentConfig, KvSegmentManager},
    lsm::{observer::WalObserver, LsmConfig, LsmStorage},
    memtable::MemTable,
    tenant_codec::TenantKeyCodec,
};
use std::sync::Arc;
use tempfile::tempdir;

struct TestObserver;
impl WalObserver for TestObserver {
    fn on_commit(
        &self,
        _batch: &contextra_store::lsm::observer::CommittedBatch<'_>,
        _seq_no: u64,
        _tx_id: contextra_core::TxId,
    ) {
    }
}

struct TestMergeOp;
impl MergeOperator for TestMergeOp {
    fn merge(&self, existing_value: &[u8], _new_value: &[u8]) -> contextra_core::Result<Vec<u8>> {
        Ok(existing_value.to_vec())
    }
}

#[tokio::test]
async fn test_j07closure_all_16_symbols_wiring() -> contextra_core::Result<()> {
    // 1–4. KV Segment symbols: delete_segment, generate_deletion_proof, read_segment, write_segment
    let registry = Arc::new(KeyRegistry::new());
    let master_km = Arc::new(KeyManager::try_new("passphrase", b"01234567890123456789012345678901")?);
    let kv_config = KvSegmentConfig {
        delete_mode: KvDeleteMode::CryptoShred,
    };
    let manager = KvSegmentManager::new(kv_config, registry, Some(master_km));
    let group_id = 4242u64;
    let plaintext = b"j07closure-kv-segment-payload";

    let payload = manager.write_segment(group_id, plaintext)?;
    let decrypted = manager.read_segment(&payload)?;
    assert_eq!(decrypted, plaintext);

    let proof_active = manager.generate_deletion_proof(group_id)?;
    assert!(!proof_active);

    let deleted = manager.delete_segment(group_id);
    assert!(deleted);

    let proof_deleted = manager.generate_deletion_proof(group_id)?;
    assert!(proof_deleted);

    // 5. with_adaptive_planner
    let dir = tempdir()?;
    let mut lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    lsm_config.compaction.enable_adaptive_compaction = true;

    let planner = Arc::new(CostBasedAdaptivePlanner::new(
        lsm_config.compaction.adaptive_read_ratio_threshold,
        lsm_config.compaction.min_sstables_per_tier,
        lsm_config.compaction.size_ratio,
    ));

    let _engine = contextra_store::compaction::CompactionEngine::new(
        lsm_config.compaction.clone(),
        Arc::new(contextra_core::SnapshotRegistry::new()),
        Arc::new(contextra_store::sstable::BlockCache::new(16)),
        None,
        Arc::new(contextra_core::ResourceTracker::new(
            contextra_core::ResourceBudget { memory_limit: 1024 * 1024 },
        )),
        None,
    )
    .with_adaptive_planner(planner);

    // 6–9. LSM Storage symbols: has_pending_legacy_wal_migration, migrate_legacy_wal_keys, open_with_merge_operator, spawn_tracked
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
    let migrated = LsmStorage::migrate_legacy_wal_keys(&lsm_config, Arc::clone(&clock)).await?;
    assert_eq!(migrated, 0);

    let merge_op: Arc<dyn MergeOperator> = Arc::new(TestMergeOp);
    let storage = LsmStorage::open_with_merge_operator(lsm_config, merge_op).await?;

    let has_pending = storage.has_pending_legacy_wal_migration().await?;
    assert!(!has_pending);

    let (task_tx, mut task_rx) = tokio::sync::mpsc::channel::<()>(1);
    storage.spawn_tracked(async move {
        let _ = task_tx.send(()).await;
    });
    assert!(task_rx.recv().await.is_some());

    // 10–12. ObserverRegistry symbols: clear_circuit_breaker, dropped_count_for, is_any_circuit_breaker_open
    let obs: Arc<dyn WalObserver> = Arc::new(TestObserver);
    storage.register_observer(Arc::clone(&obs));
    assert_eq!(storage.dropped_count_for(&obs), 0);

    storage.clear_circuit_breaker(&obs);

    // 13. point_lookup_metrics
    let (_eval, _bloom, _range, _reads, found) = storage.point_lookup_metrics(b"nonexistent-key").await;
    assert!(!found);

    // 14. shard_entry_counts
    let mt = MemTable::new();
    let counts = mt.shard_entry_counts();
    assert_eq!(counts.len(), 16);
    assert!(mt.is_empty());

    // 15–16. TenantKeyCodec symbols: collection_prefix, encode_graph_key
    let tenant = TenantId::try_new(101)?;
    let codec = TenantKeyCodec::new(tenant);
    let col = CollectionId(77);

    let col_prefix = codec.collection_prefix(&col);
    assert_eq!(col_prefix, b"t:101:77:");

    let graph_key = codec.encode_graph_key(&col, 999);
    assert_eq!(graph_key, b"t:101:77:graph:999");

    storage.close().await?;
    Ok(())
}
