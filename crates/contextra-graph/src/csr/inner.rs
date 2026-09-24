use arc_swap::ArcSwap;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use contextra_types::{DocId, Entity, EntityId, TxId};

use super::types::{EdgePayload, InternalIndex, StagedEdgePayload};

/// Inner state of the CsrGraph to manage contiguous storage.
#[derive(Clone)]
pub struct GraphInner {
    /// Mapping from public EntityId to internal contiguous index.
    pub(crate) id_map: HashMap<EntityId, InternalIndex>,
    /// Mapping from internal index back to EntityId.
    pub(crate) reverse_map: Vec<EntityId>,
    /// Entity metadata stored contiguously (Sentinel Entity with id EntityId::new(0) represents None).
    pub(crate) entities: Vec<Entity>,
    /// Community assignments mapping EntityId -> community_id.
    pub(crate) communities: HashMap<EntityId, u64>,
    /// Flag indicating whether communities have been loaded from storage or set in memory.
    pub(crate) communities_loaded: bool,

    /// CSR offsets array: offsets[i] is the start index in `targets` for node `i`.
    /// Length is nodes + 1.
    pub(crate) offsets: Vec<usize>,
    /// CSR targets array: contiguous list of neighbor internal indices.
    pub(crate) targets: Vec<InternalIndex>,
    /// CSR weights array: contiguous list of edge weights.
    pub(crate) weights: Vec<f32>,
    /// CSR valid_from array: contiguous list of bi-temporal tx_valid_from TxIds (TxId::INVALID represents None).
    pub(crate) tx_valid_froms: Vec<TxId>,
    /// CSR valid_to array: contiguous list of bi-temporal tx_valid_to TxIds (TxId::INVALID represents None).
    pub(crate) tx_valid_tos: Vec<TxId>,
    /// CSR business_valid_from array: contiguous list of business_valid_from timestamps (ms) (i64::MIN represents None).
    pub(crate) business_valid_froms: Vec<i64>,
    /// CSR business_valid_to array: contiguous list of business_valid_to timestamps (ms) (i64::MIN represents None).
    pub(crate) business_valid_tos: Vec<i64>,
    /// CSR source_doc_id array: contiguous list of optional source document IDs (DocId with inner() == 0 represents None).
    pub(crate) source_doc_ids: Vec<DocId>,
    /// Precomputed outgoing weight sums per internal node index.
    pub(crate) out_weight_sums: Vec<f32>,

    /// Reverse lookup index mapping DocId to Set of EdgeId (EntityId, EntityId)
    pub(crate) doc_to_edges: ahash::AHashMap<DocId, HashSet<(EntityId, EntityId)>>,

    /// Staging for entities not yet committed, indexed by (TxId, EntityId).
    pub(crate) staged_entities: ahash::AHashMap<(TxId, EntityId), Entity>,
    /// Staging for edges not yet committed, indexed by (TxId, EntityId).
    pub(crate) staged_edges: ahash::AHashMap<(TxId, EntityId), Vec<StagedEdgePayload>>,
    /// Staging for edge removals not yet committed, grouped by TxId.
    pub(crate) staged_removals: ahash::AHashMap<TxId, Vec<(EntityId, EntityId)>>,
    /// Edges that have been committed but not yet compacted into CSR arrays (delta buffer).
    pub(crate) pending_edges: HashMap<InternalIndex, Vec<EdgePayload>>,
    /// Tombstoned edges that have been removed and should be excluded during compaction and traversal.
    pub(crate) tombstoned_edges: HashSet<(InternalIndex, InternalIndex)>,
    /// Total number of uncompacted edges currently in `pending_edges`.
    pub(crate) pending_edge_count: usize,
    /// Flag indicating if there are uncompacted pending edges or modifications.
    pub(crate) is_dirty: bool,
    /// On-demand adjacency list cache for edge reinforcement learning.
    /// INVALIDATION STRATEGY: Cleared completely during `compact()` to prevent stale or removed edges
    /// from being served after pending/tombstone edge compaction.
    #[cfg(feature = "edge-reinforcement-learning")]
    pub(crate) edge_store: HashMap<EntityId, Vec<Edge>>,

