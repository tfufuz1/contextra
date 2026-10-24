// FILE-CONTEXT
// ZWECK: Serde-Kompatibilitätstests für WorkflowEdge (Lesen alter JSONs ohne goal, Roundtrip neuer JSONs mit goal).
// INVARIANTEN: 100% Abwärtskompatibilität beim Deserialisieren alter Edge-Formate.

#![forbid(unsafe_code)]

use contextra_agent::{GoalCondition, WorkflowEdge};
use serde_json::json;

#[test]
fn test_legacy_edge_json_deserialization() {
    // Legacy JSON format without `goal` field
    let legacy_json = json!({
        "from": "start",
        "to": "task1",
        "condition": "status.done == true",
        "priority": 5
    });

    let edge: WorkflowEdge = serde_json::from_value(legacy_json).expect("deserialize legacy edge");

    assert_eq!(edge.from, "start");
    assert_eq!(edge.to, "task1");
    assert_eq!(edge.condition.as_deref(), Some("status.done == true"));
    assert_eq!(edge.priority, 5);
    // Goal defaults to None via #[serde(default)]
    assert_eq!(edge.goal, None);
}

#[test]
fn test_new_edge_json_roundtrip_with_goal() {
    let goal_cond = GoalCondition::OutputFieldEquals {
        field_path: "code".to_string(),
        expected: json!(200),
    };

    let original_edge = WorkflowEdge {
        from: "check_step".to_string(),
        to: "process_step".to_string(),
        condition: Some("code == 200".to_string()),
        priority: 10,
        goal: Some(goal_cond),
    };

    let serialized = serde_json::to_string(&original_edge).expect("serialize edge");
    assert!(serialized.contains("\"goal\""));

    let deserialized: WorkflowEdge = serde_json::from_str(&serialized).expect("deserialize edge");
    assert_eq!(deserialized, original_edge);
}

#[test]
fn test_new_edge_json_omits_none_goal() {
    let edge_none_goal = WorkflowEdge {
        from: "step_a".to_string(),
        to: "step_b".to_string(),
        condition: None,
        priority: 1,
        goal: None,
    };

    let serialized = serde_json::to_string(&edge_none_goal).expect("serialize edge");
    // skip_serializing_if = "Option::is_none" ensures "goal" key is omitted
    assert!(!serialized.contains("\"goal\""));

    let deserialized: WorkflowEdge = serde_json::from_str(&serialized).expect("deserialize edge");
    assert_eq!(deserialized, edge_none_goal);
}
