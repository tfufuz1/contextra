use super::super::*;
use super::basic_tests::setup_test_graph;
use crate::csr::types::{CsrGraphConfig, PersistedEdgePayload, MAX_VISITED_NODES};
use crate::GraphIndexExt;
use contextra_ports::{GraphIndex, StorageEngine};
use contextra_types::{ContextraError, DocId, Edge, Entity, EntityId, TxId};
use std::sync::Arc;

#[tokio::test]
#[allow(non_snake_case)]
async fn set_storage_and_with_config_and_storage_CASE_initialization() {
    use contextra_store::{LsmConfig, LsmStorage};

    let dir = tempfile::tempdir().unwrap(); // unwrap allowed
    let storage: Arc<dyn StorageEngine> = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap allowed
    );

    let config = CsrGraphConfig {
        rebuild_threshold: 50,
        ..Default::default()
    };
    let mut graph = CsrGraph::with_config_and_storage(config, storage.clone());
    assert!(graph.storage.is_some());

    // Replace storage via set_storage
    let dir2 = tempfile::tempdir().unwrap(); // unwrap allowed
    let storage2: Arc<dyn StorageEngine> = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir2.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap allowed
    );

    graph.set_storage(storage2);
    assert!(graph.storage.is_some());
}

#[tokio::test]
#[allow(non_snake_case)]
async fn insert_entity_direct_and_edge_direct_CASE_and_boundaries() {
    let graph = Arc::new(CsrGraph::new());

    let id1 = EntityId::new(10);
    let id2 = EntityId::new(20);

    graph
        .insert_entity_direct(Entity::new(id1, "Direct1", "Type"))
        .unwrap(); // unwrap allowed
    graph
        .insert_entity_direct(Entity::new(id2, "Direct2", "Type"))
        .unwrap(); // unwrap allowed

    assert_eq!(graph.entity_count(), 2);
    assert!(graph.entity_exists(id1));
    assert!(graph.entity_exists(id2));
    assert!(!graph.entity_exists(EntityId::new(999)));

    graph
        .insert_edge_direct_with_validity(id1, id2, 1.5, Some(TxId::new(5)), Some(TxId::new(50)))
        .await
        .unwrap(); // unwrap allowed

    assert_eq!(graph.edge_count(), 1);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn persist_entity_edge_delete_persistence_CASE_direct_calls() {
    use contextra_store::{LsmConfig, LsmStorage};

    let dir = tempfile::tempdir().unwrap(); // unwrap allowed
    let storage = LsmStorage::new(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    })
    .await
    .unwrap(); // unwrap allowed

    let graph = CsrGraph::new();
    let tx = TxId::new(10);
    let id1 = EntityId::new(1);
    let id2 = EntityId::new(2);
    let entity = Entity::new(id1, "P1", "Person");
    let payload = PersistedEdgePayload {
        weight: 0.9,
        tx_valid_from: Some(TxId::new(1)),
        tx_valid_to: None,
        business_valid_from: None,
        business_valid_to: None,
        source_doc_id: None,
    };

    // Persist entity & edge directly
    graph.persist_entity(&storage, tx, &entity).await.unwrap(); // unwrap allowed
    graph
        .persist_edge(&storage, tx, &id1, &id2, &payload)
        .await
        .unwrap(); // unwrap allowed
    storage.commit(tx).await.unwrap(); // unwrap allowed

    // Delete edge persistence
    let tx2 = TxId::new(11);
    graph
        .delete_edge_persistence(&storage, tx2, &id1, &id2)
        .await
        .unwrap(); // unwrap allowed
    storage.commit(tx2).await.unwrap(); // unwrap allowed
}

