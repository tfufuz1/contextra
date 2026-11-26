//! # Konformitätsprüfsuite für den `GraphIndex`-Port (`conformance/graph`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen (Contract Invariants)
//! für Wissensgraphen-Indizes des [`contextra_ports::GraphIndex`]-Ports.
//!
//! ## Getestete Verträge
//! 1. **Kanten-Einfügen & Nachbarschaft:** Kanten mit `add_edge` tauchen in `neighbors` auf.
//! 2. **Kanten-Entfernen:** Mit `remove_edge` entfernte Kanten verschwinden aus den Nachbarn.
//! 3. **Mehrstufige Traversierung:** `traverse` findet Knoten über mehrere Hops hinweg (z. B. 2 Hops).
//! 4. **Selbstkanten (Self-Loops):** Kanten von einem Knoten auf sich selbst führen nicht zu Endlosschleifen.

use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::GraphIndex;
use contextra_types::{Edge, Entity, EntityId, TxId};
use std::future::Future;

// ANCHOR[contextra-testkit:CONF-GRAPH-SUITE-001]
/// Führt die vollständige Konformitätsprüfung für eine [`GraphIndex`]-Implementierung durch.
///
/// **Parameter:**
/// - `factory`: Asynchrone Closure, die eine frische Graphindex-Instanz erzeugt.
///
/// **Ablauf:**
/// 1. Test 1: Entitäten anlegen, Kante hinzufügen und Nachbarschaft abfragen.
/// 2. Test 2: Kante löschen und prüfen, dass sie nicht mehr als Nachbar erscheint.
/// 3. Test 3: Pfad über zwei Kanten spannen und 2-Hop-Traversierung prüfen.
/// 4. Test 4: Selbstkante anlegen und Nachbarschaftsfunktion auf Robustheit prüfen.
///
/// **Rückgabe:** `Ok(())` bei Vertragskonformität, andernfalls `Err(ConformanceFailure)`.
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
