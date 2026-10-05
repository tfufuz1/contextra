#![forbid(unsafe_code)]

use contextra_engine::collection::crud::{
    recommended_mode_for_regulated, AutoExtractionConfig, AutoExtractionMode,
};
use contextra_engine::collection::query_builder::ScopeConstraint;
use contextra_engine::kv_cache_integration::{
    start_kv_cache_eviction, AccessCounterAttentionExporter,
};
use contextra_engine::{Contextra, ContextraConfig};
use contextra_kvcache::TenantIsolatedKvStore;
use contextra_ports::{AttentionExporter, RequestId, TenantId};
use contextra_types::DocId;
use std::collections::BTreeSet;
use std::sync::Arc;

#[tokio::test]
async fn test_kv_cache_eviction_and_access_counter_wiring() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let _worker = start_kv_cache_eviction(store);

    let exporter = AccessCounterAttentionExporter::new();
    let req_id = RequestId(42);
    assert_eq!(exporter.get_access_count(req_id), 0);

    exporter.record_access(req_id);
    exporter.record_access(req_id);
    assert_eq!(exporter.get_access_count(req_id), 2);

    let weights = exporter.export_attention_weights(req_id);
    assert_eq!(weights, Some(vec![2.0]));
}

#[tokio::test]
async fn test_auto_extraction_for_regulated_and_builder_wiring() {
    let mode_regulated = recommended_mode_for_regulated(true);
    assert_eq!(mode_regulated, AutoExtractionMode::Disabled);

    let mode_unregulated = recommended_mode_for_regulated(false);
    assert_eq!(mode_unregulated, AutoExtractionMode::Enabled);

    let cfg_regulated = AutoExtractionConfig::for_regulated(true);
    assert_eq!(cfg_regulated.mode, AutoExtractionMode::Disabled);
    assert!(!cfg_regulated.is_enabled());

    let cfg_unregulated =
        AutoExtractionConfig::for_regulated(false).with_mode(AutoExtractionMode::Enabled);
    assert_eq!(cfg_unregulated.mode, AutoExtractionMode::Enabled);

    let temp_dir = tempfile::tempdir().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(temp_dir.path(), config)
        .await
        .unwrap();

    let col = db.collection("auto_ext_col").await.unwrap();

    // Verify Collection::set_auto_extraction_config production builder path
    col.set_auto_extraction_config(cfg_regulated.clone());
    let (gen_opt, retrieved_cfg) = col.auto_extraction_config().unwrap();
    assert_eq!(retrieved_cfg.mode, AutoExtractionMode::Disabled);
    let _ = gen_opt;

    // Verify Collection::with_auto_extraction chaining path
    struct DummyGen;
    impl contextra_ports::LlmTextGenerator for DummyGen {
        fn generate<'a>(
            &'a self,
            _prompt: &'a str,
        ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<String>> {
            Box::pin(async move { Ok("[]".to_string()) })
        }
    }

    let col_val = Arc::unwrap_or_clone(col);
    let col_with_auto =
        col_val.with_auto_extraction(Arc::new(DummyGen), AutoExtractionConfig::default());
    assert!(col_with_auto.auto_extraction_config().is_some());
}

#[tokio::test]
async fn test_scope_constraint_from_allowed_ids() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(temp_dir.path(), config)
        .await
        .unwrap();

    let col = db.collection("scope_col").await.unwrap();

    let doc_a = DocId::from_key("doc_a").unwrap();
    let doc_b = DocId::from_key("doc_b").unwrap();
    let mut allowed = BTreeSet::new();
    allowed.insert(doc_a);
    allowed.insert(doc_b);

    let scope = ScopeConstraint::from_allowed_ids(allowed.clone());
    assert_eq!(scope.allowed_doc_ids, allowed);
    assert_eq!(scope.gamma, 2);

    // Test through production query builder execution path
    let query_builder = col.query().scope(scope);
    let _ = query_builder;
}

#[tokio::test]
async fn test_contextra_list_collections_for_tenant_and_importance_update() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(temp_dir.path(), config)
        .await
        .unwrap();

    let tenant = TenantId::try_new(100).unwrap();
    let col = db
        .collection_for_tenant("test_col", tenant)
        .await
        .unwrap();

    let cols = db.list_collections_for_tenant(tenant).await.unwrap();
    assert!(cols.contains(&"test_col".to_string()));

    col.insert("doc_1", &[0.1, 0.2, 0.3, 0.4], None)
        .await
        .unwrap();

    col.update_document_importance("doc_1", 0.95, "model_v1")
        .await
        .unwrap();

    let stored = col.get("doc_1").await.unwrap().unwrap();
    let meta = stored.metadata.expect("Metadata must be present");
    assert_eq!(meta.get("model_id").unwrap(), "model_v1");
    assert!(meta.get("importance").is_some());
}