    /// Hyperedge storage mapping HyperEdgeId -> HyperEdge.
    pub(crate) hyperedges: HashMap<crate::hyperedge::HyperEdgeId, Arc<crate::hyperedge::HyperEdge>>,
    /// Index mapping DocId -> Set of HyperEdgeIds.
    pub(crate) doc_to_hyperedges: ahash::AHashMap<DocId, HashSet<crate::hyperedge::HyperEdgeId>>,
    /// Index mapping EntityId -> Set of HyperEdgeIds.
    pub(crate) hyperedge_index: ahash::AHashMap<EntityId, HashSet<crate::hyperedge::HyperEdgeId>>,
    /// Reverse index mapping child HyperEdgeId -> Set of parent HyperEdgeIds.
    pub(crate) child_to_parents: ahash::AHashMap<crate::hyperedge::HyperEdgeId, HashSet<crate::hyperedge::HyperEdgeId>>,
}

/// Detailed memory estimate separating shared payloads from private structural allocations (§6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MemoryEstimate {
    /// Memory used by shared payloads (e.g. `Arc<HyperEdge>` / `Arc<[RoleBinding]>`).
    pub shared_payload_bytes: usize,
    /// Memory used by private internal graph structures (table capacities, vectors, indices).
    pub private_bytes: usize,
}

impl MemoryEstimate {
    /// Returns the total memory in bytes (`shared_payload_bytes + private_bytes`).
    #[inline]
    pub fn total_bytes(&self) -> usize {
        self.shared_payload_bytes + self.private_bytes
    }
}

#[inline]
fn table_bytes<K, V>(capacity: usize) -> usize {
    capacity * (std::mem::size_of::<(K, V)>() + 1)
}

#[inline]
fn vec_bytes<T>(capacity: usize) -> usize {
    capacity * std::mem::size_of::<T>()
}

#[inline]
pub(crate) fn sentinel_entity() -> Entity {
    Entity::new(EntityId::new(0), "", "")
}

impl Default for GraphInner {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphInner {
    pub fn new() -> Self {
        Self {
            id_map: HashMap::new(),
            reverse_map: Vec::new(),
            entities: Vec::new(),
            communities: HashMap::new(),
            communities_loaded: false,
            offsets: vec![0],
            targets: Vec::new(),
            weights: Vec::new(),
            tx_valid_froms: Vec::new(),
            tx_valid_tos: Vec::new(),
            business_valid_froms: Vec::new(),
            business_valid_tos: Vec::new(),
            source_doc_ids: Vec::new(),
            out_weight_sums: Vec::new(),
            doc_to_edges: ahash::AHashMap::new(),
            staged_entities: ahash::AHashMap::default(),
            staged_edges: ahash::AHashMap::default(),
            staged_removals: ahash::AHashMap::default(),
            pending_edges: HashMap::new(),
            tombstoned_edges: HashSet::new(),
            pending_edge_count: 0,
            is_dirty: false,
            #[cfg(feature = "edge-reinforcement-learning")]
            edge_store: HashMap::new(),
            hyperedges: HashMap::new(),
            doc_to_hyperedges: ahash::AHashMap::new(),
            hyperedge_index: ahash::AHashMap::new(),
            child_to_parents: ahash::AHashMap::new(),
        }
    }

    #[inline]
    pub(crate) fn entity_at(&self, idx: usize) -> Option<&Entity> {
        self.entities.get(idx).filter(|e| e.id != EntityId::new(0))
    }

    #[inline]
    pub(crate) fn tx_valid_from_at(&self, idx: usize) -> Option<TxId> {
        self.tx_valid_froms
            .get(idx)
            .copied()
            .filter(|&tx| tx != TxId::INVALID)
    }

    #[inline]
    pub(crate) fn tx_valid_to_at(&self, idx: usize) -> Option<TxId> {
        self.tx_valid_tos
            .get(idx)
            .copied()
            .filter(|&tx| tx != TxId::INVALID)
    }