#[tokio::test]
#[allow(non_snake_case)]
async fn load_from_storage_CASE_legacy_f32_weight_and_invalid_key() {
    use contextra_store::{LsmConfig, LsmStorage};

    let dir = tempfile::tempdir().unwrap(); // unwrap allowed
    let storage = LsmStorage::new(LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    })
    .await
    .unwrap(); // unwrap allowed

    let tx = TxId::new(1);

    // Put legacy f32 edge payload
    let legacy_key = b"__graph:edge:1:2";
    let legacy_val = bincode::serialize(&0.75f32).unwrap(); // unwrap allowed
    storage.put(tx, legacy_key, &legacy_val).await.unwrap(); // unwrap allowed

    // Put invalid key (missing colon delimiter)
    let invalid_key = b"__graph:edge:invalidkeywithoutcolon";
    let payload = PersistedEdgePayload {
        weight: 1.0,
        tx_valid_from: None,
        tx_valid_to: None,
        business_valid_from: None,
        business_valid_to: None,
        source_doc_id: None,
    };
    let payload_val = bincode::serialize(&payload).unwrap(); // unwrap allowed
    storage.put(tx, invalid_key, &payload_val).await.unwrap(); // unwrap allowed

    storage.commit(tx).await.unwrap(); // unwrap allowed

    let loaded_graph = CsrGraph::load_from_storage(&storage).await.unwrap(); // unwrap allowed
    assert_eq!(loaded_graph.edge_count(), 1);
}

#[derive(Clone)]
struct LogCaptureLayer(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for LogCaptureLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = StringVisitor(String::new());
        event.record(&mut visitor);
        self.0.lock().unwrap().push(visitor.0); // unwrap
    }
}

struct StringVisitor(String);
impl tracing::field::Visit for StringVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write;
        write!(self.0, "{}={:?} ", field.name(), value).ok();
    }
}

