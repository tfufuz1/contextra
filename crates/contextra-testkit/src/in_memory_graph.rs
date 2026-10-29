use contextra_ports::{BoxFuture, GraphIndex, GraphIndexStats};
use contextra_types::{Edge, Entity, EntityId, Result, TxId};
use parking_lot::RwLock;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Debug, Clone)]
enum StagedGraphOp {
    AddEntity(Entity),
    AddEdge(Edge),
    RemoveEdge { from: EntityId, to: EntityId },
}

type EntityMap = BTreeMap<EntityId, Entity>;
type EdgeMap = BTreeMap<(EntityId, EntityId), Edge>;
type StagedGraphTxMap = BTreeMap<TxId, Vec<StagedGraphOp>>;

/// An in-memory reference implementation of [`GraphIndex`] for testing.
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
    /// Creates a new empty `InMemoryGraphIndex`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl GraphIndex for InMemoryGraphIndex {
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

    fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.staged.write().remove(&tx);
            Ok(())
        })
    }

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.staged.write().clear();
            Ok(())
        })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(*self.last_tx.read()) })
    }

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

    fn len<'a>(&'a self) -> BoxFuture<'a, usize> {
        Box::pin(async move { self.entities.read().len() })
    }

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