    #[inline]
    pub(crate) fn business_valid_from_at(&self, idx: usize) -> Option<i64> {
        self.business_valid_froms
            .get(idx)
            .copied()
            .filter(|&ts| ts != i64::MIN)
    }

    #[inline]
    pub(crate) fn business_valid_to_at(&self, idx: usize) -> Option<i64> {
        self.business_valid_tos
            .get(idx)
            .copied()
            .filter(|&ts| ts != i64::MIN)
    }

    #[inline]
    pub(crate) fn source_doc_id_at(&self, idx: usize) -> Option<DocId> {
        self.source_doc_ids
            .get(idx)
            .copied()
            .filter(|d| d.inner() != 0)
    }

    pub(crate) fn estimate_memory(&self, presized: bool) -> MemoryEstimate {
        let mut private = 0usize;

        let id_map_cap = if presized {
            self.id_map.len()
        } else {
            self.id_map.capacity()
        };
        private += table_bytes::<EntityId, usize>(id_map_cap);

        let comm_cap = if presized {
            self.communities.len()
        } else {
            self.communities.capacity()
        };
        private += table_bytes::<EntityId, u64>(comm_cap);

        let staged_ent_cap = if presized {
            self.staged_entities.len()
        } else {
            self.staged_entities.capacity()
        };
        private += table_bytes::<(TxId, EntityId), Entity>(staged_ent_cap);

        let staged_edg_cap = if presized {
            self.staged_edges.len()
        } else {
            self.staged_edges.capacity()
        };
        private += table_bytes::<(TxId, EntityId, EntityId), EdgePayload>(staged_edg_cap);

        let staged_rem_cap = if presized {
            self.staged_removals.len()
        } else {
            self.staged_removals.capacity()
        };
        private += table_bytes::<(TxId, EntityId, EntityId), ()>(staged_rem_cap);

        let tombstone_cap = if presized {
            self.tombstoned_edges.len()
        } else {
            self.tombstoned_edges.capacity()
        };
        private += table_bytes::<(EntityId, EntityId), ()>(tombstone_cap);

        let rev_cap = if presized {
            self.reverse_map.len()
        } else {
            self.reverse_map.capacity()
        };
        private += vec_bytes::<EntityId>(rev_cap);

        let ent_cap = if presized {
            self.entities.len()
        } else {
            self.entities.capacity()
        };
        private += vec_bytes::<Entity>(ent_cap);

        let off_cap = if presized {
            self.offsets.len()
        } else {
            self.offsets.capacity()
        };
        private += vec_bytes::<usize>(off_cap);

        let tar_cap = if presized {
            self.targets.len()
        } else {
            self.targets.capacity()
        };
        private += vec_bytes::<usize>(tar_cap);

        let w_cap = if presized {
            self.weights.len()
        } else {
            self.weights.capacity()
        };
        private += vec_bytes::<f32>(w_cap);

        let tvf_cap = if presized {
            self.tx_valid_froms.len()
        } else {
            self.tx_valid_froms.capacity()
        };
        private += vec_bytes::<TxId>(tvf_cap);

        let tvt_cap = if presized {
            self.tx_valid_tos.len()
        } else {
            self.tx_valid_tos.capacity()
        };
        private += vec_bytes::<TxId>(tvt_cap);

        let bvf_cap = if presized {
            self.business_valid_froms.len()
        } else {
            self.business_valid_froms.capacity()
        };
        private += vec_bytes::<i64>(bvf_cap);

        let bvt_cap = if presized {
            self.business_valid_tos.len()
        } else {
            self.business_valid_tos.capacity()
        };
        private += vec_bytes::<i64>(bvt_cap);

        let doc_cap = if presized {
            self.source_doc_ids.len()
        } else {
            self.source_doc_ids.capacity()
        };
        private += vec_bytes::<DocId>(doc_cap);

        let ows_cap = if presized {
            self.out_weight_sums.len()
        } else {
            self.out_weight_sums.capacity()
        };
        private += vec_bytes::<f32>(ows_cap);

        let pending_map_cap = if presized {
            self.pending_edges.len()
        } else {
            self.pending_edges.capacity()
        };
        private += table_bytes::<(EntityId, EntityId), Vec<EdgePayload>>(pending_map_cap);
        for v in self.pending_edges.values() {
            let v_cap = if presized { v.len() } else { v.capacity() };
            private += vec_bytes::<EdgePayload>(v_cap);
        }

        let doc_edges_map_cap = if presized {
            self.doc_to_edges.len()
        } else {
            self.doc_to_edges.capacity()
        };
        private += table_bytes::<DocId, HashSet<(EntityId, EntityId)>>(doc_edges_map_cap);
        for set in self.doc_to_edges.values() {
            let set_cap = if presized { set.len() } else { set.capacity() };
            private += table_bytes::<(EntityId, EntityId), ()>(set_cap);
        }

        let hyperedges_map_cap = if presized {
            self.hyperedges.len()
        } else {
            self.hyperedges.capacity()
        };
        private += table_bytes::<crate::hyperedge::HyperEdgeId, Arc<crate::hyperedge::HyperEdge>>(
            hyperedges_map_cap,
        );

        let doc_hyperedges_map_cap = if presized {
            self.doc_to_hyperedges.len()
        } else {
            self.doc_to_hyperedges.capacity()
        };
        private +=
            table_bytes::<DocId, HashSet<crate::hyperedge::HyperEdgeId>>(doc_hyperedges_map_cap);
        for set in self.doc_to_hyperedges.values() {
            let set_cap = if presized { set.len() } else { set.capacity() };
            private += table_bytes::<crate::hyperedge::HyperEdgeId, ()>(set_cap);
        }

        let child_to_parents_map_cap = if presized {
            self.child_to_parents.len()
        } else {
            self.child_to_parents.capacity()
        };
        private += table_bytes::<
            crate::hyperedge::HyperEdgeId,
            HashSet<crate::hyperedge::HyperEdgeId>,
        >(child_to_parents_map_cap);
        for set in self.child_to_parents.values() {
            let set_cap = if presized { set.len() } else { set.capacity() };
            private += table_bytes::<crate::hyperedge::HyperEdgeId, ()>(set_cap);
        }

        let hyperedge_index_map_cap = if presized {
            self.hyperedge_index.len()
        } else {
            self.hyperedge_index.capacity()
        };
        private += table_bytes::<EntityId, HashSet<crate::hyperedge::HyperEdgeId>>(
            hyperedge_index_map_cap,
        );
        for set in self.hyperedge_index.values() {
            let set_cap = if presized { set.len() } else { set.capacity() };
            private += table_bytes::<crate::hyperedge::HyperEdgeId, ()>(set_cap);
        }

        let shared_payload_bytes = self
            .hyperedges
            .values()
            .map(|e| {
                std::mem::size_of::<crate::hyperedge::HyperEdge>()
                    + (e.participants.len() * std::mem::size_of::<crate::hyperedge::RoleBinding>())
            })
            .sum::<usize>();

        MemoryEstimate {
            shared_payload_bytes,
            private_bytes: private,
        }
    }

