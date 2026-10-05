#[path = "../src/harness/dependency_graph_audit.rs"]
mod dependency_graph_audit;

use std::collections::{BTreeMap, HashMap};
use dependency_graph_audit::{detect_cycles, detect_ring_jumps, Finding};
use xtask::check_ring_layering::Ring;

#[test]
fn test_synthetic_graph_cycle_detection_three_crates() {
    let mut graph = BTreeMap::new();
    graph.insert("contextra-a".to_string(), vec!["contextra-b".to_string()]);
    graph.insert("contextra-b".to_string(), vec!["contextra-c".to_string()]);
    graph.insert("contextra-c".to_string(), vec!["contextra-a".to_string()]);

    let findings = detect_cycles(&graph);
    assert_eq!(findings.len(), 1);

    match &findings[0] {
        Finding::Cycle { path, message } => {
            assert_eq!(
                path,
                &vec![
                    "contextra-a".to_string(),
                    "contextra-b".to_string(),
                    "contextra-c".to_string(),
                    "contextra-a".to_string()
                ]
            );
            assert!(message.contains("contextra-a -> contextra-b -> contextra-c -> contextra-a"));
        }
        _ => panic!("Expected Finding::Cycle"),
    }
}

#[test]
fn test_synthetic_graph_acyclic() {
    let mut graph = BTreeMap::new();
    graph.insert("contextra-a".to_string(), vec!["contextra-b".to_string()]);
    graph.insert("contextra-b".to_string(), vec!["contextra-c".to_string()]);
    graph.insert("contextra-c".to_string(), vec![]);

    let findings = detect_cycles(&graph);
    assert!(findings.is_empty());
}

#[test]
fn test_synthetic_ring_jumps() {
    let mut graph = BTreeMap::new();
    // contextra-agent (Ring 3) -> contextra-types (Ring 0): distance = 3 > 1 => ring jump
    graph.insert("contextra-agent".to_string(), vec!["contextra-types".to_string()]);
    // contextra-store (Ring 1) -> contextra-types (Ring 0): distance = 1 <= 1 => no jump
    graph.insert("contextra-store".to_string(), vec!["contextra-types".to_string()]);

    let mut ring_map = HashMap::new();
    ring_map.insert("contextra-agent".to_string(), Ring::Ring3);
    ring_map.insert("contextra-store".to_string(), Ring::Ring1);
    ring_map.insert("contextra-types".to_string(), Ring::Ring0);

    let findings = detect_ring_jumps(&graph, &ring_map);
    assert_eq!(findings.len(), 1);

    match &findings[0] {
        Finding::RingJump {
            from_crate,
            from_ring,
            to_crate,
            to_ring,
            distance,
            message,
        } => {
            assert_eq!(from_crate, "contextra-agent");
            assert_eq!(*from_ring, 3);
            assert_eq!(to_crate, "contextra-types");
            assert_eq!(*to_ring, 0);
            assert_eq!(*distance, 3);
            assert!(message.contains("distance 3"));
        }
        _ => panic!("Expected Finding::RingJump"),
    }
}
