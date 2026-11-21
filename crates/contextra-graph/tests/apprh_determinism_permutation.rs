//! Integration tests verifying bit-identical APPRH determinism across permutations.

#![cfg(feature = "apprh-diffusion")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_graph::apprh::{forward_push_apprh, ApprhParams};
use contextra_graph::path_rag::{PathGraph, PprParams};
use contextra_graph::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_types::EntityId;
use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct PermutedGraph {
    binary_edges: Vec<(EntityId, EntityId, f32)>,
    hyperedges: Vec<(u64, Vec<EntityId>, f32)>,
}

impl PermutedGraph {
    fn build_graph(&self) -> ConcreteGraph {
        let mut cg = ConcreteGraph::new();
        for &(u, v, w) in &self.binary_edges {
            cg.add_binary_edge(u, v, w);
        }
        for (id, participants, w) in &self.hyperedges {
            cg.add_hyperedge(*id, participants.clone(), *w);
        }
        cg
    }
}

struct ConcreteGraph {
    binary_adj: HashMap<EntityId, Vec<(EntityId, f32)>>,
    hyperedges: HashMap<HyperEdgeId, Arc<HyperEdge>>,
    node_to_hyperedges: HashMap<EntityId, Vec<HyperEdgeId>>,
}

impl ConcreteGraph {
    fn new() -> Self {
        Self {
            binary_adj: HashMap::new(),
            hyperedges: HashMap::new(),
            node_to_hyperedges: HashMap::new(),
        }
    }

    fn add_binary_edge(&mut self, u: EntityId, v: EntityId, w: f32) {
        self.binary_adj.entry(u).or_default().push((v, w));
        self.binary_adj.entry(v).or_default().push((u, w));
    }

    fn add_hyperedge(&mut self, id: u64, participants: Vec<EntityId>, w: f32) {
        let hid = HyperEdgeId::new(id);
        let roles: Vec<RoleBinding> = participants
            .into_iter()
            .enumerate()
            .map(|(idx, entity)| RoleBinding::new(RoleId::new(idx as u32 + 1), entity))
            .collect();

        let he = Arc::new(HyperEdge::new(
            hid,
            contextra_graph::csr::EdgeType::Default,
            roles.clone(),
            w,
        ));

        self.hyperedges.insert(hid, he);
        for r in roles {
            self.node_to_hyperedges
                .entry(r.entity)
                .or_default()
                .push(hid);
        }
    }
}

impl PathGraph for ConcreteGraph {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.binary_adj.get(&node).cloned().unwrap_or_default()
    }

    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.neighbors_with_weights(node)
    }

    fn hyperedges_for_entity(&self, node: EntityId) -> Vec<HyperEdgeId> {
        self.node_to_hyperedges
            .get(&node)
            .cloned()
            .unwrap_or_default()
    }

    fn get_hyperedge(&self, id: HyperEdgeId) -> Option<Arc<HyperEdge>> {
        self.hyperedges.get(&id).cloned()
    }
}

proptest! {
    #[test]
    fn test_apprh_bit_identical_permutation_invariance(
        seed_id in 1u64..10u64,
        decay in 0.1f32..1.0f32,
    ) {
        let e1 = EntityId::new(seed_id);
        let e2 = EntityId::new(seed_id + 1);
        let e3 = EntityId::new(seed_id + 2);
        let e4 = EntityId::new(seed_id + 3);

        let binary_edges = vec![
            (e1, e2, 1.0f32),
            (e2, e3, 0.5f32),
            (e3, e4, 0.8f32),
            (e1, e3, 0.2f32),
        ];

        let hyperedges = vec![
            (100u64, vec![e1, e2, e3], 1.5f32),
            (101u64, vec![e2, e3, e4], 1.0f32),
        ];

        let p_graph1 = PermutedGraph {
            binary_edges: binary_edges.clone(),
            hyperedges: hyperedges.clone(),
        };

        let mut rev_binary = binary_edges.clone();
        rev_binary.reverse();
        let mut rev_hyper = hyperedges.clone();
        rev_hyper.reverse();

        let p_graph2 = PermutedGraph {
            binary_edges: rev_binary,
            hyperedges: rev_hyper,
        };

        let g1 = p_graph1.build_graph();
        let g2 = p_graph2.build_graph();

        let seeds = vec![e1];
        let ppr_params = PprParams::default();

        let res1 = forward_push_apprh(&g1, &seeds, &ppr_params, decay).unwrap();
        let res2 = forward_push_apprh(&g2, &seeds, &ppr_params, decay).unwrap();

        prop_assert_eq!(res1.len(), res2.len());

        for (k, v1) in &res1 {
            let v2 = res2.get(k).expect("key missing in permuted graph result");
            prop_assert_eq!(v1.to_bits(), v2.to_bits());
        }
    }
}