    pub(crate) fn estimate_memory_bytes(&self) -> usize {
        self.estimate_memory(false).total_bytes()
    }

    pub(crate) fn estimate_compaction_peak_bytes(&self) -> usize {
        let current = self.estimate_memory(false);
        let rebuild_private = self.estimate_memory(true).private_bytes;
        current.total_bytes() + rebuild_private
    }

    #[expect(
        dead_code,
        reason = "Internal GraphInner hyperedge helper method retained for planned GraphInner API symmetry"
    )]
    pub(crate) fn hyperedges_for_entity(&self, id: EntityId) -> Vec<crate::hyperedge::HyperEdgeId> {
        self.hyperedge_index
            .get(&id)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    #[expect(
        dead_code,
        reason = "Internal GraphInner hyperedge helper method retained for planned GraphInner API symmetry"
    )]
    pub(crate) fn insert_hyperedge(&mut self, edge: crate::hyperedge::HyperEdge) {
        let edge_id = edge.id;
        for participant in edge.participants.iter() {
            self.hyperedge_index
                .entry(participant.entity)
                .or_default()
                .insert(edge_id);
        }
        for &child_id in edge.child_edge_ids.iter() {
            self.child_to_parents
                .entry(child_id)
                .or_default()
                .insert(edge_id);
        }
        self.hyperedges.insert(edge_id, Arc::new(edge));
    }

    pub fn get_or_create_index(&mut self, id: EntityId) -> InternalIndex {
        if let Some(&idx) = self.id_map.get(&id) {
            idx
        } else {
            let idx = self.reverse_map.len();
            self.id_map.insert(id, idx);
            self.reverse_map.push(id);
            self.out_weight_sums.push(0.0);
            // entities vector should be kept in sync by add_entity,
            // but we might add an edge to an entity not yet added via add_entity.
            // In that case, we'll have a "shadow" entity.
            idx
        }
    }

    #[inline]
    pub(crate) fn add_to_out_weight_sum(&mut self, node_idx: usize, weight: f32) {
        if weight > 0.0 {
            let num_nodes = self.reverse_map.len();
            if self.out_weight_sums.len() < num_nodes {
                self.out_weight_sums.resize(num_nodes, 0.0);
            }
            if node_idx < self.out_weight_sums.len() {
                self.out_weight_sums[node_idx] += weight;
            }
        }
    }

    pub(crate) fn recompute_node_out_weight_sum(&mut self, node_idx: usize) {
        let num_nodes = self.reverse_map.len();
        if node_idx >= num_nodes {
            return;
        }
        if self.out_weight_sums.len() < num_nodes {
            self.out_weight_sums.resize(num_nodes, 0.0);
        }
        if self.entity_at(node_idx).is_none() {
            self.out_weight_sums[node_idx] = 0.0;
            return;
        }

        let mut sum = 0.0f32;

        if node_idx < self.offsets.len() - 1 {
            let start = self.offsets[node_idx];
            let end = self.offsets[node_idx + 1];
            for j in start..end {
                let target = self.targets[j];
                let w = self.weights[j];
                if !self.tombstoned_edges.contains(&(node_idx, target))
                    && self.entity_at(target).is_some()
                    && w > 0.0
                {
                    sum += w;
                }
            }
        }

        if let Some(pending) = self.pending_edges.get(&node_idx) {
            for edge in pending {
                let target = edge.target;
                if !self.tombstoned_edges.contains(&(node_idx, target))
                    && self.entity_at(target).is_some()
                    && edge.weight > 0.0
                {
                    sum += edge.weight;
                }
            }
        }

        self.out_weight_sums[node_idx] = sum;
    }

    /// Compacts pending edges in the delta buffer into the main CSR arrays.
    pub fn compact(&mut self) {
        let num_nodes = self.reverse_map.len();

        if self.pending_edges.is_empty() && self.tombstoned_edges.is_empty() {
            while self.offsets.len() < num_nodes + 1 {
                let last = *self.offsets.last().unwrap_or(&0);
                self.offsets.push(last);
            }
            self.pending_edge_count = 0;
            self.is_dirty = false;
            #[cfg(feature = "edge-reinforcement-learning")]
            self.edge_store.clear();
            return;
        }

        let mut new_offsets = Vec::with_capacity(num_nodes + 1);
        let mut new_targets = Vec::with_capacity(self.targets.len() + self.pending_edge_count);
        let mut new_weights = Vec::with_capacity(self.weights.len() + self.pending_edge_count);
        let mut new_tx_valid_froms =
            Vec::with_capacity(self.tx_valid_froms.len() + self.pending_edge_count);
        let mut new_tx_valid_tos =
            Vec::with_capacity(self.tx_valid_tos.len() + self.pending_edge_count);
        let mut new_business_valid_froms =
            Vec::with_capacity(self.business_valid_froms.len() + self.pending_edge_count);
        let mut new_business_valid_tos =
            Vec::with_capacity(self.business_valid_tos.len() + self.pending_edge_count);
        let mut new_source_doc_ids =
            Vec::with_capacity(self.source_doc_ids.len() + self.pending_edge_count);

        let mut current_offset = 0;
        new_offsets.push(current_offset);

        for i in 0..num_nodes {
            let mut node_edges = Vec::new();

            // 1. Get neighbors from old CSR
            let old_start = if i < self.offsets.len() - 1 {
                self.offsets[i]
            } else {
                0
            };
            let old_end = if i < self.offsets.len() - 1 {
                self.offsets[i + 1]
            } else {
                0
            };

            for j in old_start..old_end {
                let target = self.targets[j];
                if !self.tombstoned_edges.contains(&(i, target)) {
                    node_edges.push(EdgePayload {
                        target,
                        weight: self.weights[j],
                        tx_valid_from: self.tx_valid_from_at(j),
                        tx_valid_to: self.tx_valid_to_at(j),
                        business_valid_from: self.business_valid_from_at(j),
                        business_valid_to: self.business_valid_to_at(j),
                        source_doc_id: self.source_doc_id_at(j),
                    });
                }
            }

            // 2. Get neighbors from pending_edges (FIND-GRA-001)
            if let Some(staged) = self.pending_edges.get(&i) {
                for edge in staged {
                    if !self.tombstoned_edges.contains(&(i, edge.target)) {
                        node_edges.push(edge.clone());
                    }
                }
            }

            // Stable sort target indices for deterministic CSR layout
            node_edges.sort_by_key(|e| e.target);

            for edge in node_edges {
                new_targets.push(edge.target);
                new_weights.push(edge.weight);
                new_tx_valid_froms.push(edge.tx_valid_from.unwrap_or(TxId::INVALID));
                new_tx_valid_tos.push(edge.tx_valid_to.unwrap_or(TxId::INVALID));
                new_business_valid_froms.push(edge.business_valid_from.unwrap_or(i64::MIN));
                new_business_valid_tos.push(edge.business_valid_to.unwrap_or(i64::MIN));
                new_source_doc_ids.push(edge.source_doc_id.unwrap_or(DocId::new(0)));
                current_offset += 1;
            }

            new_offsets.push(current_offset);
        }

        while new_offsets.len() < num_nodes + 1 {
            let last = *new_offsets.last().unwrap_or(&0);
            new_offsets.push(last);
        }

        self.offsets = new_offsets;
        self.targets = new_targets;
        self.weights = new_weights;
        self.tx_valid_froms = new_tx_valid_froms;
        self.tx_valid_tos = new_tx_valid_tos;
        self.business_valid_froms = new_business_valid_froms;
        self.business_valid_tos = new_business_valid_tos;
        self.source_doc_ids = new_source_doc_ids;
        self.pending_edges.clear();
        self.tombstoned_edges.clear();
        self.pending_edge_count = 0;
        self.is_dirty = false;
        #[cfg(feature = "edge-reinforcement-learning")]
        self.edge_store.clear();

        // Recompute out_weight_sums for all nodes during compaction
        self.out_weight_sums.resize(num_nodes, 0.0);
        for i in 0..num_nodes {
            if self.entity_at(i).is_none() {
                self.out_weight_sums[i] = 0.0;
                continue;
            }
            let start = self.offsets[i];
            let end = self.offsets[i + 1];
            let mut sum = 0.0f32;
            for j in start..end {
                let target = self.targets[j];
                let w = self.weights[j];
                if self.entity_at(target).is_some() && w > 0.0 {
                    sum += w;
                }
            }
            self.out_weight_sums[i] = sum;
        }
    }
}

