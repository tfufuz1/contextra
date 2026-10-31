//! Modul `mutation_sequence_tests` im Ring 0 (Graph-Subsystem) von Contextra.
//!
//! Stellt Kernkomponenten für Graphtraversierung, CSR-Repräsentationen und
//! Analysemethoden bereit. Wird von Integrationsschichten und Algorithmenmodulen verwendet.

use crate::csr::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{Entity, EntityId, TxId};
use std::sync::Arc;

fn create_entity(id: u64) -> Entity {
    Entity::new(EntityId::new(id), "label", "type")
}

#[tokio::test]
async fn test_csr_mutation_sequence_equivalence() {
    let g = Arc::new(CsrGraph::new());
    let tx2 = TxId::new(2);
    let tx3 = TxId::new(3);

    for i in 1..=10 {
        g.insert_entity_direct(create_entity(i)).unwrap();
    }
    for i in 1..10 {
        g.add_edge(
            EntityId::new(i),
            EntityId::new(i + 1),
            1.0,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    }

    let mem1 = g.estimate_memory_bytes();
    assert!(mem1 > 0);

    g.compact_async().await.unwrap();

    let inner1 = g.inner_read();
    assert_eq!(inner1.targets.len(), 9);

    g.tombstone_edges_direct(&[(EntityId::new(5), EntityId::new(6))], tx2)
        .unwrap();
    g.add_edge(
        EntityId::new(5),
        EntityId::new(10),
        1.0,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    g.compact_async().await.unwrap();

    let inner2 = g.inner_read();
    assert_eq!(inner2.targets.len(), 9);

    let res = GraphIndex::traverse(g.as_ref(), EntityId::new(5), 1)
        .await
        .unwrap();
    let neighbors: Vec<EntityId> = res.into_iter().map(|(id, _)| id).collect();
    assert!(!neighbors.contains(&EntityId::new(6)));
    assert!(neighbors.contains(&EntityId::new(10)));

    g.add_edge(
        EntityId::new(6),
        EntityId::new(1),
        1.0,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    g.tombstone_edges_direct(&[(EntityId::new(1), EntityId::new(2))], tx3)
        .unwrap();
    g.compact_async().await.unwrap();

    let res1 = GraphIndex::traverse(g.as_ref(), EntityId::new(1), 1)
        .await
        .unwrap();
    assert!(
        res1.is_empty(),
        "1 -> 2 was tombstoned, so traverse from 1 should be empty"
    );

    let res6 = GraphIndex::traverse(g.as_ref(), EntityId::new(6), 1)
        .await
        .unwrap();
    let n6: Vec<EntityId> = res6.into_iter().map(|(id, _)| id).collect();
    assert!(n6.contains(&EntityId::new(1)));
}
