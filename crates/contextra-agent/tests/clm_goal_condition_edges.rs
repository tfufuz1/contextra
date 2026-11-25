// FILE-CONTEXT
// ZWECK: Tests für alle GoalCondition-Varianten, Randfälle und Determinismus.
// INVARIANTEN: Zero-Panic, deterministische Auswertung ohne Systemzeitzwang oder Zufall.

#![forbid(unsafe_code)]

use contextra_agent::{GoalCondition, StepResult};
use serde_json::json;

#[test]
fn test_output_field_equals_all_json_types() {
    let res = StepResult {
        node_id: "node1".to_string(),
        output: json!({
            "string_val": "hello",
            "number_val": 42,
            "bool_val": true,
            "null_val": null,
            "nested": {
                "arr": [10, 20, {"deep": "found"}]
            }
        }),
        tokens_consumed: 10,
        next_edge: None,
    };

    // Match string
    let cond = GoalCondition::OutputFieldEquals {
        field_path: "string_val".to_string(),
        expected: json!("hello"),
    };
    assert!(cond.evaluate(&res));

    // Match number
    let cond = GoalCondition::OutputFieldEquals {
        field_path: "number_val".to_string(),
        expected: json!(42),
    };
    assert!(cond.evaluate(&res));

    // Match bool
    let cond = GoalCondition::OutputFieldEquals {
        field_path: "bool_val".to_string(),
        expected: json!(true),
    };
    assert!(cond.evaluate(&res));

    // Match nested array index and object
    let cond = GoalCondition::OutputFieldEquals {
        field_path: "nested.arr.2.deep".to_string(),
        expected: json!("found"),
    };
    assert!(cond.evaluate(&res));

    // Mismatch expected value
    let cond_mismatch = GoalCondition::OutputFieldEquals {
        field_path: "number_val".to_string(),
        expected: json!(99),
    };
    assert!(!cond_mismatch.evaluate(&res));

    // Missing field path returns false, never panics
    let cond_missing = GoalCondition::OutputFieldEquals {
        field_path: "non.existent.path".to_string(),
        expected: json!("anything"),
    };
    assert!(!cond_missing.evaluate(&res));
}

#[test]
fn test_output_field_contains_edge_cases() {
    let res = StepResult {
        node_id: "node1".to_string(),
        output: json!({
            "text": "The quick brown fox jumps over the lazy dog",
            "num": 12345
        }),
        tokens_consumed: 5,
        next_edge: None,
    };

    let cond_match = GoalCondition::OutputFieldContains {
        field_path: "text".to_string(),
        substring: "brown fox".to_string(),
    };
    assert!(cond_match.evaluate(&res));

    let cond_no_match = GoalCondition::OutputFieldContains {
        field_path: "text".to_string(),
        substring: "cat".to_string(),
    };
    assert!(!cond_no_match.evaluate(&res));

    // Calling contains on non-string JSON value returns false without panic
    let cond_non_string = GoalCondition::OutputFieldContains {
        field_path: "num".to_string(),
        substring: "123".to_string(),
    };
    assert!(!cond_non_string.evaluate(&res));
}

#[test]
fn test_tokens_consumed_at_least_boundary() {
    let res = StepResult {
        node_id: "node1".to_string(),
        output: json!({}),
        tokens_consumed: 100,
        next_edge: None,
    };

    assert!(GoalCondition::TokensConsumedAtLeast { threshold: 100 }.evaluate(&res));
    assert!(GoalCondition::TokensConsumedAtLeast { threshold: 99 }.evaluate(&res));
    assert!(!GoalCondition::TokensConsumedAtLeast { threshold: 101 }.evaluate(&res));
}

#[test]
fn test_next_edge_equals() {
    let res_edge = StepResult {
        node_id: "node1".to_string(),
        output: json!({}),
        tokens_consumed: 0,
        next_edge: Some("branch_a".to_string()),
    };

    let res_no_edge = StepResult {
        node_id: "node1".to_string(),
        output: json!({}),
        tokens_consumed: 0,
        next_edge: None,
    };

    let cond = GoalCondition::NextEdgeEquals {
        expected_edge: "branch_a".to_string(),
    };

    assert!(cond.evaluate(&res_edge));
    assert!(!cond.evaluate(&res_no_edge));
}

#[test]
fn test_logical_combinators_and_determinism() {
    let res = StepResult {
        node_id: "node1".to_string(),
        output: json!({"status": "ok", "code": 200}),
        tokens_consumed: 50,
        next_edge: None,
    };

    let c1 = GoalCondition::OutputFieldEquals {
        field_path: "status".to_string(),
        expected: json!("ok"),
    };
    let c2 = GoalCondition::TokensConsumedAtLeast { threshold: 10 };
    let c3 = GoalCondition::NextEdgeEquals {
        expected_edge: "missing".to_string(),
    };

    // All
    let all_true = GoalCondition::All(vec![c1.clone(), c2.clone()]);
    let all_false = GoalCondition::All(vec![c1.clone(), c3.clone()]);
    assert!(all_true.evaluate(&res));
    assert!(!all_false.evaluate(&res));

    // Any
    let any_true = GoalCondition::Any(vec![c3.clone(), c1.clone()]);
    let any_false = GoalCondition::Any(vec![c3.clone()]);
    assert!(any_true.evaluate(&res));
    assert!(!any_false.evaluate(&res));

    // Not
    let not_true = GoalCondition::Not(Box::new(c3));
    let not_false = GoalCondition::Not(Box::new(c1));
    assert!(not_true.evaluate(&res));
    assert!(!not_false.evaluate(&res));

    // Evaluation 1000 times produces identical boolean output deterministically
    for _ in 0..1000 {
        assert!(all_true.evaluate(&res));
    }
}
