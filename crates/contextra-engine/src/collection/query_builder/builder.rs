use super::strategy::SearchStrategy;
use super::Collection;
#[allow(deprecated)]
use crate::filter::MetadataFilter;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{DocId, EntityId, FilterExpr, FusionStrategy, FusionWeights, MemoryType};

#[cfg(feature = "adaptive-candidate-pool-sizing")]
use std::sync::Arc;

/// Default candidate pool expansion multiplier for Cross-Encoder reranking (10x requested k).
///
/// Grounded in empirical evaluation from T2-RAGBench (arXiv:2604.01733), showing Recall@5 = 0.888
/// with ~100 candidate items compared to 0.458 with only 20 candidates.
pub const DEFAULT_RERANK_POOL_MULTIPLIER: usize = 10;

/// Default upper cap for pre-reranking candidate pool expansion (200 candidates).
///
/// Prevents retrieval cost explosion for queries with large `k`.
pub const DEFAULT_RERANK_POOL_MAX: usize = 200;

/// Fluent query builder for unifying vector, text, graph, and hybrid search operations.
pub struct HybridQueryBuilder<'a, S: StorageEngine, V: VectorIndex> {
    pub(super) collection: &'a Collection<S, V>,
    pub(super) text: Option<String>,
    pub(super) vector: Option<Vec<f32>>,
    pub(super) k: Option<usize>,
    pub(super) weights: Option<FusionWeights>,
    pub(super) fusion_strategy: Option<FusionStrategy>,
    pub(super) strategy: Option<SearchStrategy>,
    pub(super) filter: Option<FilterExpr>,
    pub(super) anchor_entities: Option<Vec<EntityId>>,
    pub(super) same_community_as: Option<EntityId>,
    pub(super) memory_type_filter: Option<Vec<MemoryType>>,
    pub(super) include_superseded: bool,
    pub(super) include_provenance: bool,
    pub(super) filter_fn: Option<Box<dyn Fn(DocId) -> bool + Send + Sync>>,
    pub(super) hard_scope: Option<super::scope::ScopeConstraint>,
    #[cfg(feature = "reranking")]
    pub(super) reranker: Option<&'a contextra_infer_onnx::CrossEncoderReranker>,
    pub(super) rerank_pool_multiplier: Option<usize>,
    pub(super) rerank_pool_max: Option<usize>,
    #[cfg(feature = "adaptive-candidate-pool-sizing")]
    pub(super) pid_controller: Option<Arc<parking_lot::Mutex<contextra_adapt::PidController>>>,
    pub(super) seq: Option<u64>,
    pub(super) as_of_timestamp: Option<u64>,
    pub(super) query_timestamp: Option<u64>,
    pub(super) current_tx: Option<contextra_types::TxId>,
}

impl<'a, S: StorageEngine, V: VectorIndex> HybridQueryBuilder<'a, S, V> {
    /// Creates a new `HybridQueryBuilder` for a given `Collection`.
    pub fn new(collection: &'a Collection<S, V>) -> Self {
        Self {
            collection,
            text: None,
            vector: None,
            k: None,
            weights: None,
            fusion_strategy: None,
            strategy: None,
            filter: None,
            anchor_entities: None,
            same_community_as: None,
            memory_type_filter: None,
            include_superseded: false,
            include_provenance: false,
            filter_fn: None,
            hard_scope: None,
            #[cfg(feature = "reranking")]
            reranker: None,
            rerank_pool_multiplier: None,
            rerank_pool_max: None,
            #[cfg(feature = "adaptive-candidate-pool-sizing")]
            pid_controller: None,
            seq: None,
            as_of_timestamp: None,
            query_timestamp: None,
            current_tx: None,
        }
    }

    /// Sets full-text search query string.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Sets dense vector query embedding.
    pub fn embedding(mut self, embedding: impl AsRef<[f32]>) -> Self {
        self.vector = Some(embedding.as_ref().to_vec());
        self
    }

    /// Alias for `.embedding()` to specify dense vector query embedding.
    pub fn vector(self, vector: impl AsRef<[f32]>) -> Self {
        self.embedding(vector)
    }

    /// Sets top-k maximum result count.
    pub fn k(mut self, k: usize) -> Self {
        self.k = Some(k);
        self
    }

    /// Sets custom signal fusion weights using `SignalWeights` or `FusionWeights`.
    pub fn weights<W>(mut self, weights: W) -> Self
    where
        W: TryInto<FusionWeights, Error = contextra_types::ContextraError>,
    {
        if let Ok(fw) = weights.try_into() {
            self.weights = Some(fw);
        }
        self
    }

    /// Sets custom signal fusion weights directly using `FusionWeights`.
    pub fn fusion_weights(mut self, weights: FusionWeights) -> Self {
        self.weights = Some(weights);
        self
    }

    /// Sets search fusion strategy (`FusionStrategy::Rrf` or `FusionStrategy::ScoreNormalized`).
    pub fn fusion_strategy(mut self, strategy: FusionStrategy) -> Self {
        self.fusion_strategy = Some(strategy);
        self
    }

    /// Sets metadata expression filter (`FilterExpr`).
    pub fn filter(mut self, filter: FilterExpr) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Sets legacy metadata filter (`MetadataFilter`).
    #[allow(deprecated)]
    pub fn metadata_filter(mut self, filter: MetadataFilter) -> Self {
        if let Ok(expr) = FilterExpr::try_from(filter) {
            self.filter = Some(expr);
        }
        self
    }

