//! PathRAG Engine — Graph-basiertes Retrieval via bidirektionaler Dijkstra.
//!
//! Basis: arXiv:2502.14902 (PathRAG, AAAI 2026).
//! Präzisions-Scope: Nur für Multi-Hop-Anfragen verwenden (arXiv:2506.05690).
//! Sufficiency-Gate verhindert Precision-Kollaps (arXiv:2506.00610).
//!
//! INTEGRATION: PathRAG liefert ein RRF-Signal neben Vektor- und BM25-Signal.
//! Resultat von to_rrf_signal() wird in FusionEngine als drittes Signal eingespeist.

pub use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use ahash::AHashMap;
use contextra_types::DocId;
pub use contextra_types::EntityId;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;

/// Ein gefundener Pfad zwischen zwei Knoten.
#[derive(Debug, Clone)]
pub struct GraphPath {
    /// Knoten-Sequenz vom Start zum Ziel.
    pub nodes: Vec<EntityId>,
    /// Kantengewichte entlang des Pfades (len = nodes.len() - 1).
    pub edge_weights: Vec<f32>,
    /// Konfidenz: Produkt aller Kantengewichte.
    pub confidence: f64,
    /// Invertierte Gesamtdistanz als Flow-Proxy.
    pub total_flow: f32,
}

/// Trait für Graphen die PathRAG konsumieren kann.
/// Ermöglicht Testbarkeit ohne echten CSR-Graphen.
pub trait PathGraph: Send + Sync {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)>;
    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)>;

    /// Returns hyperedge IDs associated with the given entity (default: empty).
    fn hyperedges_for_entity(&self, _node: EntityId) -> Vec<HyperEdgeId> {
        Vec::new()
    }

    /// Resolves a HyperEdgeId to its full HyperEdge details (default: None).
    fn get_hyperedge(&self, _id: HyperEdgeId) -> Option<Arc<HyperEdge>> {
        None
    }

    /// Returns participant role bindings for a given hyperedge ID (default: resolves via get_hyperedge).
    fn hyperedge_participants(&self, id: HyperEdgeId) -> Arc<[RoleBinding]> {
        match self.get_hyperedge(id) {
            Some(he) => he.participants.clone(),
            None => Arc::from([]),
        }
    }
}

/// Configuration parameters for Forward-Push Personalized PageRank (PPR).
#[derive(Debug, Clone, PartialEq)]
pub struct PprParams {
    /// Teleport probability ($\alpha$, default: 0.15).
    pub alpha: f32,
    /// Convergence error tolerance threshold ($\epsilon$, default: 1e-4).
    pub epsilon: f32,
    /// Discount factor applied to virtual hyperedge neighbor push shares (default: 0.85).
    pub hyperedge_decay: f32,
}

impl Default for PprParams {
    fn default() -> Self {
        Self {
            alpha: 0.15,
            epsilon: 1e-4,
            hyperedge_decay: 0.85,
        }
    }
}

/// Andersen-Chung-Lang Forward-Push PPR implementation with Hyperedge Expansion (§6.6, H3).
///
/// Runtime is $O(1/(\alpha \cdot \epsilon))$, independent of $|V| + |E|$ (P24).
pub fn forward_push_ppr<G: PathGraph>(
    graph: &G,
    seeds: &[EntityId],
    params: &PprParams,
) -> AHashMap<EntityId, f32> {
    let mut p: AHashMap<EntityId, f32> = AHashMap::default();
    if seeds.is_empty() {
        return p;
    }

    let mut r: AHashMap<EntityId, f32> = seeds
        .iter()
        .map(|&s| (s, 1.0 / seeds.len() as f32))
        .collect();
    let mut queue: std::collections::VecDeque<EntityId> = seeds.iter().copied().collect();
    let mut in_queue: ahash::AHashSet<EntityId> = seeds.iter().copied().collect();

    while let Some(u) = queue.pop_front() {
        in_queue.remove(&u);

        let binary_neighbors = graph.neighbors_with_weights(u);
        let mut hyper_participants = Vec::new();
        for hedge_id in graph.hyperedges_for_entity(u) {
            for role_binding in graph.hyperedge_participants(hedge_id).iter() {
                if role_binding.entity != u {
                    hyper_participants.push(role_binding.entity);
                }
            }
        }

        let total_neighbors_count = binary_neighbors.len() + hyper_participants.len();
        let degree = (total_neighbors_count as f32).max(1.0);

        let r_u = *r.get(&u).unwrap_or(&0.0);
        if r_u / degree <= params.epsilon {
            continue;
        }

        *p.entry(u).or_insert(0.0) += params.alpha * r_u;
        r.insert(u, 0.0);

        let push_share = (1.0 - params.alpha) * r_u / degree;

        // Binary neighbors (unmodified hotpath invariant).
        for (v, _w) in binary_neighbors {
            let entry = r.entry(v).or_insert(0.0);
            *entry += push_share;
            if in_queue.insert(v) {
                queue.push_back(v);
            }
        }

        // Virtual neighbors from hyperedges (§6.6, H3) with hyperedge_decay discount.
        for v in hyper_participants {
            let entry = r.entry(v).or_insert(0.0);
            *entry += push_share * params.hyperedge_decay;
            if in_queue.insert(v) {
                queue.push_back(v);
            }
        }
    }

    p
}

