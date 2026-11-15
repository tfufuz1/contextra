//! Conformance tests for  port implementations.
use acta_graph::storage::{conformance::run_all, InMemoryGraphStorage};

#[tokio::test]
async fn test_in_memory_graph_storage_conformance() {
    let storage = InMemoryGraphStorage::new();
    run_all(storage).await;
}

#[cfg(feature = "workspace")]
#[tokio::test]
async fn test_engine_handle_conformance() {
    use acta_graph::compat::storage_bridge::EngineHandle;
    use acta_graph::test_doubles::InMemoryStorageEngine;
    use std::sync::Arc;

    let engine = Arc::new(InMemoryStorageEngine::new());
    let handle = EngineHandle(engine);
    run_all(handle).await;
}