#[cfg(feature = "edge-reinforcement-learning")]
impl GraphInner {
    pub(crate) fn find_edge_mut_by_entities(
        &mut self,
        from: EntityId,
        to: EntityId,
    ) -> Option<&mut Edge> {
        let from_idx = *self.id_map.get(&from)?;
        let to_idx = *self.id_map.get(&to)?;

        let exists_pending = self
            .pending_edges
            .get(&from_idx)
            .is_some_and(|edges| edges.iter().any(|e| e.target == to_idx));

        let exists_csr = if from_idx < self.offsets.len() - 1 {
            let start = self.offsets[from_idx];
            let end = self.offsets[from_idx + 1];
            self.targets[start..end].contains(&to_idx)
        } else {
            false
        };

        if !exists_pending && !exists_csr {
            return None;
        }

        let vec = self.edge_store.entry(from).or_default();
        if !vec.iter().any(|e| e.target == to) {
            let current_weight = if let Some(pending) = self.pending_edges.get(&from_idx) {
                pending
                    .iter()
                    .find(|e| e.target == to_idx)
                    .map(|e| e.weight)
            } else {
                None
            }
            .unwrap_or_else(|| {
                if from_idx < self.offsets.len() - 1 {
                    let start = self.offsets[from_idx];
                    let end = self.offsets[from_idx + 1];
                    for j in start..end {
                        if self.targets[j] == to_idx {
                            return self.weights[j];
                        }
                    }
                }
                1.0
            });

            vec.push(Edge::new(to, current_weight));
        }

        self.edge_store
            .get_mut(&from)?
            .iter_mut()
            .find(|e| e.target == to)
    }