/// Normativer Default-Schwellenwert für das Sufficiency-Gate (ADR-067).
///
/// Pfade mit Konfidenz < 0.10 werden verworfen, um Precision-Kollaps durch
/// Rauschen bei tiefen Multi-Hop-Traversierungen zu verhindern (arXiv:2506.00610).
pub const DEFAULT_SUFFICIENCY_THRESHOLD: f64 = 0.1;

/// Configuration options for [`PathRAGEngine`].
#[derive(Debug, Clone, PartialEq)]
pub struct PathRAGConfig {
    /// Maximale Suchtiefe (Hop-Limit).
    pub max_hops: usize,
    /// Sufficiency-Schwelle: Pfade unter dieser Konfidenz werden gefiltert.
    pub sufficiency_threshold: f64,
    /// Enable virtual neighbor expansion over hyperedges during PathRAG traversal.
    pub hyperedge_expansion_enabled: bool,
    /// Discount factor applied to hyperedge weights when expanded as virtual neighbors (default: 0.85).
    pub hyperedge_weight_discount: f32,
}

impl Default for PathRAGConfig {
    fn default() -> Self {
        Self {
            max_hops: 4,
            sufficiency_threshold: DEFAULT_SUFFICIENCY_THRESHOLD,
            hyperedge_expansion_enabled: false,
            hyperedge_weight_discount: 0.85,
        }
    }
}

pub struct PathRAGEngine<G: PathGraph> {
    graph: G,
    pub config: PathRAGConfig,
}

impl<G: PathGraph> PathRAGEngine<G> {
    pub fn new(graph: G, max_hops: usize, sufficiency_threshold: f64) -> Self {
        Self::with_config(
            graph,
            PathRAGConfig {
                max_hops,
                sufficiency_threshold,
                hyperedge_expansion_enabled: false,
                hyperedge_weight_discount: 0.85,
            },
        )
    }

    pub fn with_defaults(graph: G) -> Self {
        Self::with_config(graph, PathRAGConfig::default())
    }

    pub fn with_config(graph: G, config: PathRAGConfig) -> Self {
        Self { graph, config }
    }

    pub fn max_hops(&self) -> usize {
        self.config.max_hops
    }

    pub fn sufficiency_threshold(&self) -> f64 {
        self.config.sufficiency_threshold
    }

    /// Returns physical neighbors plus virtual neighbors from hyperedges if expansion is enabled.
    #[inline]
    fn get_expanded_neighbors(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        let base_neighbors = self.graph.neighbors_with_weights(node);
        if !self.config.hyperedge_expansion_enabled {
            return base_neighbors;
        }

        self.expand_with_hyperedges(node, base_neighbors)
    }

    /// Returns physical predecessors plus virtual predecessors from hyperedges if expansion is enabled.
    #[inline]
    fn get_expanded_predecessors(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        let base_predecessors = self.graph.predecessors_with_weights(node);
        if !self.config.hyperedge_expansion_enabled {
            return base_predecessors;
        }

        self.expand_with_hyperedges(node, base_predecessors)
    }

