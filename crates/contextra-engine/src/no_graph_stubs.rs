//! Stubs for `contextra-graph` when feature `graph` is disabled.

#[cfg(not(feature = "graph"))]
use contextra_ports::{
    BoxFuture, Edge, Entity, EntityId, GraphIndex, GraphIndexStats, Result, TxId,
};
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "graph"))]
use std::sync::Arc;

#[cfg(not(feature = "graph"))]
#[derive(Debug, Clone, Default)]
pub struct CsrGraph;

#[cfg(not(feature = "graph"))]
impl CsrGraph {
    pub fn new() -> Self {
        Self
    }

    pub async fn load_from_storage<S>(_storage: &S) -> contextra_types::Result<Self> {
        Ok(Self)
    }

    pub fn set_storage<S>(&mut self, _storage: Arc<S>) {}

    pub async fn remove_entity(&self, _tx: TxId, _id: EntityId) -> contextra_types::Result<()> {
        Err(contextra_types::ContextraError::capability_unsupported(
            "graph",
            "graph feature is disabled",
        ))
    }

    pub async fn get_communities_batch(
        &self,
        _ids: &[EntityId],
    ) -> contextra_types::Result<ahash::AHashMap<EntityId, u64>> {
        Err(contextra_types::ContextraError::capability_unsupported(
            "graph",
            "graph feature is disabled",
        ))
    }

    pub fn set_communities_batch(&self, _assignments: &[CommunityAssignment]) {}

    pub fn get_hyperedge(&self, _id: hyperedge::HyperEdgeId) -> Option<hyperedge::HyperEdge> {
        None
    }

    pub fn hyperedges_for_entity(&self, _entity: EntityId) -> Vec<hyperedge::HyperEdgeId> {
        vec![]
    }

    pub fn insert_hyperedge(&self, _hyperedge: hyperedge::HyperEdge) {}
}

#[cfg(not(feature = "graph"))]
impl GraphIndex for CsrGraph {
    fn traverse<'a>(
        &'a self,
        _start_node: EntityId,
        _max_hops: usize,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            Err(contextra_types::ContextraError::capability_unsupported(
                "graph",
                "graph feature is disabled",
            ))
        })
    }

    fn add_entity<'a>(&'a self, _tx: TxId, _entity: Entity) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            Err(contextra_types::ContextraError::capability_unsupported(
                "graph",
                "graph feature is disabled",
            ))
        })
    }

    fn add_edge<'a>(&'a self, _tx: TxId, _edge: Edge) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            Err(contextra_types::ContextraError::capability_unsupported(
                "graph",
                "graph feature is disabled",
            ))
        })
    }

    fn commit<'a>(&'a self, _tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback<'a>(&'a self, _tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId(0)) })
    }

    fn len<'a>(&'a self) -> BoxFuture<'a, usize> {
        Box::pin(async move { 0 })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<GraphIndexStats>> {
        Box::pin(async move {
            Err(contextra_types::ContextraError::capability_unsupported(
                "graph",
                "graph feature is disabled",
            ))
        })
    }
}

pub mod hyperedge {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct HyperEdgeId(pub u64);

    impl HyperEdgeId {
        pub fn new(id: u64) -> Self {
            Self(id)
        }

        pub fn inner(&self) -> u64 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct RoleId(pub u32);

    impl RoleId {
        pub fn new(id: u32) -> Self {
            Self(id)
        }

        pub fn inner(&self) -> u32 {
            self.0
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RoleBinding {
        pub role_id: RoleId,
        pub entity_id: contextra_ports::EntityId,
    }

    impl RoleBinding {
        pub fn new(role_id: RoleId, entity_id: contextra_ports::EntityId) -> Self {
            Self { role_id, entity_id }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HyperEdge {
        pub id: HyperEdgeId,
        pub edge_type: super::csr::EdgeType,
        pub bindings: Vec<RoleBinding>,
        pub prefix: &'static str,
    }

    impl HyperEdge {
        pub fn new(
            id: HyperEdgeId,
            edge_type: super::csr::EdgeType,
            bindings: Vec<RoleBinding>,
            prefix: &'static str,
        ) -> Self {
            Self {
                id,
                edge_type,
                bindings,
                prefix,
            }
        }
    }

    pub const HYPEREDGE_PREFIX: &str = "__hyperedge:";
}

pub mod csr {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum EdgeType {
        Default,
    }
}

pub async fn cascade_invalidate_edges_for_superseded_doc<G>(
    _graph: &G,
    _doc_id: contextra_types::DocId,
) -> contextra_types::Result<()> {
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityAssignment {
    pub entity_id: contextra_ports::EntityId,
    pub community_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommunityDetectionConfig {
    pub auto_trigger_threshold: u64,
}

pub async fn detect_communities<G>(
    _graph: &G,
    _config: &CommunityDetectionConfig,
) -> contextra_types::Result<Vec<CommunityAssignment>> {
    Err(contextra_types::ContextraError::capability_unsupported(
        "graph",
        "graph feature is disabled",
    ))
}

pub mod percolation {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct PercolationConfig;

    pub fn compute_percolation_health(_num_nodes: usize, _num_edges: usize) -> f32 {
        1.0
    }

    pub fn should_trigger_rebonding(_phi: f32, _config: &PercolationConfig) -> bool {
        false
    }

    pub fn find_rebonding_candidates<G>(
        _graph: &G,
    ) -> Vec<(contextra_ports::EntityId, contextra_ports::EntityId)> {
        vec![]
    }
}