    pub(crate) fn all_entity_ids(&self) -> Vec<EntityId> {
        self.reverse_map.clone()
    }

    pub(crate) fn outgoing_edges_mut(&mut self, entity_id: EntityId) -> &mut [Edge] {
        if let Some(from_idx) = self.id_map.get(&entity_id).copied() {
            let mut target_ids = Vec::new();
            if let Some(pending) = self.pending_edges.get(&from_idx) {
                for p in pending {
                    if let Some(&t_id) = self.reverse_map.get(p.target) {
                        target_ids.push((t_id, p.weight));
                    }
                }
            }
            if from_idx < self.offsets.len() - 1 {
                let start = self.offsets[from_idx];
                let end = self.offsets[from_idx + 1];
                for j in start..end {
                    let t_idx = self.targets[j];
                    if let Some(&t_id) = self.reverse_map.get(t_idx) {
                        target_ids.push((t_id, self.weights[j]));
                    }
                }
            }

            let vec = self.edge_store.entry(entity_id).or_default();
            for (t_id, w) in target_ids {
                if !vec.iter().any(|e| e.target == t_id) {
                    vec.push(Edge::new(t_id, w));
                }
            }
        }

        self.edge_store
            .get_mut(&entity_id)
            .map(|v| v.as_mut_slice())
            .unwrap_or(&mut [])
    }