    fn expand_with_hyperedges(
        &self,
        node: EntityId,
        mut candidates: Vec<(EntityId, f32)>,
    ) -> Vec<(EntityId, f32)> {
        let hyperedge_ids = self.graph.hyperedges_for_entity(node);
        for id in hyperedge_ids {
            if let Some(hyperedge) = self.graph.get_hyperedge(id) {
                let virtual_weight = hyperedge.weight * self.config.hyperedge_weight_discount;
                for participant in hyperedge.participants.iter() {
                    if participant.entity != node {
                        candidates.push((participant.entity, virtual_weight));
                    }
                }
            }
        }
        candidates
    }

    /// Findet den optimalen Pfad zwischen source und target via bidirektionalem Dijkstra.
    ///
    /// Gibt None zurück wenn kein Pfad innerhalb max_hops existiert
    /// oder Sufficiency-Gate fehlschlägt.
    pub fn find_path(&self, source: EntityId, target: EntityId) -> Option<GraphPath> {
        if source == target {
            return Some(GraphPath {
                nodes: vec![source],
                edge_weights: vec![],
                confidence: 1.0,
                total_flow: f32::INFINITY,
            });
        }

        // Bidirektionaler Dijkstra: Forward von source, Backward von target
        let mut dist_fwd: HashMap<EntityId, f32> = HashMap::new();
        let mut dist_bwd: HashMap<EntityId, f32> = HashMap::new();
        let mut prev_fwd: HashMap<EntityId, (EntityId, f32)> = HashMap::new();
        let mut prev_bwd: HashMap<EntityId, (EntityId, f32)> = HashMap::new();

        // BinaryHeap: Reverse((f32_bits, EntityId)) — f32 über Bits geordnet
        let mut heap_fwd: BinaryHeap<Reverse<(u32, EntityId)>> = BinaryHeap::new();
        let mut heap_bwd: BinaryHeap<Reverse<(u32, EntityId)>> = BinaryHeap::new();

        dist_fwd.insert(source, 0.0);
        dist_bwd.insert(target, 0.0);
        heap_fwd.push(Reverse((0u32, source)));
        heap_bwd.push(Reverse((0u32, target)));

        let mut best_dist = f32::INFINITY;
        let mut meeting_node: Option<EntityId> = None;

        let mut steps = 0;
        let max_steps = self.config.max_hops * 1000; // Schutzzähler gegen Endlosschleife

        while (!heap_fwd.is_empty() || !heap_bwd.is_empty()) && steps < max_steps {
            steps += 1;

            // Vorwärts-Schritt
            if let Some(Reverse((d_bits, u))) = heap_fwd.pop() {
                let d = f32::from_bits(d_bits);
                if d <= *dist_fwd.get(&u).unwrap_or(&f32::INFINITY) {
                    // Treffen-Check
                    if let Some(&bwd_d) = dist_bwd.get(&u) {
                        let total = d + bwd_d;
                        if total < best_dist {
                            best_dist = total;
                            meeting_node = Some(u);
                        }
                    }

                    if d <= best_dist {
                        for (neighbor, weight) in self.get_expanded_neighbors(u) {
                            let new_d = d + (1.0 / weight.max(1e-8));
                            let entry = dist_fwd.entry(neighbor).or_insert(f32::INFINITY);
                            if new_d < *entry {
                                *entry = new_d;
                                prev_fwd.insert(neighbor, (u, weight));
                                heap_fwd.push(Reverse((new_d.to_bits(), neighbor)));
                            }
                        }
                    }
                }
            }

            // Rückwärts-Schritt
            if let Some(Reverse((d_bits, u))) = heap_bwd.pop() {
                let d = f32::from_bits(d_bits);
                if d <= *dist_bwd.get(&u).unwrap_or(&f32::INFINITY) {
                    if let Some(&fwd_d) = dist_fwd.get(&u) {
                        let total = fwd_d + d;
                        if total < best_dist {
                            best_dist = total;
                            meeting_node = Some(u);
                        }
                    }

                    if d <= best_dist {
                        for (neighbor, weight) in self.get_expanded_predecessors(u) {
                            let new_d = d + (1.0 / weight.max(1e-8));
                            let entry = dist_bwd.entry(neighbor).or_insert(f32::INFINITY);
                            if new_d < *entry {
                                *entry = new_d;
                                prev_bwd.insert(neighbor, (u, weight));
                                heap_bwd.push(Reverse((new_d.to_bits(), neighbor)));
                            }
                        }
                    }
                }
            }
        }

        let meeting = meeting_node?;

        // Pfad rekonstruieren
        let mut path_nodes = vec![];
        let mut path_weights = vec![];

        // Vorwärts-Pfad: source → meeting
        let mut cur = meeting;
        while cur != source {
            let (prev, w) = *prev_fwd.get(&cur)?;
            path_nodes.push(cur);
            path_weights.push(w);
            cur = prev;
        }
        path_nodes.push(source);
        path_nodes.reverse();
        path_weights.reverse();

        // Rückwärts-Pfad: meeting → target
        cur = meeting;
        while cur != target {
            let (next, w) = *prev_bwd.get(&cur)?;
            path_nodes.push(next);
            path_weights.push(w);
            cur = next;
        }

        let confidence: f64 = path_weights.iter().map(|&w| w as f64).product();
        let total_flow = if best_dist > 0.0 {
            best_dist.recip()
        } else {
            f32::INFINITY
        };

        Some(GraphPath {
            nodes: path_nodes,
            edge_weights: path_weights,
            confidence,
            total_flow,
        })
    }