#[tokio::test]
async fn test_traverse_max_hops_exceeded_emits_warning() {
    use tracing_subscriber::layer::SubscriberExt;

    let logs = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let capture_layer = LogCaptureLayer(logs.clone());
    let subscriber = tracing_subscriber::registry().with(capture_layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let graph = setup_test_graph().await;

    let _ = graph.traverse(EntityId::new(1), 5).await.unwrap(); // unwrap

    let captured = logs.lock().unwrap(); // unwrap
    let warning_found = captured.iter().any(|msg| {
        msg.contains("exceeds internal cap MAX_TRAVERSAL_HOPS")
            && msg.contains("requested_max_hops=5")
    });

    assert!(
        warning_found,
        "Expected warning log when max_hops > MAX_TRAVERSAL_HOPS, got: {:?}",
        *captured
    );
}

#[tokio::test]
#[allow(non_snake_case)]
async fn traverse_at_time_CASE_saturating_max_hops() {
    let graph = setup_test_graph().await;

    // Traverse with max_hops 100 (should saturate safely to MAX_TRAVERSAL_HOPS=3 without panic/OOM)
    let results = graph
        .traverse_at_time(EntityId::new(1), 100, TxId::new(100))
        .await
        .unwrap(); // unwrap allowed

    assert!(!results.is_empty());
    assert!(results.len() <= 4);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn traverse_CASE_exceeds_max_hops_returns_invalid_input() {
    let graph = setup_test_graph().await;

    let err = graph.traverse(EntityId::new(1), 101).await.unwrap_err();
    assert!(matches!(err, ContextraError::InvalidInput(_)));

    let err_time = graph
        .traverse_at_time(EntityId::new(1), 101, TxId::new(100))
        .await
        .unwrap_err();
    assert!(matches!(err_time, ContextraError::InvalidInput(_)));
}

#[tokio::test]
#[allow(non_snake_case)]
async fn pagerank_CASE_empty_graph_and_isolated_node() {
    let empty_graph = CsrGraph::new();
    let ranks_empty = empty_graph.pagerank(0.85, 100, 1e-6).await;
    assert!(ranks_empty.is_empty());

    let iso_graph = CsrGraph::new();
    iso_graph
        .insert_entity_direct(Entity::new(EntityId::new(1), "Iso", "Type"))
        .unwrap(); // unwrap allowed

    let ranks_iso = iso_graph.pagerank(0.85, 100, 1e-6).await;
    assert_eq!(ranks_iso.len(), 1);
    let rank = ranks_iso[&EntityId::new(1)];
    assert!((rank - 1.0).abs() < 1e-4);
}

#[test]
#[allow(non_snake_case)]
fn serialization_roundtrip_CASE_persisted_edge_payload() {
    let payload = PersistedEdgePayload {
        weight: 0.825,
        tx_valid_from: Some(TxId::new(10)),
        tx_valid_to: Some(TxId::new(20)),
        business_valid_from: Some(1000),
        business_valid_to: Some(2000),
        source_doc_id: None,
    };

    let serialized = bincode::serialize(&payload).unwrap(); // unwrap allowed
    let deserialized: PersistedEdgePayload = bincode::deserialize(&serialized).unwrap(); // unwrap allowed

    assert!((payload.weight - deserialized.weight).abs() < f32::EPSILON);
    assert_eq!(payload.tx_valid_from, deserialized.tx_valid_from);
    assert_eq!(payload.tx_valid_to, deserialized.tx_valid_to);
    assert_eq!(
        payload.business_valid_from,
        deserialized.business_valid_from
    );
    assert_eq!(payload.business_valid_to, deserialized.business_valid_to);
}

proptest::proptest! {
    #[test]
    fn prop_csr_offset_array_structural_consistency(
        node_count in 1..=30usize,
        edge_pairs in proptest::collection::vec((0..30usize, 0..30usize, 0.1f32..2.0f32), 1..100)
    ) {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let res: std::result::Result<(), proptest::test_runner::TestCaseError> = rt.block_on(async {
            let graph = Arc::new(CsrGraph::new());
            for i in 0..node_count {
                graph.insert_entity_direct(Entity::new(EntityId::new(i as u64 + 1), format!("N{i}"), "Type")).unwrap(); // unwrap
            }

            for (src, dst, w) in edge_pairs {
                let src_id = EntityId::new((src % node_count) as u64 + 1);
                let dst_id = EntityId::new((dst % node_count) as u64 + 1);
                graph.insert_edge_direct(src_id, dst_id, w).await.unwrap(); // unwrap
            }

            graph.compact();

            let inner = graph.inner_read();

            // Invariant 1: offsets length must equal reverse_map length + 1 after compaction
            proptest::prop_assert_eq!(inner.offsets.len(), inner.reverse_map.len() + 1);

            // Invariant 2: offsets must be monotonically non-decreasing
            for window in inner.offsets.windows(2) {
                proptest::prop_assert!(window[0] <= window[1]);
            }

            // Invariant 3: final offset must match targets length
            proptest::prop_assert_eq!(*inner.offsets.last().unwrap(), inner.targets.len()); // unwrap

            // Invariant 4: parallel arrays (targets, weights, tx_valid_froms, tx_valid_tos, business_valid_froms, business_valid_tos, source_doc_ids) must have equal lengths
            proptest::prop_assert_eq!(inner.targets.len(), inner.weights.len());
            proptest::prop_assert_eq!(inner.targets.len(), inner.tx_valid_froms.len());
            proptest::prop_assert_eq!(inner.targets.len(), inner.tx_valid_tos.len());
            proptest::prop_assert_eq!(inner.targets.len(), inner.business_valid_froms.len());
            proptest::prop_assert_eq!(inner.targets.len(), inner.business_valid_tos.len());
            proptest::prop_assert_eq!(inner.targets.len(), inner.source_doc_ids.len());
            Ok(())
        });
        res?;
    }
}

#[tokio::test]
async fn test_bitemporal_independent_axis_evaluation() {
    let graph = CsrGraph::new();
    let tx1 = TxId::new(1);
    let id1 = EntityId::new(1);
    let id2 = EntityId::new(2);

    graph
        .add_entity(tx1, Entity::new(id1, "ContractNode1", "Company"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx1, Entity::new(id2, "ContractNode2", "Vendor"))
        .await
        .unwrap(); // unwrap

    let bitemporal_edge = Edge::new(id1, id2, "contract_valid")
        .with_tx_validity(Some(TxId::new(10)), Some(TxId::new(100)))
        .with_business_validity(Some(1000), Some(2000));

    graph.add_edge(tx1, bitemporal_edge).await.unwrap(); // unwrap
    graph.commit(tx1).await.unwrap(); // unwrap

    async fn verify_bitemporal_assertions(g: &CsrGraph, id1: EntityId, id2: EntityId, label: &str) {
        // Case 1: System valid (tx=50), Business invalid (500 < 1000) -> NOT visible
        let res1 = g
            .traverse_at_bitemporal(id1, 1, TxId::new(50), Some(500))
            .await
            .unwrap(); // unwrap
        assert!(
            res1.is_empty(),
            "[{label}] Edge must NOT be visible before business validity start (500 < 1000)"
        );

        // Case 2: System valid (tx=50), Business valid (1000 <= 1500 < 2000) -> VISIBLE
        let res2 = g
            .traverse_at_bitemporal(id1, 1, TxId::new(50), Some(1500))
            .await
            .unwrap(); // unwrap
        assert_eq!(
            res2.len(),
            1,
            "[{label}] Edge MUST be visible when both system and business axes match"
        );
        assert_eq!(res2[0].0, id2);

        // Case 3: System valid (tx=50), Business expired (2500 >= 2000) -> NOT visible
        let res3 = g
            .traverse_at_bitemporal(id1, 1, TxId::new(50), Some(2500))
            .await
            .unwrap(); // unwrap
        assert!(
            res3.is_empty(),
            "[{label}] Edge must NOT be visible after business validity end (2500 >= 2000)"
        );

        // Case 4: Business valid (1500), System before valid (5 < 10) -> NOT visible
        let res4 = g
            .traverse_at_bitemporal(id1, 1, TxId::new(5), Some(1500))
            .await
            .unwrap(); // unwrap
        assert!(
            res4.is_empty(),
            "[{label}] Edge must NOT be visible before system validity start (5 < 10)"
        );

        // Case 5: Business valid (1500), System expired (150 >= 100) -> NOT visible
        let res5 = g
            .traverse_at_bitemporal(id1, 1, TxId::new(150), Some(1500))
            .await
            .unwrap(); // unwrap
        assert!(
            res5.is_empty(),
            "[{label}] Edge must NOT be visible after system validity end (150 >= 100)"
        );

        // Case 6: System valid (tx=50), Business filter omitted (None) -> VISIBLE
        let res6 = g
            .traverse_at_bitemporal(id1, 1, TxId::new(50), None)
            .await
            .unwrap(); // unwrap
        assert_eq!(
            res6.len(),
            1,
            "[{label}] Edge MUST be visible when business filter is None"
        );
    }

    // 1. Verify uncompacted delta buffer path
    verify_bitemporal_assertions(&graph, id1, id2, "delta_buffer").await;

    // 2. Compact graph and verify CSR array path
    graph.compact();
    verify_bitemporal_assertions(&graph, id1, id2, "compacted_csr").await;
}

#[tokio::test]
async fn test_bitemporal_regression_pure_tx_time_unchanged() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let id1 = EntityId::new(1);
    let id2 = EntityId::new(2);

    graph
        .add_entity(tx, Entity::new(id1, "N1", "T"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(id2, "N2", "T"))
        .await
        .unwrap(); // unwrap

    // Pure transaction-time edge (no business time set)
    let pure_tx_edge = Edge::new(id1, id2, "pure_tx_rel")
        .with_tx_validity(Some(TxId::new(10)), Some(TxId::new(100)));

    graph.add_edge(tx, pure_tx_edge).await.unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    // Before tx_valid_from (9 < 10) -> NOT visible regardless of business_as_of
    assert!(graph
        .traverse_at_bitemporal(id1, 1, TxId::new(9), Some(999999))
        .await
        .unwrap()
        .is_empty());

    // At tx_valid_from (10) -> VISIBLE regardless of business_as_of
    let res_tx = graph
        .traverse_at_bitemporal(id1, 1, TxId::new(10), Some(999999))
        .await
        .unwrap();
    assert_eq!(res_tx.len(), 1);

    // Standard traverse_at_time call
    let res_time = graph.traverse_at_time(id1, 1, TxId::new(50)).await.unwrap();
    assert_eq!(res_time.len(), 1);

    // Compact and re-verify
    graph.compact();
    assert_eq!(
        graph
            .traverse_at_bitemporal(id1, 1, TxId::new(50), Some(12345))
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn test_hub_node_1m_neighbors_bfs_capped() -> contextra_types::Result<()> {
    // Use a high rebuild_threshold to avoid repeated O(N) CSR compactions during setup
    let graph = Arc::new(CsrGraph::with_config(CsrGraphConfig {
        rebuild_threshold: 2_000_000,
        ..Default::default()
    }));
    let start = EntityId::new(1);
    let hub = EntityId::new(2);

    // Build 1,000,000 outgoing edges directly in CSR layout
    graph.insert_entity_direct(Entity::new(start, "StartNode", "Type"))?;
    graph.insert_entity_direct(Entity::new(hub, "HubNode", "Supernode"))?;
    graph.insert_edge_direct(start, hub, 1.0).await?;

    // Build 15,000 outgoing edges directly in CSR layout (exceeds MAX_VISITED_NODES = 10,000)
    let num_neighbors = 15_000usize;
    for i in 0..num_neighbors {
        let leaf_id = EntityId::new(3 + i as u64);
        graph.insert_entity_direct(Entity::new(leaf_id, "Leaf", "Type"))?;
        graph.insert_edge_direct(hub, leaf_id, 0.9).await?;
    }
    graph.compact();

    let start_time = std::time::Instant::now();
    let results = graph.traverse(start, 2).await?;
    let elapsed = start_time.elapsed();

    assert!(
        elapsed.as_millis() < 1000,
        "1M neighbor hub node BFS traversal must terminate in < 1 second, took {:?}",
        elapsed
    );
    assert!(
        results.len() <= MAX_VISITED_NODES,
        "Traversal result count must be capped by MAX_VISITED_NODES ({MAX_VISITED_NODES}), got {}",
        results.len()
    );
    assert_eq!(
        results.len(),
        MAX_VISITED_NODES - 1, // Start node excluded from result list
        "Visited cap includes hub and leaves, total returned results equals MAX_VISITED_NODES - 1"
    );
    Ok(())
}

#[tokio::test]
async fn test_consistency_enforcer_contradiction_suppression() {
    let graph = Arc::new(CsrGraph::with_consistency_enforcer(3));
    let from = EntityId::new(10);
    let to = EntityId::new(20);
    let pred_hash = [7u8; 32];
    let object_repr = b"ContradictoryValue".to_vec();

    // 1st insertion: recorded, not yet suppressed (count = 1)
    let res1 = graph
        .add_edge(
            from,
            to,
            1.0,
            None,
            None,
            None,
            None,
            None,
            Some(pred_hash),
            Some(object_repr.clone()),
        )
        .await;
    assert!(res1.is_ok(), "First insertion should succeed");

    // 2nd insertion: recorded, not yet suppressed (count = 2)
    let res2 = graph
        .add_edge(
            from,
            to,
            1.0,
            None,
            None,
            None,
            None,
            None,
            Some(pred_hash),
            Some(object_repr.clone()),
        )
        .await;
    assert!(res2.is_ok(), "Second insertion should succeed");

    // 3rd insertion: recorded, reaches suppression threshold (count = 3) -> suppressed!
    let res3 = graph
        .add_edge(
            from,
            to,
            1.0,
            None,
            None,
            None,
            None,
            None,
            Some(pred_hash),
            Some(object_repr),
        )
        .await;
    assert!(
        res3.is_err(),
        "Third insertion should fail due to consistency enforcer contradiction suppression"
    );
    let err = res3.unwrap_err();
    assert!(
        matches!(err, ContextraError::PolicyViolation(ref msg) if msg.contains("Contradictory edge suppressed by consistency enforcer")),
        "Expected policy violation error, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_default_csr_graph_no_consistency_check() {
    let graph = Arc::new(CsrGraph::new());
    let from = EntityId::new(100);
    let to = EntityId::new(200);
    let pred_hash = [9u8; 32];
    let object_repr = b"SomeValue".to_vec();

    // Standard CsrGraph::new() has consistency_enforcer = None, so insertion never blocks
    for i in 1..=5 {
        let res = graph
            .add_edge(
                from,
                to,
                1.0,
                None,
                None,
                None,
                None,
                None,
                Some(pred_hash),
                Some(object_repr.clone()),
            )
            .await;
        assert!(
            res.is_ok(),
            "Insertion {i} on default CsrGraph without consistency enforcer should always succeed"
        );
    }
}

#[tokio::test]
async fn test_csr_is_entity_deleted_returns_false_for_live_entity() {
    use contextra_store::{LsmConfig, LsmStorage};

    let dir = tempfile::tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(),
    );
    let graph = CsrGraph::with_storage(storage.clone());
    let entity_live = EntityId::new(10);
    let entity_deleted = EntityId::new(20);

    // No storage marker exists for entity_live -> returns false
    assert!(!graph.is_entity_deleted(entity_live).await);

    // Put deletion tombstone key in LSM storage for entity_deleted
    let tx = TxId::new(1);
    let tombstone_key = format!("graph:entity:deleted:{}", entity_deleted.0);
    storage
        .put(tx, tombstone_key.as_bytes(), b"deleted")
        .await
        .unwrap();
    storage.commit(tx).await.unwrap();

    // Marker exists -> returns true
    assert!(graph.is_entity_deleted(entity_deleted).await);
    // Live entity still returns false
    assert!(!graph.is_entity_deleted(entity_live).await);
}

#[test]
#[cfg(feature = "edge-reinforcement-learning")]
fn test_edge_store_invalidated_after_compact() {
    let mut inner = GraphInner::new();
    let entity_a = EntityId::new(1);
    let entity_b = EntityId::new(2);

    let idx_a = inner.get_or_create_index(entity_a);
    let idx_b = inner.get_or_create_index(entity_b);

    // (1) Anlegen einer Kante A -> B in pending_edges
    inner
        .pending_edges
        .entry(idx_a)
        .or_default()
        .push(EdgePayload {
            target: idx_b,
            weight: 1.0,
            tx_valid_from: None,
            tx_valid_to: None,
            business_valid_from: None,
            business_valid_to: None,
            source_doc_id: None,
        });
    inner.pending_edge_count += 1;

    // (2) `outgoing_edges_mut(A)` aufrufen um den edge_store-Cache zu befüllen
    let cached = inner.outgoing_edges_mut(entity_a);
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].target, entity_b);

    // (3) Kante über Tombstone-Mechanismus entfernen
    inner.tombstoned_edges.insert((idx_a, idx_b));

    // (4) `compact()` aufrufen
    inner.compact();

    // (5) Beweisen, dass `outgoing_edges_mut(A)` danach die Kante NICHT mehr zurückgibt
    let cached_after_compact = inner.outgoing_edges_mut(entity_a);
    assert_eq!(
            cached_after_compact.len(),
            0,
            "edge_store must be cleared by compact() so deleted/tombstoned edges are no longer returned"
        );
}
