// FILE-CONTEXT Header (Format v3)
// ZWECK: Strukturierte, deterministische Auswertung von Teilziel-Bedingungen (GoalCondition) für Agentenschritte.
// INVARIANTEN: Rein deterministisch; keine Uhr- oder Zufallsabhängigkeit; niemals panic in evaluate/describe/parse.
// NICHT-OFFENSICHTLICH: Dot-getrennte JSON-Pfadtraversierung liefert bei nicht-existierenden Pfaden oder Typ-Mismatches stets false.
// HOTSPOTS: GoalCondition::evaluate (ll. 50-100), resolve_json_path (ll. 110-140).
// STAND: TS:2026-10-01T00:00:00Z (SESSION: 088b4a44)

//! Strukturierte, deterministische Auswertung von Teilziel-Bedingungen für Agentenschritte.
//!
//! Stellt den Enum [`GoalCondition`] sowie die Konvertierungsfunktion [`parse_legacy_condition_string`]
//! bereit, um dynamische Kanten-Bedingungen im Agenten-Workflow ohne freie Strings zu prüfen.

#![forbid(unsafe_code)]

use crate::step::StepResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Repräsentiert eine strukturierte Bedingung zur Auswertung, ob ein Teilziel eines Agentenschritts erfüllt ist.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GoalCondition {
    /// Prüft, ob ein JSON-Feld unter `field_path` im `StepResult::output` mit `expected` übereinstimmt.
    OutputFieldEquals {
        /// Punkt-getrennter JSON-Pfad (z. B. "status.done").
        field_path: String,
        /// Erwarteter JSON-Wert.
        expected: Value,
    },
    /// Prüft, ob ein String-Feld unter `field_path` im `StepResult::output` den Substring `substring` enthält.
    OutputFieldContains {
        /// Punkt-getrennter JSON-Pfad (z. B. "result.message").
        field_path: String,
        /// Zu suchender Substring.
        substring: String,
    },
    /// Prüft, ob mindestens `threshold` Tokens im `StepResult` verbraucht wurden.
    TokensConsumedAtLeast {
        /// Minimale Tokenanzahl.
        threshold: usize,
    },
    /// Prüft, ob `StepResult::next_edge` vorhanden ist und mit `expected_edge` übereinstimmt.
    NextEdgeEquals {
        /// Erwartete Kantenbezeichnung.
        expected_edge: String,
    },
    /// Logisches UND über alle enthaltenen Teilbedingungen.
    ///
    /// Ein leerer Vektor `All(vec![])` liefert vakuum-bedingt (vacuous truth) stets `true`.
    All(Vec<GoalCondition>),
    /// Logisches ODER über die enthaltenen Teilbedingungen.
    ///
    /// Ein leerer Vektor `Any(vec![])` liefert stets `false`.
    Any(Vec<GoalCondition>),
    /// Logische Negation einer Teilbedingung.
    Not(Box<GoalCondition>),
}

impl GoalCondition {
    /// Wertet die Bedingung deterministisch gegen ein gegebenes [`StepResult`] aus.
    ///
    /// Liefert bei fehlendem oder ungültigem JSON-Pfad stets `false`. Panict niemals.
    pub fn evaluate(&self, result: &StepResult) -> bool {
        match self {
            GoalCondition::OutputFieldEquals {
                field_path,
                expected,
            } => {
                if let Some(val) = resolve_json_path(&result.output, field_path) {
                    val == expected
                } else {
                    false
                }
            }
            GoalCondition::OutputFieldContains {
                field_path,
                substring,
            } => {
                if let Some(Value::String(s)) = resolve_json_path(&result.output, field_path) {
                    s.contains(substring)
                } else {
                    false
                }
            }
            GoalCondition::TokensConsumedAtLeast { threshold } => {
                result.tokens_consumed >= *threshold
            }
            GoalCondition::NextEdgeEquals { expected_edge } => {
                if let Some(ref next_edge) = result.next_edge {
                    next_edge == expected_edge
                } else {
                    false
                }
            }
            GoalCondition::All(conditions) => conditions.iter().all(|cond| cond.evaluate(result)),
            GoalCondition::Any(conditions) => conditions.iter().any(|cond| cond.evaluate(result)),
            GoalCondition::Not(condition) => !condition.evaluate(result),
        }
    }

