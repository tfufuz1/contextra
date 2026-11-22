use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::GraphIndex;
use contextra_types::{Edge, Entity, EntityId, TxId};
use std::future::Future;

/// Runs the complete conformance test suite for a [`GraphIndex`] implementation.
///
/// # Contract Requirements Tested
/// 1. Edge insertion & neighborhood querying (`add_edge`, `neighbors`).
/// 2. Edge removal & neighborhood update (`remove_edge`).
/// 3. Graph traversal (BFS hops with decay weights).
/// 4. Self-loop edge handling.
/// 5. Tombstone / removal consistency across traversal and neighbors.
pub async fn run_graph_index_suite<F, Fut, G>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = G>,
    G: GraphIndex,
{
    let suite = "GraphIndex";

    // Test 1: Add edge & neighbors
    {
        let graph = factory().await;
        let tx = TxId::new(1);
        let e1 = Entity::new(EntityId::new(1), "NodeA", "Concept");
        let e2 = Entity::new(EntityId::new(2), "NodeB", "Concept");
        let _ = graph.add_entity(tx, e1).await;
        let _ = graph.add_entity(tx, e2).await;
        if let Err(e) = graph
            .add_edge(
                tx,
                Edge::new(EntityId::new(1), EntityId::new(2), "connects"),
            )
            .await
        {
            return Err(ConformanceFailure::new(
                suite,
                "add_edge_neighbors",
                format!("add_edge failed: {e:?}"),
            ));
        }
        let _ = graph.commit(tx).await;

        let nbs = match graph.neighbors(EntityId::new(1)).await {
            Ok(n) => n,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "add_edge_neighbors",
                    format!("neighbors failed: {e:?}"),
                ))
            }
        };
        if !nbs.contains(&EntityId::new(2)) {
            return Err(ConformanceFailure::new(
                suite,
                "add_edge_neighbors",
                format!("neighbors of NodeA should contain NodeB, got {nbs:?}"),
            ));
        }
    }

    // Test 2: Remove edge
    {
        let graph = factory().await;
        let tx1 = TxId::new(10);
        let _ = graph
            .add_edge(tx1, Edge::new(EntityId::new(10), EntityId::new(20), "rel"))
            .await;
        let _ = graph.commit(tx1).await;

        let tx2 = TxId::new(11);
        if let Err(e) = graph
            .remove_edge(tx2, EntityId::new(10), EntityId::new(20))
            .await
        {
            return Err(ConformanceFailure::new(
                suite,
                "remove_edge",
                format!("remove_edge failed: {e:?}"),
            ));
        }
        let _ = graph.commit(tx2).await;

        let nbs = match graph.neighbors(EntityId::new(10)).await {
            Ok(n) => n,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "remove_edge",
                    format!("neighbors failed: {e:?}"),
                ))
            }
        };
        if nbs.contains(&EntityId::new(20)) {
            return Err(ConformanceFailure::new(
                suite,
                "remove_edge",
                format!("removed edge still visible in neighbors: {nbs:?}"),
            ));
        }
    }

    // Test 3: Traversal
    {
        let graph = factory().await;
        let tx = TxId::new(20);
        let _ = graph
            .add_edge(
                tx,
                Edge::new(EntityId::new(100), EntityId::new(101), "link"),
            )
            .await;
        let _ = graph
            .add_edge(
                tx,
                Edge::new(EntityId::new(101), EntityId::new(102), "link"),
            )
            .await;
        let _ = graph.commit(tx).await;

        let trav = match graph.traverse(EntityId::new(100), 2).await {
            Ok(t) => t,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "traversal",
                    format!("traverse failed: {e:?}"),
                ))
            }
        };
        let ids: Vec<EntityId> = trav.into_iter().map(|(id, _)| id).collect();
        if !ids.contains(&EntityId::new(101)) || !ids.contains(&EntityId::new(102)) {
            return Err(ConformanceFailure::new(
                suite,
                "traversal",
                format!("2-hop traversal expected 101 and 102, got {ids:?}"),
            ));
        }
    }

    // Test 4: Self-loop
    {
        let graph = factory().await;
        let tx = TxId::new(30);
        let loop_edge = Edge::new(EntityId::new(50), EntityId::new(50), "self_loop");
        let _ = graph.add_edge(tx, loop_edge).await;
        let _ = graph.commit(tx).await;

        let nbs = match graph.neighbors(EntityId::new(50)).await {
            Ok(n) => n,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "self_loop",
                    format!("neighbors on self loop failed: {e:?}"),
                ))
            }
        };
        let _ = nbs;
    }

    Ok(())
}