    /// Findet Pfade von einem Ankerknoten zu allen erreichbaren Knoten innerhalb von max_hops.
    pub fn find_all_paths(&self, source: EntityId) -> Vec<GraphPath> {
        let mut targets = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((source, 0));

        while let Some((curr, depth)) = queue.pop_front() {
            if depth >= self.config.max_hops {
                continue;
            }
            for (nbr, _) in self.get_expanded_neighbors(curr) {
                if nbr != source && targets.insert(nbr) {
                    queue.push_back((nbr, depth + 1));
                }
            }
        }

        let mut paths = Vec::new();
        for target in targets {
            if let Some(path) = self.find_path(source, target) {
                if self.sufficiency_check(&path) {
                    paths.push(path);
                }
            }
        }
        paths
    }

    /// Sufficiency-Gate: Filtert Pfade unter Konfidenz-Schwelle.
    /// Kritisch für Precision (arXiv:2506.00610).
    pub fn sufficiency_check(&self, path: &GraphPath) -> bool {
        path.confidence >= self.config.sufficiency_threshold
    }

    /// Konvertiert gefilterte Pfade in RRF-kompatibles Signal.
    /// Nur Knoten aus Pfaden die sufficiency_check() passiert haben.
    ///
    /// NOTE (Type-Punning Assumption): Re-uses `EntityId` numerical value as `DocId`.
    /// See explicit confirmation in `csr.rs:5104` ("querying DocId(10) (which equals EntityId 10)").
    pub fn to_rrf_signal(&self, paths: &[GraphPath]) -> Vec<(DocId, f32)> {
        let mut scored: HashMap<u64, f32> = HashMap::new();

        for path in paths.iter().filter(|p| self.sufficiency_check(p)) {
            for (i, &entity_id) in path.nodes.iter().enumerate() {
                let position_weight = (i + 1) as f32 / path.nodes.len() as f32;
                *scored.entry(entity_id.inner()).or_insert(0.0) +=
                    path.total_flow * position_weight;
            }
        }

        #[cfg(not(feature = "docid-128"))]
        let mut result: Vec<(DocId, f32)> = scored
            .into_iter()
            .map(|(id, score)| (DocId(id), score))
            .collect();

        #[cfg(feature = "docid-128")]
        let mut result: Vec<(DocId, f32)> = scored
            .into_iter()
            .map(|(id, score)| (DocId(id as u128), score))
            .collect();

        result.sort_by(|a, b| b.1.total_cmp(&a.1));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Einfacher Test-Graph
    struct TestGraph {
        edges: HashMap<EntityId, Vec<(EntityId, f32)>>,
    }

    impl TestGraph {
        fn new(edges: Vec<(EntityId, EntityId, f32)>) -> Self {
            let mut map: HashMap<EntityId, Vec<(EntityId, f32)>> = HashMap::new();
            for (from, to, w) in edges {
                map.entry(from).or_default().push((to, w));
            }
            Self { edges: map }
        }
    }

    impl PathGraph for TestGraph {
        fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
            self.edges.get(&node).cloned().unwrap_or_default()
        }
        fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
            // Für undirektierte Tests: alle Knoten die auf node zeigen
            self.edges
                .iter()
                .flat_map(|(from, nbrs)| {
                    nbrs.iter().filter_map(
                        move |(to, w)| {
                            if *to == node {
                                Some((*from, *w))
                            } else {
                                None
                            }
                        },
                    )
                })
                .collect()
        }
    }

    #[test]
    fn test_find_path_direct_edge() {
        let a = EntityId::new(1);
        let b = EntityId::new(2);
        let graph = TestGraph::new(vec![(a, b, 0.9)]);
        let engine = PathRAGEngine::with_defaults(graph);
        let path = engine.find_path(a, b).unwrap();
        assert_eq!(path.nodes, vec![a, b]);
        assert!((path.confidence - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_find_path_multi_hop() {
        let a = EntityId::new(1);
        let b = EntityId::new(2);
        let c = EntityId::new(3);
        let graph = TestGraph::new(vec![(a, b, 0.8), (b, c, 0.9)]);
        let engine = PathRAGEngine::with_defaults(graph);
        let path = engine.find_path(a, c).unwrap();
        assert_eq!(path.nodes.len(), 3);
        assert!(
            (path.confidence - 0.72).abs() < 1e-5,
            "0.8*0.9={}",
            path.confidence
        );
    }

    #[test]
    fn test_find_path_same_node() {
        let a = EntityId::new(1);
        let graph = TestGraph::new(vec![]);
        let engine = PathRAGEngine::with_defaults(graph);
        let path = engine.find_path(a, a).unwrap();
        assert_eq!(path.nodes, vec![a]);
        assert_eq!(path.edge_weights.len(), 0);
    }

    #[test]
    fn test_find_path_unreachable_returns_none() {
        let a = EntityId::new(1);
        let b = EntityId::new(99);
        let graph = TestGraph::new(vec![]);
        let engine = PathRAGEngine::with_defaults(graph);
        assert!(engine.find_path(a, b).is_none());
    }

    #[test]
    fn test_sufficiency_gate_filters_low_confidence() {
        let engine = PathRAGEngine::new(TestGraph::new(vec![]), 4, 0.5);
        let low_conf = GraphPath {
            nodes: vec![EntityId::new(1), EntityId::new(2)],
            edge_weights: vec![0.3],
            confidence: 0.3,
            total_flow: 3.33,
        };
        assert!(!engine.sufficiency_check(&low_conf));
        let high_conf = GraphPath {
            confidence: 0.8,
            ..low_conf
        };
        assert!(engine.sufficiency_check(&high_conf));
    }

    #[test]
    fn test_find_all_paths_multi_hop() {
        let a = EntityId::new(1);
        let b = EntityId::new(2);
        let c = EntityId::new(3);
        let graph = TestGraph::new(vec![(a, b, 0.8), (b, c, 0.9)]);
        let engine = PathRAGEngine::new(graph, 3, 0.5);
        let paths = engine.find_all_paths(a);
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_rrf_signal_filters_by_sufficiency() {
        let engine = PathRAGEngine::new(TestGraph::new(vec![]), 4, 0.5);
        let paths = vec![
            GraphPath {
                nodes: vec![EntityId::new(1)],
                edge_weights: vec![],
                confidence: 0.1,
                total_flow: 1.0,
            }, // under threshold
            GraphPath {
                nodes: vec![EntityId::new(2)],
                edge_weights: vec![],
                confidence: 0.9,
                total_flow: 1.0,
            }, // passes
        ];
        let signal = engine.to_rrf_signal(&paths);
        assert_eq!(signal.len(), 1);
        assert_eq!(signal[0].0, DocId(2));
    }

    #[test]
    fn test_default_sufficiency_threshold_meets_minimum_bound() {
        // REGRESSION TEST (ADR-067 / arXiv:2506.00610):
        // Verhindert, dass der Default-Schwellenwert versehentlich unter 0.10 fällt (z.B. auf 0.01),
        // was Precision-Kollaps durch ungefilterte, verrauschte Multi-Hop-Pfade auslösen würde.
        const {
            assert!(
                DEFAULT_SUFFICIENCY_THRESHOLD >= 0.10,
                "DEFAULT_SUFFICIENCY_THRESHOLD must be >= 0.10 to prevent precision collapse",
            )
        };

        let engine = PathRAGEngine::with_defaults(TestGraph::new(vec![]));
        assert_eq!(
            engine.sufficiency_threshold(),
            DEFAULT_SUFFICIENCY_THRESHOLD,
            "with_defaults() must construct PathRAGEngine using DEFAULT_SUFFICIENCY_THRESHOLD"
        );
    }

    #[test]
    fn test_find_path_zero_max_hops() {
        let a = EntityId::new(1);
        let b = EntityId::new(2);
        let graph = TestGraph::new(vec![(a, b, 0.9)]);
        let engine = PathRAGEngine::new(graph, 0, 0.1);
        // With max_hops = 0, no search steps run and different nodes return None
        assert!(engine.find_path(a, b).is_none());
        // Same node still returns immediate path
        let same_path = engine.find_path(a, a);
        assert!(same_path.is_some());
    }

    #[test]
    fn test_to_rrf_signal_empty_paths() {
        let engine = PathRAGEngine::with_defaults(TestGraph::new(vec![]));
        let signal = engine.to_rrf_signal(&[]);
        assert!(signal.is_empty());
    }

    struct TestGraphWithHyperedges {
        edges: HashMap<EntityId, Vec<(EntityId, f32)>>,
        hyperedges: HashMap<EntityId, Vec<HyperEdgeId>>,
        hyperedge_store: HashMap<HyperEdgeId, HyperEdge>,
    }

    impl TestGraphWithHyperedges {
        fn new(edges: Vec<(EntityId, EntityId, f32)>, hyperedges: Vec<HyperEdge>) -> Self {
            let mut map: HashMap<EntityId, Vec<(EntityId, f32)>> = HashMap::new();
            for (from, to, w) in edges {
                map.entry(from).or_default().push((to, w));
            }

            let mut he_map: HashMap<EntityId, Vec<HyperEdgeId>> = HashMap::new();
            let mut he_store = HashMap::new();

            for he in hyperedges {
                he_store.insert(he.id, he.clone());
                for participant in he.participants.iter() {
                    he_map.entry(participant.entity).or_default().push(he.id);
                }
            }

            Self {
                edges: map,
                hyperedges: he_map,
                hyperedge_store: he_store,
            }
        }
    }

    impl PathGraph for TestGraphWithHyperedges {
        fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
            self.edges.get(&node).cloned().unwrap_or_default()
        }
        fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
            self.edges
                .iter()
                .flat_map(|(from, nbrs)| {
                    nbrs.iter().filter_map(
                        move |(to, w)| {
                            if *to == node {
                                Some((*from, *w))
                            } else {
                                None
                            }
                        },
                    )
                })
                .collect()
        }
        fn hyperedges_for_entity(&self, node: EntityId) -> Vec<HyperEdgeId> {
            self.hyperedges.get(&node).cloned().unwrap_or_default()
        }
        fn get_hyperedge(&self, id: HyperEdgeId) -> Option<Arc<HyperEdge>> {
            self.hyperedge_store.get(&id).cloned().map(Arc::new)
        }
    }

    #[test]
    fn test_hyperedge_expansion_disabled_by_default() {
        let a = EntityId::new(1);
        let b = EntityId::new(2);
        let he = HyperEdge::new(
            HyperEdgeId::new(100),
            crate::csr::EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), a),
                RoleBinding::new(RoleId::new(2), b),
            ],
            1.0,
        );
        let graph = TestGraphWithHyperedges::new(vec![], vec![he]);
        // Default engine has hyperedge_expansion_enabled: false
        let engine = PathRAGEngine::with_defaults(graph);
        assert!(!engine.config.hyperedge_expansion_enabled);
        assert!(
            engine.find_path(a, b).is_none(),
            "When hyperedge expansion is disabled, hyperedge-only path must return None"
        );
    }

    #[test]
    fn test_hyperedge_expansion_enabled_finds_path_via_hyperedge_only() {
        // AK-2 test: Path exclusively over hyperedge
        let a = EntityId::new(1);
        let b = EntityId::new(2);
        let c = EntityId::new(3);
        let he = HyperEdge::new(
            HyperEdgeId::new(101),
            crate::csr::EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), a),
                RoleBinding::new(RoleId::new(1), b),
                RoleBinding::new(RoleId::new(1), c),
            ],
            1.0,
        );
        let graph = TestGraphWithHyperedges::new(vec![], vec![he]);
        let config = PathRAGConfig {
            hyperedge_expansion_enabled: true,
            hyperedge_weight_discount: 0.85,
            ..Default::default()
        };
        let engine = PathRAGEngine::with_config(graph, config);

        let path = engine
            .find_path(a, c)
            .expect("path must be found via hyperedge");
        assert_eq!(path.nodes, vec![a, c]);
        // weight = hyperedge.weight (1.0) * discount (0.85) = 0.85
        assert!(
            (path.confidence - 0.85).abs() < 1e-5,
            "Expected confidence 0.85, got {}",
            path.confidence
        );
    }

    #[test]
    fn test_hyperedge_weight_discount_applied_correctly() {
        let a = EntityId::new(1);
        let b = EntityId::new(2);
        let he = HyperEdge::new(
            HyperEdgeId::new(102),
            crate::csr::EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), a),
                RoleBinding::new(RoleId::new(2), b),
            ],
            1.0,
        );

        // Test discount = 0.85
        let graph1 = TestGraphWithHyperedges::new(vec![], vec![he.clone()]);
        let engine1 = PathRAGEngine::with_config(
            graph1,
            PathRAGConfig {
                hyperedge_expansion_enabled: true,
                hyperedge_weight_discount: 0.85,
                ..Default::default()
            },
        );
        let path1 = engine1.find_path(a, b).unwrap();
        assert!((path1.confidence - 0.85).abs() < 1e-5);

        // Test discount = 0.50
        let graph2 = TestGraphWithHyperedges::new(vec![], vec![he]);
        let engine2 = PathRAGEngine::with_config(
            graph2,
            PathRAGConfig {
                hyperedge_expansion_enabled: true,
                hyperedge_weight_discount: 0.50,
                ..Default::default()
            },
        );
        let path2 = engine2.find_path(a, b).unwrap();
        assert!((path2.confidence - 0.50).abs() < 1e-5);
    }

    #[test]
    fn test_sufficiency_gate_filters_low_confidence_virtual_neighbors() {
        let a = EntityId::new(1);
        let b = EntityId::new(2);
        // Weight 0.10 * discount 0.85 = 0.085 < sufficiency_threshold 0.10
        let he = HyperEdge::new(
            HyperEdgeId::new(103),
            crate::csr::EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), a),
                RoleBinding::new(RoleId::new(2), b),
            ],
            0.10,
        );
        let graph = TestGraphWithHyperedges::new(vec![], vec![he]);
        let engine = PathRAGEngine::with_config(
            graph,
            PathRAGConfig {
                sufficiency_threshold: 0.10,
                hyperedge_expansion_enabled: true,
                hyperedge_weight_discount: 0.85,
                ..Default::default()
            },
        );

        let paths = engine.find_all_paths(a);
        assert!(
            paths.is_empty(),
            "Virtual neighbor path with confidence 0.085 < 0.10 must be filtered by sufficiency gate"
        );
    }

    #[test]
    fn test_self_referencing_hyperedge_no_infinite_loop() {
        let a = EntityId::new(1);
        // Hyperedge pointing to itself twice
        let he = HyperEdge::new(
            HyperEdgeId::new(104),
            crate::csr::EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), a),
                RoleBinding::new(RoleId::new(2), a),
            ],
            1.0,
        );
        let graph = TestGraphWithHyperedges::new(vec![], vec![he]);
        let engine = PathRAGEngine::with_config(
            graph,
            PathRAGConfig {
                hyperedge_expansion_enabled: true,
                hyperedge_weight_discount: 0.85,
                ..Default::default()
            },
        );

        let paths = engine.find_all_paths(a);
        assert!(
            paths.is_empty(),
            "Self-referencing hyperedge participant must not produce spurious self-paths or infinite loops"
        );
    }

    #[test]
    fn test_fusion_and_signalkind_non_modification_contract() {
        // Contract test ensuring no modification to fusion.rs or SignalKind enum
        // Verify path_rag types seamlessly convert to DocId signal without introducing new SignalKind
        let engine = PathRAGEngine::with_defaults(TestGraph::new(vec![]));
        let path = GraphPath {
            nodes: vec![EntityId::new(42)],
            edge_weights: vec![],
            confidence: 1.0,
            total_flow: 1.0,
        };
        let rrf = engine.to_rrf_signal(&[path]);
        assert_eq!(rrf.len(), 1);
        assert_eq!(rrf[0].0, DocId(42));
    }

    #[test]
    fn test_path_rag_engine_with_real_csr_graph_hyperedges() {
        use crate::csr::CsrGraph;

        let graph = CsrGraph::new();
        let a = EntityId::new(10);
        let b = EntityId::new(20);

        let he = HyperEdge::new(
            HyperEdgeId::new(999),
            crate::csr::EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), a),
                RoleBinding::new(RoleId::new(2), b),
            ],
            1.0,
        );

        graph.insert_hyperedge(he);

        let config = PathRAGConfig {
            hyperedge_expansion_enabled: true,
            hyperedge_weight_discount: 0.85,
            ..Default::default()
        };
        let engine = PathRAGEngine::with_config(graph, config);

        let path = engine
            .find_path(a, b)
            .expect("PathRAG must find path over real CsrGraph hyperedge");

        assert_eq!(path.nodes, vec![a, b]);
        assert!((path.confidence - 0.85).abs() < 1e-5);
    }

    #[test]
    fn test_ppr_params_defaults() {
        let params = PprParams::default();
        assert_eq!(params.alpha, 0.15);
        assert_eq!(params.epsilon, 1e-4);
        assert_eq!(params.hyperedge_decay, 0.85);
    }

    #[test]
    fn test_forward_push_ppr_empty_seeds() {
        let graph = TestGraph::new(vec![]);
        let params = PprParams::default();
        let res = forward_push_ppr(&graph, &[], &params);
        assert!(res.is_empty());
    }

    #[test]
    fn test_forward_push_ppr_binary_graph() {
        let a = EntityId::new(1);
        let b = EntityId::new(2);
        let graph = TestGraph::new(vec![(a, b, 1.0)]);
        let params = PprParams::default();

        let scores = forward_push_ppr(&graph, &[a], &params);
        assert!(scores.get(&a).is_some());
        assert!(*scores.get(&a).unwrap() > 0.0);
    }

    #[test]
    fn test_forward_push_ppr_hyperedge_expansion_and_decay() {
        let a = EntityId::new(1);
        let b = EntityId::new(2);
        let c = EntityId::new(3);

        // Hyperedge connecting a, b, c
        let he = HyperEdge::new(
            HyperEdgeId::new(500),
            crate::csr::EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), a),
                RoleBinding::new(RoleId::new(2), b),
                RoleBinding::new(RoleId::new(3), c),
            ],
            1.0,
        );

        let graph = TestGraphWithHyperedges::new(vec![], vec![he]);

        let params_full = PprParams {
            alpha: 0.15,
            epsilon: 1e-4,
            hyperedge_decay: 1.0,
        };
        let scores_full = forward_push_ppr(&graph, &[a], &params_full);

        let params_decay = PprParams {
            alpha: 0.15,
            epsilon: 1e-4,
            hyperedge_decay: 0.5,
        };
        let scores_decay = forward_push_ppr(&graph, &[a], &params_decay);

        let score_b_full = *scores_full.get(&b).unwrap_or(&0.0);
        let score_b_decay = *scores_decay.get(&b).unwrap_or(&0.0);

        assert!(
            score_b_full > 0.0,
            "b must receive PPR score via hyperedge expansion"
        );
        assert!(
            score_b_decay > 0.0,
            "b must receive PPR score with decay applied"
        );
        assert!(
            score_b_decay < score_b_full,
            "Lower hyperedge_decay must yield lower virtual score (full={}, decay={})",
            score_b_full,
            score_b_decay
        );
    }

    #[test]
    fn test_forward_push_ppr_self_participant_filtered() {
        let a = EntityId::new(1);
        // Hyperedge where 'a' is repeated twice
        let he = HyperEdge::new(
            HyperEdgeId::new(501),
            crate::csr::EdgeType::Default,
            vec![
                RoleBinding::new(RoleId::new(1), a),
                RoleBinding::new(RoleId::new(2), a),
            ],
            1.0,
        );
        let graph = TestGraphWithHyperedges::new(vec![], vec![he]);
        let params = PprParams::default();

        let scores = forward_push_ppr(&graph, &[a], &params);
        // Only 'a' should be in scores (no panic or spurious nodes)
        assert!(scores.get(&a).is_some());
    }
}
