// FILE-CONTEXT
// ZWECK: MemFuse Database Strangler Shell / Re-export Facade (Layer 3).
// INVARIANTEN: Backward compatibility for all public types; re-exports from memfuse-engine and memfuse-cognition.

#![forbid(unsafe_code)]

pub use memfuse_cognition as cognition;
pub use memfuse_engine as engine;

// Re-exports from memfuse-engine
pub use memfuse_engine::{
    background_workers, chunker, collection, export, filter, import, temporal_filter, transaction,
};
#[allow(deprecated)]
pub use memfuse_engine::MetadataFilter;
pub use memfuse_engine::{
    CommunityDetectionConfig, DbStats, Document, EmbeddingBackend, ExportCollectionV1,
    ExportDocumentV1, ExportMemoryV1, ExportRelationV1, HybridQueryBuilder, ImportSummary,
    Language, MemFuse, MemFuseConfig, MemFuseStats, ProvenanceRecord, SearchResult,
    SearchStrategy, SignalContribution, SignalWeights, MAX_SCAN_RESULTS, SCHEMA_VERSION_V1,
};

#[cfg(feature = "graph-connectivity-health")]
pub use memfuse_engine::collection::maintenance::PercolationResult;
#[cfg(feature = "sandbox")]
pub use memfuse_engine::SandboxBridge;

// Re-exports from memfuse-cognition
#[allow(deprecated)]
pub use memfuse_cognition::execute_sleep_cycle;
pub use memfuse_cognition::{
    cleanup_orphaned_consolidation_intents, compact_segment_via_context_compactor,
    compute_community_hash, detect_near_duplicates, execute_background_consolidation,
    execute_consolidation_pass, group_turns_into_segments,
    run_consolidation_pass, run_synthesis_pass, CommunityStabilityTracker, CompactedContext,
    CompactionStrategy, ConsolidationConfig, ConsolidationEngine, ConsolidationNodesGuard,
    ConsolidationPhaseResult, ConsolidationSession, ContextCompactor, ContextManager,
    MaintenanceConfig, MaintenanceScheduler, MetaChunk, SpatialFence, StatusToken, SynthesisConfig,
    SynthesisPhaseResult, TurnSegment,
};

pub mod context {
    pub use memfuse_cognition::context::*;
}
pub mod context_compaction {
    pub use memfuse_cognition::context_compaction::*;
}
pub mod memory_consolidation {
    pub use memfuse_cognition::memory_consolidation::*;
}
pub mod synthesis_phase {
    pub use memfuse_cognition::synthesis_phase::*;
}
pub mod consolidation_executor {
    pub use memfuse_cognition::consolidation_executor::*;
}
pub mod consolidation_locks {
    pub use memfuse_cognition::consolidation_locks::*;
}
pub mod maintenance_scheduler {
    pub use memfuse_cognition::maintenance_scheduler::*;
}
pub mod maintenance_config {
    pub use memfuse_cognition::maintenance_config::*;
}

#[deprecated(note = "use background_workers instead")]
pub mod reaper {
    pub use memfuse_engine::background_workers::*;
}
#[deprecated(note = "use start_consolidation_worker instead")]
#[allow(deprecated)]
pub use memfuse_cognition::start_consolidation_reaper;

// mod Collection is used via pub mod collection
pub mod decay_controller {
    pub use memfuse_adapt::decay_controller::*;
}
pub use memfuse_engine::fusion;
pub mod homeostat {
    pub use memfuse_adapt::homeostat::*;
}
pub mod multistep;
pub mod pid_latency_controller {
    pub use memfuse_adapt::pid_latency_controller::*;
}

#[cfg(feature = "volatile-vault")]
pub mod volatile_vault;
#[cfg(feature = "volatile-vault")]
pub use volatile_vault::{
    CommitReceipt, PurgeReceipt, SignalModality, VaultChunk, VaultChunkMetadata, VaultConfig,
    VaultError, VolatileContextVault,
};

pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
#[allow(deprecated)]
pub use homeostat::{pid_regulated_candidate_pool, RerankDeadline, RerankPidController};
pub use multistep::{MultiStepConfig, MultiStepEngine, MultiStepResult, QueryRewriter};
pub use pid_latency_controller::{
    LatencyBudgetGuard, PidLatencyController, DEFAULT_TARGET_LATENCY_MS, MAX_SCALING_FACTOR,
    MIN_SCALING_FACTOR,
};

pub use collection::{Collection, CollectionConfig};
pub use memfuse_checkpoint;
#[cfg(feature = "graph-connectivity-health")]
pub use memfuse_graph::percolation::PercolationConfig;

pub use memfuse_core::DistanceMetric;
pub use memfuse_core::DriftStatusProvider;
pub use memfuse_core::SegmentSynthesizer;
pub use memfuse_core::TextEmbeddingEngine;
pub use serde_json::json;

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use memfuse_core::types::TenantId;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn test_db(dim: usize) -> (MemFuse, TempDir) {
        let tmp = TempDir::new().expect("temp dir"); // expect
        let config = MemFuseConfig {
            dimension: dim,
            max_elements: 10_000,
            distance_metric: DistanceMetric::Cosine,
            ..Default::default()
        };
        let db = MemFuse::open_with_config(tmp.path(), config)
            .await
            .expect("open db"); // expect
        (db, tmp)
    }

    #[tokio::test]
    async fn test_insert_search_roundtrip() {
        let (db, _tmp) = test_db(4).await;

        db.insert(
            "doc-1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"topic": "rust"})),
        )
        .await
        .expect("insert"); // expect

        db.insert(
            "doc-2",
            &[0.0, 1.0, 0.0, 0.0],
            Some(json!({"topic": "python"})),
        )
        .await
        .expect("insert"); // expect

        db.insert("doc-3", &[0.9, 0.1, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect

        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 2).await.expect("search"); // expect
        assert_eq!(results.len(), 2);
        // doc-1 should be closest
        assert!(results[0].score > results[1].score);
    }

    #[tokio::test]
    async fn test_insert_search_returns_metadata() {
        let (db, _tmp) = test_db(4).await;

        db.insert(
            "doc-1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"topic": "rust", "priority": 1})),
        )
        .await
        .expect("insert"); // expect

        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 1).await.expect("search"); // expect
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc-1");
        let meta = results[0].metadata.as_ref().expect("metadata should exist"); // expect
        assert_eq!(meta["topic"], "rust");
        assert_eq!(meta["priority"], 1);
    }

    #[tokio::test]
    async fn test_get_by_key() {
        let (db, _tmp) = test_db(4).await;

        db.insert(
            "doc-1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"topic": "rust"})),
        )
        .await
        .expect("insert"); // expect

        let doc = db.get("doc-1").await.expect("get").expect("should exist"); // expect
        assert_eq!(doc.id, "doc-1");
        assert_eq!(doc.metadata.expect("valid")["topic"], "rust"); // expect

        let none = db.get("nonexistent").await.expect("get"); // expect
        assert!(none.is_none());
    }

    #[tokio::test]
    async fn test_update() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"v": 1})))
            .await
            .expect("insert"); // expect

        db.update("doc-1", &[0.0, 1.0, 0.0, 0.0], Some(json!({"v": 2})))
            .await
            .expect("update"); // expect

        // Metadata should be updated
        let doc = db.get("doc-1").await.expect("get").expect("exists"); // expect
        assert_eq!(doc.metadata.expect("valid")["v"], 2); // expect

        // Vector should be updated — search for new vector should find it
        let results = db.search(&[0.0, 1.0, 0.0, 0.0], 1).await.expect("search"); // expect
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc-1");
    }

    #[tokio::test]
    async fn test_delete() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect
        assert_eq!(db.len().await.expect("len"), 1); // expect

        db.delete("doc-1").await.expect("delete"); // expect
        assert_eq!(db.len().await.expect("len"), 0); // expect

        // get should return None after delete
        let doc = db.get("doc-1").await.expect("get"); // expect
        assert!(doc.is_none());
    }

    #[tokio::test]
    async fn test_relate() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect
        db.insert("doc-2", &[0.0, 1.0, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect

        // Should not error
        db.relate("doc-1", "doc-2", "references")
            .await
            .expect("relate"); // expect
    }

    #[tokio::test]
    async fn test_dimension_mismatch() {
        let (db, _tmp) = test_db(4).await;
        let result = db.insert("doc-1", &[1.0, 0.0], None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_search() {
        let (db, _tmp) = test_db(4).await;
        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 5).await.expect("search"); // expect
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_relate_and_scan_prefix() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect
        db.insert("doc-2", &[0.0, 1.0, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect
        db.insert("doc-3", &[0.0, 0.0, 1.0, 0.0], None)
            .await
            .expect("insert"); // expect

        db.relate_bidirectional("doc-1", "doc-2", "references")
            .await
            .expect("relate"); // expect
        db.relate("doc-1", "doc-3", "references")
            .await
            .expect("relate"); // expect

        // Scan for relations of doc-1
        let results = db
            .scan_prefix("__rel:doc-1:references:", None)
            .await
            .expect("scan"); // expect
        assert_eq!(results.len(), 2);

        let related_ids: Vec<String> = results
            .into_iter()
            .map(|(_, v)| v["to"].as_str().expect("valid").to_string()) // expect
            .collect();
        assert!(related_ids.contains(&"doc-2".to_string()));
        assert!(related_ids.contains(&"doc-3".to_string()));

        // Check backward edge setup automatically
        let backward_results = db
            .scan_prefix("__rel:doc-2:references:", None)
            .await
            .expect("scan bwd"); // expect
        assert_eq!(backward_results.len(), 1);
        assert_eq!(backward_results[0].1["to"], "doc-1");
    }

    #[tokio::test]
    async fn test_stats_aggregation() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect

        let stats = db.stats().await.expect("stats"); // expect
        assert_eq!(stats.index_stats.num_vectors, 1);
        assert_eq!(stats.active_memory_count, 1);
        assert_eq!(stats.drift_status, "nicht verfügbar");
        assert!(stats.storage_stats.memtable_size_bytes > 0);
    }

    #[tokio::test]
    async fn test_stats_live_data_and_weak_reference_lifecycle() {
        let (db, _tmp) = test_db(4).await;

        // 1. Unattached state
        let stats_unattached = db.stats().await.expect("stats");
        assert_eq!(stats_unattached.drift_status, "nicht verfügbar");
        assert_eq!(stats_unattached.calibration_ece, None);
        assert_eq!(stats_unattached.last_calibration_at, None);
        assert_eq!(stats_unattached.pid_pool_size, None);

        // 2. Attached state with live components
        struct DummyRouter;
        impl DriftStatusProvider for DummyRouter {
            fn overall_drift_status(&self) -> String {
                "stabil".to_string()
            }
        }

        let router_arc: Arc<dyn DriftStatusProvider> = Arc::new(DummyRouter);
        let calibrator_arc = Arc::new(parking_lot::Mutex::new(
            memfuse_rank::IsotonicCalibrator::new(5, 100),
        ));
        let pid_arc = Arc::new(parking_lot::Mutex::new(
            memfuse_adapt::PidController::new(150.0, 50, 200, Some(100)),
        ));

        // Warmup calibrator so ECE is populated
        {
            let mut cal = calibrator_arc.lock();
            for i in 0..10 {
                cal.record_outcome(i as f32 / 10.0, i > 5);
            }
            cal.force_rebuild();
        }

        db.set_router(Arc::downgrade(&router_arc));
        db.set_calibrator(Arc::downgrade(&calibrator_arc));
        db.set_pid_controller(Arc::downgrade(&pid_arc));

        let stats_attached = db.stats().await.expect("stats");
        assert_eq!(stats_attached.drift_status, "stabil");
        assert!(stats_attached.calibration_ece.is_some());
        assert!(stats_attached.last_calibration_at.is_some());
        assert_eq!(stats_attached.pid_pool_size, Some(100));

        // 3. Drop strong references (simulating shutdown) -> stats() must not panic and fallback gracefully
        drop(router_arc);
        drop(calibrator_arc);
        drop(pid_arc);

        let stats_dropped = db.stats().await.expect("stats after drop");
        assert_eq!(stats_dropped.drift_status, "nicht verfügbar");
        assert_eq!(stats_dropped.calibration_ece, None);
        assert_eq!(stats_dropped.last_calibration_at, None);
        assert_eq!(stats_dropped.pid_pool_size, None);
    }

    #[tokio::test]
    async fn test_integration_end_to_end() {
        let (db, _tmp) = test_db(4).await;

        // 1. Insert
        db.insert(
            "agent-1",
            &[1.0, 0.5, 0.0, 0.0],
            Some(json!({"type": "agent"})),
        )
        .await
        .expect("insert agent"); // expect
        db.insert(
            "task-1",
            &[0.9, 0.6, 0.0, 0.0],
            Some(json!({"type": "task"})),
        )
        .await
        .expect("insert task"); // expect
        db.insert(
            "task-2",
            &[0.0, 0.0, 1.0, 0.5],
            Some(json!({"type": "task"})),
        )
        .await
        .expect("insert task 2"); // expect

        // 2. Relate
        db.relate("agent-1", "task-1", "assigned_to")
            .await
            .expect("relate 1"); // expect
        db.relate("agent-1", "task-2", "assigned_to")
            .await
            .expect("relate 2"); // expect

        // 3. Search
        let results = db.search(&[1.0, 0.5, 0.0, 0.0], 2).await.expect("search"); // expect
        assert_eq!(results[0].id, "agent-1"); // Exactly matches
        assert_eq!(results[1].id, "task-1"); // Close match

        // 4. Update
        db.update(
            "task-1",
            &[0.1, 0.1, 0.9, 0.9],
            Some(json!({"type": "task", "status": "done"})),
        )
        .await
        .expect("update task"); // expect

        // 5. Scan prefix
        let edges = db
            .scan_prefix("__rel:agent-1:assigned_to:", None)
            .await
            .expect("scan"); // expect
        assert_eq!(edges.len(), 2);

        // 6. Delete
        db.delete("agent-1").await.expect("delete"); // expect

        // 7. Verify empty search and missing doc
        let get_agent = db.get("agent-1").await.expect("get"); // expect
        assert!(get_agent.is_none());
        assert_eq!(db.len().await.expect("len"), 2); // 3 inserted, 1 deleted // expect
    }

    #[tokio::test]
    async fn test_concurrent_collection_idempotency() {
        let (db, _tmp) = test_db(4).await;
        let db = Arc::new(db);
        let mut handles = Vec::new();

        for _ in 0..10 {
            let db_clone = db.clone();
            handles.push(tokio::spawn(async move {
                db_clone.collection("c").await.expect("collection c") // expect
            }));
        }

        let mut cols = Vec::new();
        for handle in handles {
            cols.push(handle.await.expect("join handle")); // expect
        }

        let first = &cols[0];
        for col in &cols[1..] {
            assert!(
                Arc::ptr_eq(first, col),
                "collection(\"c\") must return the exact same Arc instance"
            );
        }
    }

    #[tokio::test]
    async fn collections_are_isolated() {
        let (db, _tmp) = test_db(4).await;
        let vec = vec![1.0, 0.0, 0.0, 0.0];
        let col_a = db.collection("alpha").await.unwrap(); // unwrap
        let col_b = db.collection("beta").await.unwrap(); // unwrap
        col_a.insert("doc1", &vec, None).await.unwrap(); // unwrap
        let results = col_b.search(&vec, 10).await.unwrap(); // unwrap
        assert!(
            results.is_empty(),
            "Collection B must not see Collection A's data"
        );
    }

    #[tokio::test]
    async fn test_collections_are_isolated() {
        let (db, _tmp) = test_db(4).await;
        let col_a = db.collection("a").await.expect("col a"); // expect
        let col_b = db.collection("b").await.expect("col b"); // expect

        col_a
            .insert("k1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"val": "a"})))
            .await
            .expect("ins a"); // expect
        col_b
            .insert("k1", &[0.0, 1.0, 0.0, 0.0], Some(json!({"val": "b"})))
            .await
            .expect("ins b"); // expect

        let res_a = col_a.get("k1").await.expect("get a").expect("exists"); // expect
        let res_b = col_b.get("k1").await.expect("get b").expect("exists"); // expect

        assert_eq!(res_a.metadata.expect("test")["val"], "a"); // expect
        assert_eq!(res_b.metadata.expect("test")["val"], "b"); // expect

        let search_a = col_a
            .search(&[1.0, 0.0, 0.0, 0.0], 1)
            .await
            .expect("search a"); // expect
        assert_eq!(search_a.len(), 1);
        assert_eq!(search_a[0].id, "k1");
        assert_eq!(search_a[0].metadata.as_ref().expect("test")["val"], "a"); // expect
    }

    #[tokio::test]
    async fn test_close_and_reopen_100_docs() {
        let tmp = TempDir::new().expect("temp dir"); // expect
        let path = tmp.path().to_path_buf();
        let config = MemFuseConfig {
            dimension: 4,
            ..Default::default()
        };

        {
            let db = MemFuse::open_with_config(&path, config.clone())
                .await
                .expect("open 1"); // expect
            for i in 0..100 {
                let id = format!("doc-{}", i);
                let val = (i as f32) / 100.0;
                db.insert(&id, &[val, 1.0 - val, 0.0, 0.0], Some(json!({"idx": i})))
                    .await
                    .expect("insert"); // expect
            }
            db.close().await.expect("close"); // expect
        }

        {
            let db = MemFuse::open_with_config(&path, config)
                .await
                .expect("open 2"); // expect
            assert_eq!(db.len().await.expect("len"), 100); // expect
            for i in 0..100 {
                let id = format!("doc-{}", i);
                let doc = db.get(&id).await.expect("get").expect("exists"); // expect
                assert_eq!(doc.id, id);
                assert_eq!(doc.metadata.expect("valid")["idx"], i); // expect
            }
            let results = db.search(&[0.5, 0.5, 0.0, 0.0], 10).await.expect("search"); // expect
            assert_eq!(results.len(), 10);
        }
    }

    #[tokio::test]
    async fn test_close_and_reopen() {
        let tmp = TempDir::new().expect("temp dir"); // expect
        let path = tmp.path().to_path_buf();
        let config = MemFuseConfig {
            dimension: 4,
            ..Default::default()
        };

        {
            let db = MemFuse::open_with_config(&path, config.clone())
                .await
                .expect("open 1"); // expect
            db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"v": 1})))
                .await
                .expect("insert"); // expect
            db.close().await.expect("close"); // expect
        }

        {
            let db = MemFuse::open_with_config(&path, config)
                .await
                .expect("open 2"); // expect
            let doc = db.get("doc-1").await.expect("get").expect("exists"); // expect
            assert_eq!(doc.id, "doc-1");
            assert_eq!(doc.metadata.expect("valid")["v"], 1); // expect
        }
    }

    #[tokio::test]
    async fn test_drop_removes_all_data() {
        let (db, _tmp) = test_db(4).await;
        let col = db.collection("drop-me").await.expect("col"); // expect
        col.insert("k1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("ins"); // expect

        let tenant_id = TenantId::try_new(1).expect("tenant_id"); // expect
        db.drop_collection("drop-me", tenant_id, &[0u8; 32])
            .await
            .expect("drop"); // expect

        let col2 = db.collection("drop-me").await.expect("re-create"); // expect
        assert_eq!(col2.len().await, 0);
        assert!(col2.get("k1").await.expect("get").is_none()); // expect
    }

    #[tokio::test]
    async fn test_default_collection_compat() {
        let (db, _tmp) = test_db(4).await;
        db.insert("k", &[1.0, 0.0, 0.0, 0.0], Some(json!({"v": 1})))
            .await
            .expect("ins"); // expect

        let doc = db.get("k").await.expect("get").expect("exists"); // expect
        assert_eq!(doc.id, "k");

        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 1).await.expect("search"); // expect
        assert_eq!(results[0].id, "k");
    }

    #[tokio::test]
    async fn test_list_collections() {
        let (db, _tmp) = test_db(4).await;
        db.collection("c1").await.expect("c1"); // expect
        db.collection("c2").await.expect("c2"); // expect
        db.collection("c3").await.expect("c3"); // expect

        let list = db.list_collections().await.expect("list"); // expect
        assert!(list.contains(&"default".to_string()));
        assert!(list.contains(&"c1".to_string()));
        assert!(list.contains(&"c2".to_string()));
        assert!(list.contains(&"c3".to_string()));
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn test_memfuse_config_defaults_to_onnx_backend() {
        let config = MemFuseConfig::default();
        assert_eq!(config.dimension, 768);
        assert_eq!(
            config.embedding_backend,
            EmbeddingBackend::Onnx {
                model_name: "nomic-embed-text".to_string(),
                cache_dir: None,
            }
        );
        assert_eq!(
            EmbeddingBackend::default(),
            EmbeddingBackend::Onnx {
                model_name: "nomic-embed-text".to_string(),
                cache_dir: None,
            }
        );
    }

    #[tokio::test]
    #[cfg(feature = "onnx")]
    async fn test_onnx_default_backend_e2e_with_fixture() {
        let tmp = TempDir::new().expect("temp dir");
        let cache_tmp = TempDir::new().expect("cache temp dir");
        let model_dir = cache_tmp.path().join("nomic-embed-text");
        std::fs::create_dir_all(&model_dir).expect("create model dir");

        let fixture_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("memfuse-embed")
            .join("tests")
            .join("fixtures");

        std::fs::copy(fixture_dir.join("model.onnx"), model_dir.join("model.onnx"))
            .expect("copy model");
        std::fs::copy(
            fixture_dir.join("tokenizer.json"),
            model_dir.join("tokenizer.json"),
        )
        .expect("copy tokenizer");

        let config = MemFuseConfig {
            dimension: 32, // Fixture BERT model output dim is 32
            embedding_backend: EmbeddingBackend::Onnx {
                model_name: "nomic-embed-text".to_string(),
                cache_dir: Some(cache_tmp.path().to_path_buf()),
            },
            ..Default::default()
        };

        let db = MemFuse::open_with_config(tmp.path(), config)
            .await
            .expect("open_with_config");

        assert!(db.embedder.read().is_some());
        let embedder = db.embedder.read().as_ref().cloned().expect("embedder");
        let vec = embedder.embed("test document").await.expect("embed test");
        assert_eq!(vec.len(), 32);
    }

    #[tokio::test]
    async fn test_open_dimension_mismatch_fails() -> memfuse_core::Result<()> {
        let dir = tempfile::tempdir()
            .map_err(|e| memfuse_core::MemFuseError::InvalidInput(e.to_string()))?;
        let config_768 = MemFuseConfig {
            dimension: 768,
            ..Default::default()
        };
        let _db = MemFuse::open_with_config(dir.path(), config_768).await?;

        // Zweites Öffnen mit falscher Dimension muss früh fehlschlagen
        let config_1536 = MemFuseConfig {
            dimension: 1536,
            ..Default::default()
        };
        let result = MemFuse::open_with_config(dir.path(), config_1536).await;
        assert!(result.is_err());
        let err_msg = match result {
            Err(e) => e.to_string(),
            Ok(_) => panic!("expected error"),
        };
        assert!(err_msg.contains("Dimension mismatch"));
        Ok(())
    }

    #[test]
    fn test_provenance_record_serialization_roundtrip() {
        let p = ProvenanceRecord {
            vector_distance: Some(0.42),
            bm25_score: Some(1.23),
            graph_score: None,
            rerank_score: None,
            signal_ranks: {
                let mut m = ahash::AHashMap::new();
                m.insert("vector".to_string(), 1u32);
                m.insert("bm25".to_string(), 3u32);
                m
            },
            source_collection: Some("test_col".to_string()),
            index_type: Some("hnsw".to_string()),
            signal_contributions: ahash::AHashMap::new(),
            coherence_bonus: 0.0,
        };
        let json = serde_json::to_string(&p).expect("serialize");
        let back: ProvenanceRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.vector_distance, Some(0.42));
        assert_eq!(back.bm25_score, Some(1.23));
        assert_eq!(back.graph_score, None);
        assert_eq!(back.source_collection.as_deref(), Some("test_col"));
    }

    #[test]
    fn test_search_result_with_provenance_serialization() {
        let sr = SearchResult {
            id: "doc1".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["vector".to_string()],
            provenance: Some(ProvenanceRecord {
                source_collection: Some("my_col".to_string()),
                ..Default::default()
            }),
        };
        let json = serde_json::to_string(&sr).expect("serialize");
        assert!(json.contains("provenance"));
        assert!(json.contains("my_col"));
    }

    #[test]
    fn test_search_result_without_provenance_serialization_omits_field() {
        let sr = SearchResult {
            id: "doc2".to_string(),
            score: 0.5,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        };
        let json = serde_json::to_string(&sr).expect("serialize");
        // provenance: None → Feld soll komplett fehlen (skip_serializing_if)
        assert!(
            !json.contains("provenance"),
            "provenance=None soll im JSON weggelassen werden: {json}"
        );
    }

    #[tokio::test]
    async fn test_provenance_rrf_sum_invariant() {
        let (db, _tmp) = test_db(4).await;
        let col = db.collection("prov_test").await.expect("collection");

        for i in 0..10 {
            let id = format!("doc-{}", i);
            let val = (i as f32) / 10.0;
            col.insert(
                &id,
                &[val, 1.0 - val, 0.0, 0.0],
                Some(json!({ "text": format!("rust memory system {}", i) })),
            )
            .await
            .expect("insert");
        }

        let results = col
            .query()
            .text("rust memory")
            .embedding([0.5, 0.5, 0.0, 0.0])
            .include_provenance(true)
            .k(5)
            .execute()
            .await
            .expect("search");

        assert!(!results.is_empty(), "Results must not be empty");

        for result in &results {
            let prov = result
                .provenance
                .as_ref()
                .expect("Provenance must be present when include_provenance=true");
            let sum_contrib: f32 = prov
                .signal_contributions
                .values()
                .map(|c| c.rrf_contribution)
                .sum();
            assert!(
                (sum_contrib - result.score).abs() < 1e-6,
                "INV-PROV-1 verletzt: Summe der Signal-Beiträge ({}) ≠ RRF-Score ({})",
                sum_contrib,
                result.score
            );
        }
    }

    #[tokio::test]
    async fn test_search_results_have_provenance_is_some() {
        let (db, _tmp) = test_db(4).await;
        let col = db.collection("search_prov_test").await.expect("collection");

        col.insert(
            "doc-1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "rust search provenance test"})),
        )
        .await
        .expect("insert");

        let vec_results = col
            .search(&[1.0, 0.0, 0.0, 0.0], 1)
            .await
            .expect("vector search");
        assert_eq!(vec_results.len(), 1);
        assert!(
            vec_results[0].provenance.is_some(),
            "Vector search results must contain provenance record"
        );
        let prov = vec_results[0].provenance.as_ref().expect("provenance");
        assert_eq!(prov.source_collection.as_deref(), Some("search_prov_test"));
        assert_eq!(prov.index_type.as_deref(), Some("hnsw"));

        let text_results = col
            .query()
            .text("rust search")
            .embedding([1.0, 0.0, 0.0, 0.0])
            .include_provenance(true)
            .k(1)
            .execute()
            .await
            .expect("text search");
        assert_eq!(text_results.len(), 1);
        assert!(
            text_results[0].provenance.is_some(),
            "Text search results must contain provenance record"
        );
    }
}
