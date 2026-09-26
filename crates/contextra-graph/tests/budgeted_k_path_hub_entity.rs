#![cfg(feature = "k-path-diffusion")]

use contextra_graph::path_rag::k_path::{KPathConfig, KPathDiffusion};
use contextra_graph::path_rag::{PathGraph, PathRAGEngine};
use contextra_types::EntityId;
use std::collections::HashMap;

struct SyntheticHubGraph {
    edges: HashMap<EntityId, Vec<(EntityId, f32)>>,
}

impl SyntheticHubGraph {
    fn new_hub_graph(source: EntityId, hub: EntityId, target: EntityId, fanout: usize) -> Self {
        let mut map: HashMap<EntityId, Vec<(EntityId, f32)>> = HashMap::new();

        // source -> hub
        map.entry(source).or_default().push((hub, 1.0));

        // hub -> fanout leaf nodes
        for i in 0..fanout {
            let leaf = EntityId::new(100_000 + i as u64);
            map.entry(hub).or_default().push((leaf, 1.0));

            // Connect first 10 leaves to target
            if i < 10 {
                map.entry(leaf).or_default().push((target, 1.0));
            }
        }

        Self { edges: map }
    }
}

impl PathGraph for SyntheticHubGraph {
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
}

#[test]
fn test_budgeted_k_path_hub_entity_high_fanout_exhaustion() {
    let source = EntityId::new(10);
    let hub = EntityId::new(20);
    let target = EntityId::new(30);
    let fanout = 5000;

    let graph = SyntheticHubGraph::new_hub_graph(source, hub, target, fanout);
    let engine = PathRAGEngine::with_defaults(graph);

    let max_visited_nodes = 50;
    let config = KPathConfig {
        k: 100,
        max_visited_nodes,
        max_hops: 4,
    };

    let result = engine
        .find_k_paths(source, target, &config)
        .expect("find_k_paths must return Ok");

    assert!(
        result.budget_exhausted,
        "Traversal must exhaust budget on high fan-out hub node"
    );

    for path in &result.paths {
        assert!(
            path.len() - 1 <= config.max_hops,
            "Discovered path hop count must not exceed max_hops"
        );
        assert_eq!(path.first(), Some(&source));
        assert_eq!(path.last(), Some(&target));
    }
}

#[test]
fn test_budgeted_k_path_within_budget_success() {
    let source = EntityId::new(10);
    let hub = EntityId::new(20);
    let target = EntityId::new(30);
    let fanout = 5;

    let graph = SyntheticHubGraph::new_hub_graph(source, hub, target, fanout);
    let engine = PathRAGEngine::with_defaults(graph);

    let config = KPathConfig {
        k: 3,
        max_visited_nodes: 500,
        max_hops: 4,
    };

    let result = engine
        .find_k_paths(source, target, &config)
        .expect("find_k_paths must return Ok");

    assert!(
        !result.budget_exhausted,
        "Budget must not be exhausted when graph is within limits"
    );
    assert!(
        !result.paths.is_empty(),
        "Must discover paths to target node"
    );
    assert!(
        result.paths.len() <= config.k,
        "Must return at most k paths"
    );
}

#[test]
fn test_budgeted_k_path_edge_cases() {
    let source = EntityId::new(1);
    let target = EntityId::new(2);
    let graph = SyntheticHubGraph::new_hub_graph(source, EntityId::new(99), target, 0);
    let engine = PathRAGEngine::with_defaults(graph);

    // Same node
    let same_res = engine
        .find_k_paths(source, source, &KPathConfig::default())
        .unwrap();
    assert_eq!(same_res.paths, vec![vec![source]]);
    assert!(!same_res.budget_exhausted);

    // k = 0
    let zero_k_res = engine
        .find_k_paths(
            source,
            target,
            &KPathConfig {
                k: 0,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(zero_k_res.paths.is_empty());
    assert!(!zero_k_res.budget_exhausted);

    // max_visited_nodes = 0
    let zero_visited_res = engine
        .find_k_paths(
            source,
            target,
            &KPathConfig {
                max_visited_nodes: 0,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(zero_visited_res.paths.is_empty());
    assert!(zero_visited_res.budget_exhausted);
}
