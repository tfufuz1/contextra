use contextra_core::{DocId, EntityId};
use contextra_graph::path_rag::{GraphPath, PathGraph, PathRAGEngine};
use std::collections::HashMap;

struct DummyGraph;

impl PathGraph for DummyGraph {
    fn neighbors_with_weights(&self, _node: EntityId) -> Vec<(EntityId, f32)> {
        vec![]
    }
    fn predecessors_with_weights(&self, _node: EntityId) -> Vec<(EntityId, f32)> {
        vec![]
    }
}

#[test]
fn test_path_rag_to_rrf_signal_docid_conversion_and_sorting() {
    let engine = PathRAGEngine::new(DummyGraph, 4, 0.1);

    let path1 = GraphPath {
        nodes: vec![EntityId::new(10), EntityId::new(20)],
        edge_weights: vec![0.8],
        confidence: 0.8,
        total_flow: 0.8,
    };

    let path2 = GraphPath {
        nodes: vec![EntityId::new(20), EntityId::new(30)],
        edge_weights: vec![0.9],
        confidence: 0.9,
        total_flow: 1.5,
    };

    let paths = vec![path1, path2];

    let signal = engine.to_rrf_signal(&paths);

    assert_eq!(
        signal.len(),
        3,
        "Expected 3 unique nodes/DocIds in RRF signal"
    );

    // Scores should be sorted descending
    for i in 1..signal.len() {
        assert!(
            signal[i - 1].1 >= signal[i].1,
            "RRF signal must be sorted descending by score"
        );
    }

    // Verify DocId matching expected numerical values
    let map: HashMap<DocId, f32> = signal.into_iter().collect();

    #[cfg(not(feature = "docid-128"))]
    {
        assert!(map.contains_key(&DocId(10)));
        assert!(map.contains_key(&DocId(20)));
        assert!(map.contains_key(&DocId(30)));
    }

    #[cfg(feature = "docid-128")]
    {
        assert!(map.contains_key(&DocId(10u128)));
        assert!(map.contains_key(&DocId(20u128)));
        assert!(map.contains_key(&DocId(30u128)));
    }
}