    /// Sets hybrid search / graph traversal strategy.
    pub fn strategy(mut self, strategy: impl Into<SearchStrategy>) -> Self {
        self.strategy = Some(strategy.into());
        self
    }

    /// Sets anchor seed entities for graph traversal signal.
    pub fn anchor_entities(mut self, anchors: impl IntoIterator<Item = EntityId>) -> Self {
        self.anchor_entities = Some(anchors.into_iter().collect());
        self
    }

    /// Alias for `.anchor_entities()`.
    pub fn anchors(self, anchors: impl IntoIterator<Item = EntityId>) -> Self {
        self.anchor_entities(anchors)
    }

    /// Sets target entity for community context boosting/filtering.
    pub fn same_community_as(mut self, entity_id: EntityId) -> Self {
        self.same_community_as = Some(entity_id);
        self
    }

    /// Sets cognitive memory type filter (Pre-RRF filter).
    pub fn memory_type_filter(mut self, types: impl IntoIterator<Item = MemoryType>) -> Self {
        self.memory_type_filter = Some(types.into_iter().collect());
        self
    }

    /// Sets whether to calculate and attach ProvenanceRecord to output results.
    pub fn include_provenance(mut self, include: bool) -> Self {
        self.include_provenance = include;
        self
    }

    /// Sets whether to include superseded documents (Post-RRF Supersedes Displacement, ADR-038).
    pub fn include_superseded(mut self, include: bool) -> Self {
        self.include_superseded = include;
        self
    }

    /// Alias for `.memory_type_filter()`.
    pub fn memory_types(self, types: impl IntoIterator<Item = MemoryType>) -> Self {
        self.memory_type_filter(types)
    }

    /// Sets custom filter predicate function for vector search candidates.
    pub fn filter_fn(mut self, f: impl Fn(DocId) -> bool + Send + Sync + 'static) -> Self {
        self.filter_fn = Some(Box::new(f));
        self
    }

    /// Sets optional CrossEncoder reranker for post-retrieval ranking.
    #[cfg(feature = "reranking")]
    pub fn reranker(mut self, reranker: &'a contextra_infer_onnx::CrossEncoderReranker) -> Self {
        self.reranker = Some(reranker);
        self
    }

    /// Sets the candidate pool multiplier for pre-reranking candidate expansion.
    ///
    /// Default: 10 (`DEFAULT_RERANK_POOL_MULTIPLIER`), yielding ~100 candidates for `k=10`
    /// per T2-RAGBench empirical recall optimization.
    pub fn rerank_pool_multiplier(mut self, multiplier: usize) -> Self {
        self.rerank_pool_multiplier = Some(multiplier);
        self
    }

    /// Sets the upper cap for pre-reranking candidate pool expansion.
    ///
    /// Default: 200 (`DEFAULT_RERANK_POOL_MAX`), capping pre-retrieval pool size for large `k`.
    pub fn rerank_pool_max(mut self, max: usize) -> Self {
        self.rerank_pool_max = Some(max);
        self
    }

    /// Sets optional PID controller for adaptive reranking candidate pool sizing.
    #[cfg(feature = "adaptive-candidate-pool-sizing")]
    pub fn pid_controller(
        mut self,
        pid: Arc<parking_lot::Mutex<contextra_adapt::PidController>>,
    ) -> Self {
        self.pid_controller = Some(pid);
        self
    }

    /// Sets snapshot sequence number for MVCC snapshot-isolated queries.
    pub fn seq(mut self, seq_no: u64) -> Self {
        self.seq = Some(seq_no);
        self
    }

    /// Sets historical point-in-time "as-of" timestamp for temporal validity filtering (Post-RRF, Pre-Reranking).
    pub fn as_of(mut self, timestamp: u64) -> Self {
        self.as_of_timestamp = Some(timestamp);
        self
    }

    /// Sets query business timestamp for temporal validity filtering (Post-RRF, Pre-Reranking).
    pub fn query_timestamp(mut self, timestamp: u64) -> Self {
        self.query_timestamp = Some(timestamp);
        self
    }

    /// Sets current system transaction ID for temporal validity filtering (Post-RRF, Pre-Reranking).
    pub fn current_tx(mut self, tx: contextra_types::TxId) -> Self {
        self.current_tx = Some(tx);
        self
    }

    /// Configures builder options from an existing `HybridQuery` struct.
    pub fn query_config(mut self, query: &contextra_types::HybridQuery) -> Self {
        if let Some(ref text) = query.text_query {
            self.text = Some(text.clone());
        }
        if let Some(ref vector) = query.vector_query {
            self.vector = Some(vector.clone());
        }
        if let Some(ref start_node) = query.graph_start_node {
            if let Ok(eid) = EntityId::from_key(start_node) {
                self.anchor_entities = Some(vec![eid]);
            }
        }
        self.strategy = Some(query.graph_strategy.clone().into());
        self.weights = Some(query.fusion_weights.clone());
        self.fusion_strategy = Some(query.fusion_strategy);
        self.filter = query.filter.clone();
        self.same_community_as = query.same_community_as;
        self.memory_type_filter = query.memory_type_filter.clone();
        self.include_superseded = query.include_superseded;
        self.include_provenance = query.include_provenance;
        self.rerank_pool_multiplier = query.rerank_pool_multiplier;
        self.rerank_pool_max = query.rerank_pool_max;
        self.k = Some(query.k);
        self
    }
}

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Recommended unified entrypoint for search queries using fluent `HybridQueryBuilder`.
    pub fn query(&self) -> HybridQueryBuilder<'_, S, V> {
        HybridQueryBuilder::new(self)
    }
}