    /// Liefert eine menschenlesbare Beschreibung der Bedingung (z. B. für Audit-Logging).
    pub fn describe(&self) -> String {
        match self {
            GoalCondition::OutputFieldEquals {
                field_path,
                expected,
            } => {
                format!("OutputFieldEquals({field_path} == {expected})")
            }
            GoalCondition::OutputFieldContains {
                field_path,
                substring,
            } => {
                format!("OutputFieldContains({field_path} contains \"{substring}\")")
            }
            GoalCondition::TokensConsumedAtLeast { threshold } => {
                format!("TokensConsumedAtLeast(tokens_consumed >= {threshold})")
            }
            GoalCondition::NextEdgeEquals { expected_edge } => {
                format!("NextEdgeEquals(next_edge == \"{expected_edge}\")")
            }
            GoalCondition::All(conditions) => {
                let parts: Vec<String> = conditions.iter().map(|c| c.describe()).collect();
                format!("All([{}])", parts.join(", "))
            }
            GoalCondition::Any(conditions) => {
                let parts: Vec<String> = conditions.iter().map(|c| c.describe()).collect();
                format!("Any([{}])", parts.join(", "))
            }
            GoalCondition::Not(condition) => {
                format!("Not({})", condition.describe())
            }
        }
    }
}

/// Traversiert eine `serde_json::Value`-Struktur anhand eines Punkt-getrennten Pfads (z. B. "status.done").
///
/// Unterstützt Objekt-Schlüssel und Array-Indizes (numerische Segmentnamen).
fn resolve_json_path<'a>(value: &'a Value, field_path: &str) -> Option<&'a Value> {
    if field_path.trim().is_empty() {
        return None;
    }
    let mut current = value;
    for segment in field_path.split('.') {
        if segment.is_empty() {
            return None;
        }
        match current {
            Value::Object(map) => {
                current = map.get(segment)?;
            }
            Value::Array(arr) => {
                let index = segment.parse::<usize>().ok()?;
                current = arr.get(index)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

/// Bildet einen freien Bedingungs-String aus Altbestand auf eine strukturierte [`GoalCondition`] ab.
///
/// Unterstützt Gleichheitsprüfungen ("field == value", "next_edge == edge_name"),
/// Substring-Suchen ("field contains substring") und Token-Schwellenwerte ("tokens_consumed >= N").
/// Bei nicht parsbarer Syntax wird `None` zurückgegeben.
pub fn parse_legacy_condition_string(raw: &str) -> Option<GoalCondition> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some((left, right)) = trimmed.split_once("==") {
        let left = left.trim();
        let right = right.trim();
        if left.is_empty() || right.is_empty() {
            return None;
        }

        if left == "next_edge" {
            let unquoted = right.trim_matches('"').trim_matches('\'');
            return Some(GoalCondition::NextEdgeEquals {
                expected_edge: unquoted.to_string(),
            });
        }

        let expected_value = serde_json::from_str::<Value>(right).unwrap_or_else(|_| {
            let unquoted = right.trim_matches('"').trim_matches('\'');
            Value::String(unquoted.to_string())
        });

        return Some(GoalCondition::OutputFieldEquals {
            field_path: left.to_string(),
            expected: expected_value,
        });
    }

    if let Some((left, right)) = trimmed.split_once(" contains ") {
        let left = left.trim();
        let right = right.trim();
        if left.is_empty() || right.is_empty() {
            return None;
        }
        let unquoted = right.trim_matches('"').trim_matches('\'');
        return Some(GoalCondition::OutputFieldContains {
            field_path: left.to_string(),
            substring: unquoted.to_string(),
        });
    }

    if let Some((left, right)) = trimmed.split_once(">=") {
        let left = left.trim();
        let right = right.trim();
        if left == "tokens_consumed" {
            if let Ok(threshold) = right.parse::<usize>() {
                return Some(GoalCondition::TokensConsumedAtLeast { threshold });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn dummy_step_result(
        output: Value,
        tokens_consumed: usize,
        next_edge: Option<&str>,
    ) -> StepResult {
        StepResult {
            node_id: "node_1".to_string(),
            output,
            tokens_consumed,
            next_edge: next_edge.map(ToString::to_string),
        }
    }

    #[test]
    fn test_output_field_equals() {
        let res = dummy_step_result(
            json!({
                "status": {
                    "done": true,
                    "code": 200
                },
                "items": [
                    {"name": "first"}
                ]
            }),
            50,
            None,
        );

        let cond_true = GoalCondition::OutputFieldEquals {
            field_path: "status.done".to_string(),
            expected: json!(true),
        };
        assert!(cond_true.evaluate(&res));

        let cond_array = GoalCondition::OutputFieldEquals {
            field_path: "items.0.name".to_string(),
            expected: json!("first"),
        };
        assert!(cond_array.evaluate(&res));

        let cond_false_val = GoalCondition::OutputFieldEquals {
            field_path: "status.code".to_string(),
            expected: json!(500),
        };
        assert!(!cond_false_val.evaluate(&res));

        let cond_missing = GoalCondition::OutputFieldEquals {
            field_path: "status.missing_field".to_string(),
            expected: json!(null),
        };
        assert!(!cond_missing.evaluate(&res));
    }

    #[test]
    fn test_output_field_contains() {
        let res = dummy_step_result(
            json!({
                "message": "Operation completed successfully",
                "count": 42
            }),
            10,
            None,
        );

        let cond_contains = GoalCondition::OutputFieldContains {
            field_path: "message".to_string(),
            substring: "completed".to_string(),
        };
        assert!(cond_contains.evaluate(&res));

        let cond_not_contains = GoalCondition::OutputFieldContains {
            field_path: "message".to_string(),
            substring: "failed".to_string(),
        };
        assert!(!cond_not_contains.evaluate(&res));

        // Non-string field return false, no panic
        let cond_non_string = GoalCondition::OutputFieldContains {
            field_path: "count".to_string(),
            substring: "42".to_string(),
        };
        assert!(!cond_non_string.evaluate(&res));

        // Missing field returns false
        let cond_missing = GoalCondition::OutputFieldContains {
            field_path: "non_existent".to_string(),
            substring: "test".to_string(),
        };
        assert!(!cond_missing.evaluate(&res));
    }

    #[test]
    fn test_tokens_consumed_at_least() {
        let res = dummy_step_result(json!({}), 100, None);

        let cond_pass = GoalCondition::TokensConsumedAtLeast { threshold: 100 };
        assert!(cond_pass.evaluate(&res));

        let cond_pass_lower = GoalCondition::TokensConsumedAtLeast { threshold: 50 };
        assert!(cond_pass_lower.evaluate(&res));

        let cond_fail = GoalCondition::TokensConsumedAtLeast { threshold: 101 };
        assert!(!cond_fail.evaluate(&res));
    }

    #[test]
    fn test_next_edge_equals() {
        let res_edge = dummy_step_result(json!({}), 10, Some("edge_alpha"));
        let res_no_edge = dummy_step_result(json!({}), 10, None);

        let cond_edge = GoalCondition::NextEdgeEquals {
            expected_edge: "edge_alpha".to_string(),
        };
        assert!(cond_edge.evaluate(&res_edge));

        let cond_edge_mismatch = GoalCondition::NextEdgeEquals {
            expected_edge: "edge_beta".to_string(),
        };
        assert!(!cond_edge_mismatch.evaluate(&res_edge));

        assert!(!cond_edge.evaluate(&res_no_edge));
    }

    #[test]
    fn test_all_any_not_logic() {
        let res = dummy_step_result(
            json!({
                "status": "ok",
                "score": 95
            }),
            20,
            Some("next_step"),
        );

        let cond_ok = GoalCondition::OutputFieldEquals {
            field_path: "status".to_string(),
            expected: json!("ok"),
        };
        let cond_tokens = GoalCondition::TokensConsumedAtLeast { threshold: 10 };
        let cond_false = GoalCondition::TokensConsumedAtLeast { threshold: 500 };

        // All
        assert!(GoalCondition::All(vec![cond_ok.clone(), cond_tokens.clone()]).evaluate(&res));
        assert!(!GoalCondition::All(vec![cond_ok.clone(), cond_false.clone()]).evaluate(&res));
        // Vacuous truth for empty All
        assert!(GoalCondition::All(vec![]).evaluate(&res));

        // Any
        assert!(GoalCondition::Any(vec![cond_false.clone(), cond_tokens.clone()]).evaluate(&res));
        assert!(!GoalCondition::Any(vec![cond_false.clone()]).evaluate(&res));
        // Neutral element for empty Any
        assert!(!GoalCondition::Any(vec![]).evaluate(&res));

        // Not
        assert!(GoalCondition::Not(Box::new(cond_false.clone())).evaluate(&res));
        assert!(!GoalCondition::Not(Box::new(cond_ok.clone())).evaluate(&res));
    }

    #[test]
    fn test_describe() {
        let cond_eq = GoalCondition::OutputFieldEquals {
            field_path: "status.done".to_string(),
            expected: json!(true),
        };
        assert_eq!(cond_eq.describe(), "OutputFieldEquals(status.done == true)");

        let cond_contains = GoalCondition::OutputFieldContains {
            field_path: "error.msg".to_string(),
            substring: "timeout".to_string(),
        };
        assert_eq!(
            cond_contains.describe(),
            "OutputFieldContains(error.msg contains \"timeout\")"
        );

        let cond_tokens = GoalCondition::TokensConsumedAtLeast { threshold: 42 };
        assert_eq!(
            cond_tokens.describe(),
            "TokensConsumedAtLeast(tokens_consumed >= 42)"
        );

        let cond_edge = GoalCondition::NextEdgeEquals {
            expected_edge: "retry".to_string(),
        };
        assert_eq!(
            cond_edge.describe(),
            "NextEdgeEquals(next_edge == \"retry\")"
        );

        let cond_all = GoalCondition::All(vec![cond_eq, cond_tokens]);
        assert_eq!(
            cond_all.describe(),
            "All([OutputFieldEquals(status.done == true), TokensConsumedAtLeast(tokens_consumed >= 42)])"
        );

        let cond_not = GoalCondition::Not(Box::new(cond_edge));
        assert_eq!(
            cond_not.describe(),
            "Not(NextEdgeEquals(next_edge == \"retry\"))"
        );
    }

    #[test]
    fn test_parse_legacy_condition_string() {
        // OutputFieldEquals parsing
        let parsed_eq_bool = parse_legacy_condition_string("status.done == true");
        assert_eq!(
            parsed_eq_bool,
            Some(GoalCondition::OutputFieldEquals {
                field_path: "status.done".to_string(),
                expected: json!(true),
            })
        );

        let parsed_eq_num = parse_legacy_condition_string("code == 200");
        assert_eq!(
            parsed_eq_num,
            Some(GoalCondition::OutputFieldEquals {
                field_path: "code".to_string(),
                expected: json!(200),
            })
        );

        let parsed_eq_unquoted_str = parse_legacy_condition_string("state == active");
        assert_eq!(
            parsed_eq_unquoted_str,
            Some(GoalCondition::OutputFieldEquals {
                field_path: "state".to_string(),
                expected: json!("active"),
            })
        );

        // NextEdgeEquals parsing
        let parsed_edge = parse_legacy_condition_string("next_edge == 'branch_1'");
        assert_eq!(
            parsed_edge,
            Some(GoalCondition::NextEdgeEquals {
                expected_edge: "branch_1".to_string(),
            })
        );

        // OutputFieldContains parsing
        let parsed_contains = parse_legacy_condition_string("result.summary contains 'error'");
        assert_eq!(
            parsed_contains,
            Some(GoalCondition::OutputFieldContains {
                field_path: "result.summary".to_string(),
                substring: "error".to_string(),
            })
        );

        // TokensConsumedAtLeast parsing
        let parsed_tokens = parse_legacy_condition_string("tokens_consumed >= 500");
        assert_eq!(
            parsed_tokens,
            Some(GoalCondition::TokensConsumedAtLeast { threshold: 500 })
        );

        // Invalid / Non-parsable syntax
        assert_eq!(
            parse_legacy_condition_string("invalid condition format"),
            None
        );
        assert_eq!(parse_legacy_condition_string(""), None);
        assert_eq!(parse_legacy_condition_string("   "), None);
    }
}
