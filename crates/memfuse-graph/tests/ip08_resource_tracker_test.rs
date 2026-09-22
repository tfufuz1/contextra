use std::sync::Arc;
use memfuse_core::{Entity, EntityId, ResourceBudget, ResourceTracker};
use memfuse_graph::csr::{CsrGraph, CsrGraphConfig};

#[tokio::test]
async fn test_compact_async_resource_tracker_budget_exhaustion() {
    // 1. Create a ResourceTracker with 100 bytes memory limit
    let budget = ResourceBudget {
        memory_limit: 100,
        ..Default::default()
    };
    let tracker = Arc::new(ResourceTracker::new(budget));

    // Consume 95 bytes to reach 95% capacity limit
    tracker.consume_memory(95).expect("consume 95 bytes");
    assert!(!tracker.has_memory_capacity());

    // 2. Configure CsrGraph with the exhausted tracker
    let config = CsrGraphConfig {
        rebuild_threshold: 100, // keep threshold high so add_edge does not trigger auto-compaction during insertion
        resource_tracker: Some(tracker.clone()),
        ..Default::default()
    };
    let graph = Arc::new(CsrGraph::with_config(config));

    // 3. Add entities and edge to dirty the graph delta buffer
    let e1 = Entity::new(EntityId::from("entity_1"), "Entity 1", "test");
    let e2 = Entity::new(EntityId::from("entity_2"), "Entity 2", "test");
    graph.insert_entity_direct(e1).unwrap();
    graph.insert_entity_direct(e2).unwrap();

    graph
        .insert_edge_direct(EntityId::from("entity_1"), EntityId::from("entity_2"), 1.0)
        .await
        .unwrap();

    // Verify edge count reflects pending edge before compact
    assert_eq!(graph.edge_count(), 1);

    // 4. Call compact_async - should be deferred because ResourceTracker is exhausted
    graph.compact_async().await.unwrap();

    // 5. Release memory in ResourceTracker so it has capacity again
    tracker.release_memory(50);
    assert!(tracker.has_memory_capacity());

    // 6. Call compact_async - should now succeed and compact
    graph.compact_async().await.unwrap();

    assert_eq!(graph.edge_count(), 1);
}
