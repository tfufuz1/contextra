//! # In-Memory-Graphindex für Tests (`in_memory_graph`)
//!
//! Dieses Modul stellt mit [`InMemoryGraphIndex`] eine deterministische
//! Referenzimplementierung des [`contextra_ports::GraphIndex`]-Ports bereit.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Port:** Implementiert [`contextra_ports::GraphIndex`].
//! - **Graphmodell:** Gerichteter Graph aus Entitäten ([`contextra_types::Entity`], [`contextra_types::EntityId`])
//!   und gewichteten Kanten ([`contextra_types::Edge`]).
//! - **Traversierung:** Breitensuche (BFS) mit exponentiellem Dämpfungsfaktor (`* 0.8 * edge.weight`).
//! - **Transaktionales Staging:** Isolierung unbestätigter Graph-Operationen über `TxId`.

use contextra_ports::{BoxFuture, GraphIndex, GraphIndexStats};
use contextra_types::{Edge, Entity, EntityId, Result, TxId};
use parking_lot::RwLock;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

// ANCHOR[contextra-testkit:STAGED-GRAPH-OP-ENUM-001]
/// Unbestätigte Graph-Operation innerhalb einer Transaktion.
#[derive(Debug, Clone)]
enum StagedGraphOp {
    /// Hinzufügen oder Aktualisieren einer Entität (Knoten).
    AddEntity(Entity),
    /// Hinzufügen einer gerichteten Kante.
    AddEdge(Edge),
    /// Entfernen einer Kante zwischen zwei Entitäten.
    RemoveEdge { from: EntityId, to: EntityId },
}

/// Knotenbestand: Entitäts-ID -> Entität.
type EntityMap = BTreeMap<EntityId, Entity>;
/// Kantenbestand: (Von, Nach) -> Kante.
type EdgeMap = BTreeMap<(EntityId, EntityId), Edge>;
/// Staging-Puffer: Transaktions-ID -> Liste von Graph-Operationen.
type StagedGraphTxMap = BTreeMap<TxId, Vec<StagedGraphOp>>;

// ANCHOR[contextra-testkit:IN-MEM-GRAPH-STRUCT-001]
/// Thread-sichere In-Memory-Referenzimplementierung von [`GraphIndex`] für Tests.
///
/// **Zweck:** Ermöglicht Wissensgraph- und Beziehungstests ohne externe Graphdatenbanken.
///
/// **Datenhaltung:**
/// Speichert Knoten in einer `BTreeMap<EntityId, Entity>` und gerichtete Kanten
/// in einer `BTreeMap<(EntityId, EntityId), Edge>`, geschützt über `parking_lot::RwLock`.
#[derive(Debug, Clone)]
pub struct InMemoryGraphIndex {
    entities: Arc<RwLock<EntityMap>>,
    edges: Arc<RwLock<EdgeMap>>,
    staged: Arc<RwLock<StagedGraphTxMap>>,
    last_tx: Arc<RwLock<TxId>>,
}

impl Default for InMemoryGraphIndex {
    fn default() -> Self {
        Self {
            entities: Arc::new(RwLock::new(BTreeMap::new())),
            edges: Arc::new(RwLock::new(BTreeMap::new())),
            staged: Arc::new(RwLock::new(BTreeMap::new())),
            last_tx: Arc::new(RwLock::new(TxId::new(0))),
        }
    }
}

impl InMemoryGraphIndex {
    // ANCHOR[contextra-testkit:IN-MEM-GRAPH-NEW-001]
    /// Erstellt einen neuen, leeren `InMemoryGraphIndex`.
    pub fn new() -> Self {
        Self::default()
    }
}