    pub(crate) fn sync_edge_reinforcement_weights(
        &mut self,
        config: &crate::edge_reinforcement::EdgeReinforcementConfig,
    ) {
        use crate::edge_reinforcement::compute_edge_weight;

        for (&from_id, edges) in &self.edge_store {
            let Some(&from_idx) = self.id_map.get(&from_id) else {
                continue;
            };

            for edge in edges {
                let new_weight = compute_edge_weight(
                    edge.cooccurrence_weight,
                    edge.traversal_weight,
                    config.alpha,
                );

                if let Some(&to_idx) = self.id_map.get(&edge.target) {
                    // Update in pending_edges
                    if let Some(pending) = self.pending_edges.get_mut(&from_idx) {
                        for p_edge in pending.iter_mut() {
                            if p_edge.target == to_idx {
                                p_edge.weight = new_weight;
                            }
                        }
                    }

                    // Update in CSR arrays
                    if from_idx < self.offsets.len() - 1 {
                        let start = self.offsets[from_idx];
                        let end = self.offsets[from_idx + 1];
                        for j in start..end {
                            if self.targets[j] == to_idx {
                                self.weights[j] = new_weight;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// RAII guard wrapping the writer mutex guard, automatically publishing updated `Arc<GraphInner>` snapshots to `ArcSwap` on drop.
pub(crate) struct InnerWriteGuard<'a> {
    pub(crate) guard: parking_lot::MutexGuard<'a, GraphInner>,
    pub(crate) arc_swap: &'a ArcSwap<GraphInner>,
}

impl<'a> std::ops::Deref for InnerWriteGuard<'a> {
    type Target = GraphInner;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a> std::ops::DerefMut for InnerWriteGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<'a> Drop for InnerWriteGuard<'a> {
    fn drop(&mut self) {
        self.arc_swap.store(Arc::new((*self.guard).clone()));
    }
}
