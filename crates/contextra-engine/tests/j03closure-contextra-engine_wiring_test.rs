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
async fn test_j03_closure_kv_cache_and_exporter_wiring() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let _worker = start_kv_cache_eviction(store);

    let exporter = AccessCounterAttentionExporter::new();
    let req_id = RequestId(101);

    // Initial access count should be 0
    assert_eq!(exporter.get_access_count(req_id), 0);

    // Calling export_attention_weights triggers fallback record_access
    let weights = exporter.export_attention_weights(req_id);
    assert_eq!(weights, Some(vec![1.0]));
    assert_eq!(exporter.get_access_count(req_id), 1);

    // Explicit record_access increments frequency
    exporter.record_access(req_id);
    assert_eq!(exporter.get_access_count(req_id), 2);
    assert_eq!(exporter.export_attention_weights(req_id), Some(vec![2.0]));
}

#[tokio::test]
async fn test_j03_closure_auto_extraction_config_and_regulated_mode() {
    let mode_reg = recommended_mode_for_regulated(true);
    assert_eq!(mode_reg, AutoExtractionMode::Disabled);

    let mode_unreg = recommended_mode_for_regulated(false);
    assert_eq!(mode_unreg, AutoExtractionMode::Enabled);

    let cfg = AutoExtractionConfig::for_regulated(false).with_mode(AutoExtractionMode::Enabled);
    assert_eq!(cfg.mode, AutoExtractionMode::Enabled);
    assert!(cfg.is_enabled());

    let temp_dir = tempfile::tempdir().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(temp_dir.path(), config)
        .await
        .unwrap();

    let col = db.collection("ext_closure_col").await.unwrap();

    col.set_auto_extraction_config(cfg.clone());
    let (_, retrieved) = col.auto_extraction_config().expect("Config must exist");
    assert_eq!(retrieved.mode, AutoExtractionMode::Enabled);

    struct TestGen;
    impl contextra_ports::LlmTextGenerator for TestGen {
        fn generate<'a>(
            &'a self,
            _prompt: &'a str,
        ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<String>> {
            Box::pin(async move { Ok("[]".to_string()) })
        }
    }

    let col_val = Arc::unwrap_or_clone(col);
    let col_with = col_val.with_auto_extraction(Arc::new(TestGen), AutoExtractionConfig::default());
    assert!(col_with.auto_extraction_config().is_some());
}

#[tokio::test]
async fn test_j03_closure_scope_constraint_and_builder_wiring() {
    let doc1 = DocId::from_key("doc1").unwrap();
    let doc2 = DocId::from_key("doc2").unwrap();
    let mut allowed = BTreeSet::new();
    allowed.insert(doc1);
    allowed.insert(doc2);

    let scope = ScopeConstraint::from_allowed_ids(allowed.clone());
    assert_eq!(scope.allowed_doc_ids, allowed);
    assert_eq!(scope.gamma, 2);

    let default_scope = ScopeConstraint::default();
    assert!(default_scope.allowed_doc_ids.is_empty());
    assert_eq!(default_scope.gamma, 2);

    let temp_dir = tempfile::tempdir().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(temp_dir.path(), config)
        .await
        .unwrap();

    let col = db.collection("scope_closure_col").await.unwrap();
    let qb = col.query().scope_ids(allowed);
    let _ = qb;
}

#[tokio::test]
async fn test_j03_closure_tenant_collections_and_document_importance() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(temp_dir.path(), config)
        .await
        .unwrap();

    let tenant = TenantId::try_new(505).unwrap();
    let col = db
        .collection_for_tenant("tenant_closure_col", tenant)
        .await
        .unwrap();

    let tenant_cols = db.list_collections_for_tenant(tenant).await.unwrap();
    assert!(tenant_cols.contains(&"tenant_closure_col".to_string()));

    col.insert("doc_imp_1", &[0.1, 0.2, 0.3, 0.4], None)
        .await
        .unwrap();

    col.update_document_importance("doc_imp_1", 0.88, "model_closure_v1")
        .await
        .unwrap();

    let stored = col.get("doc_imp_1").await.unwrap().unwrap();
    let meta = stored.metadata.expect("Metadata must be present");
    assert_eq!(meta.get("model_id").unwrap(), "model_closure_v1");
    assert!(meta.get("importance").is_some());

    db.purge_tenant(tenant).await.unwrap();
}
