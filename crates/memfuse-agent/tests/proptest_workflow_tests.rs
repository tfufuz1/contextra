#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// FILE-CONTEXT Header (Format v3)
// ZWECK: Property-based testing for memfuse-agent structural invariants (StateGraph, AgentContext, BackgroundEvent).
// INVARIANTEN: Property tests verify boundary constraints and state consistency independently without mirroring production validation formulas.
// NICHT-OFFENSICHTLICH: Uses independent partitions (known valid IDs vs known invalid IDs) rather than re-implementing validation logic.
// HOTSPOTS: Random seed generators & iteration loops.
// STAND: TS:2026-09-13T01:36:52Z (SESSION: c1c85419)

use memfuse_agent::{BackgroundEvent, NodeType, StateGraph};

fn generate_valid_id(seed: usize, len: usize) -> String {
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";
    let chars_len = chars.len();
    let target_len = (len % 200) + 1; // 1..=200 chars (strictly valid len)
    let mut s = String::with_capacity(target_len);
    let mut current = seed;
    for _ in 0..target_len {
        current = current.wrapping_mul(1103515245).wrapping_add(12345);
        let idx = (current / 65536) % chars_len;
        s.push(chars.chars().nth(idx).unwrap());
    }
    s
}

#[test]
fn test_state_graph_node_addition_valid_inputs_always_succeed() {
    for seed in 0..500 {
        let node_id = generate_valid_id(seed, seed);
        let mut graph = StateGraph::default();
        let res = graph.try_add_node(
            &node_id,
            "test description",
            NodeType::Task,
            Some("test_handler"),
        );
        assert!(res.is_ok(), "Valid node ID {:?} failed to insert", node_id);
        assert!(graph.get_node(&node_id).is_some());
    }
}

#[test]
fn test_state_graph_node_addition_invalid_null_byte_always_rejected() {
    for seed in 0..100 {
        let prefix = generate_valid_id(seed, 10);
        let suffix = generate_valid_id(seed + 100, 10);
        let node_id = format!("{}\0{}", prefix, suffix);
        let mut graph = StateGraph::default();
        let res = graph.try_add_node(
            &node_id,
            "test description",
            NodeType::Task,
            Some("test_handler"),
        );
        assert!(
            res.is_err(),
            "Null byte node ID {:?} was unexpectedly accepted",
            node_id
        );
    }
}

#[test]
fn test_state_graph_node_addition_invalid_oversized_always_rejected() {
    let oversized_id = "a".repeat(257);
    let mut graph = StateGraph::default();
    let res = graph.try_add_node(
        &oversized_id,
        "test description",
        NodeType::Task,
        Some("test_handler"),
    );
    assert!(res.is_err(), "Oversized node ID was unexpectedly accepted");
}

#[test]
fn test_background_event_validation_valid_inputs_always_succeed() {
    for seed in 0..500 {
        let source_id = generate_valid_id(seed, seed);
        let payload = serde_json::json!({ "seed": seed });
        let res = BackgroundEvent::try_new(payload, &source_id, 1);
        assert!(
            res.is_ok(),
            "Valid source ID {:?} failed in BackgroundEvent::try_new",
            source_id
        );
        let event = res.unwrap();
        assert_eq!(event.source, source_id);
    }
}