// ANCHOR[contextra-testkit:IN-MEM-GRAPH-TRAIT-IMPL-001]
impl GraphIndex for InMemoryGraphIndex {
    /// Stellt das Hinzufügen einer Entität im Transaktionskontext bereit.
    fn add_entity<'a>(&'a self, tx: TxId, entity: Entity) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut staged = self.staged.write();
            staged
                .entry(tx)
                .or_default()
                .push(StagedGraphOp::AddEntity(entity));
            Ok(())
        })
    }

    /// Stellt das Hinzufügen einer gerichteten Kante im Transaktionskontext bereit.
    fn add_edge<'a>(&'a self, tx: TxId, edge: Edge) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut staged = self.staged.write();
            staged
                .entry(tx)
                .or_default()
                .push(StagedGraphOp::AddEdge(edge));
            Ok(())
        })
    }

    /// Stellt das Entfernen einer Kante im Transaktionskontext bereit.
    fn remove_edge<'a>(
        &'a self,
        tx: TxId,
        from: EntityId,
        to: EntityId,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut staged = self.staged.write();
            staged
                .entry(tx)
                .or_default()
                .push(StagedGraphOp::RemoveEdge { from, to });
            Ok(())
        })
    }

    /// Bestätigt alle ausstehenden Operationen der Transaktion `tx`.
    ///
    /// **Ablauf:**
    /// 1. Entnimmt Staging-Operationen für `tx`.
    /// 2. Wendet Knoten- und Kantenänderungen deterministisch auf `entities` und `edges` an.
    /// 3. Aktualisiert `last_tx`.
    fn commit<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let ops = {
                let mut staged = self.staged.write();
                staged.remove(&tx)
            };

            if let Some(ops) = ops {
                let mut entities = self.entities.write();
                let mut edges = self.edges.write();
                for op in ops {
                    match op {
                        StagedGraphOp::AddEntity(e) => {
                            entities.insert(e.id, e);
                        }
                        StagedGraphOp::AddEdge(e) => {
                            edges.insert((e.from, e.to), e);
                        }
                        StagedGraphOp::RemoveEdge { from, to } => {
                            edges.remove(&(from, to));
                        }
                    }
                }
                *self.last_tx.write() = tx;
            }
            Ok(())
        })
    }

    /// Verwirft die unbestätigten Graph-Änderungen der Transaktion `tx`.
    fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.staged.write().remove(&tx);
            Ok(())
        })
    }

    /// Setzt alle offenen Staging-Puffer zurück.
    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.staged.write().clear();
            Ok(())
        })
    }

    /// Liefert die ID der letzten bestätigten Transaktion.
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(*self.last_tx.read()) })
    }

    // ANCHOR[contextra-testkit:IN-MEM-GRAPH-NEIGHBORS-001]
    /// Liefert alle ausgehenden Nachbarknoten eines Startknotens.
    fn neighbors<'a>(&'a self, start_node: EntityId) -> BoxFuture<'a, Result<Vec<EntityId>>> {
        Box::pin(async move {
            let edges = self.edges.read();
            let mut result = BTreeSet::new();
            for ((from, to), _) in edges.iter() {
                if *from == start_node {
                    result.insert(*to);
                }
            }
            Ok(result.into_iter().collect())
        })
    }

    // ANCHOR[contextra-testkit:IN-MEM-GRAPH-TRAVERSE-001]
    /// Führt eine gewichtete Breitensuche (BFS) ab `start_node` bis zu `max_hops` durch.
    ///
    /// **Ablauf:**
    /// 1. Startet mit Score 1.0 am Startknoten.
    /// 2. Iteriert bis zu `max_hops` Mal über die aktuelle Front (`current_frontier`).
    /// 3. Berechnet Folgescores: `next_score = score * 0.8 * edge.weight`.
    /// 4. Merkt sich in `visited` für jeden Knoten den maximalen erreichten Score.
    /// 5. Sortiert gefundene Knoten absteigend nach Score.
    fn traverse<'a>(
        &'a self,
        start_node: EntityId,
        max_hops: usize,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            let edges = self.edges.read();
            let mut visited = BTreeMap::new();
            let mut current_frontier = vec![(start_node, 1.0f32)];

            for _hop in 0..max_hops {
                let mut next_frontier = Vec::new();
                for (curr, score) in current_frontier {
                    for ((from, to), edge) in edges.iter() {
                        if *from == curr {
                            let next_score = score * 0.8 * edge.weight;
                            if !visited.contains_key(to)
                                || visited.get(to).copied().unwrap_or(0.0) < next_score
                            {
                                visited.insert(*to, next_score);
                                next_frontier.push((*to, next_score));
                            }
                        }
                    }
                }
                if next_frontier.is_empty() {
                    break;
                }
                current_frontier = next_frontier;
            }

            let mut results: Vec<(EntityId, f32)> = visited.into_iter().collect();
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            Ok(results)
        })
    }

    /// Gibt die Gesamtzahl der bestätigten Entitäten zurück.
    fn len<'a>(&'a self) -> BoxFuture<'a, usize> {
        Box::pin(async move { self.entities.read().len() })
    }

    /// Liefert statistische Kennzahlen zum In-Memory-Graphindex.
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<GraphIndexStats>> {
        Box::pin(async move {
            let num_entities = self.entities.read().len();
            let num_edges = self.edges.read().len();
            Ok(GraphIndexStats {
                num_entities,
                num_edges,
                memory_usage_bytes: (num_entities + num_edges) * 64,
            })
        })
    }
}
