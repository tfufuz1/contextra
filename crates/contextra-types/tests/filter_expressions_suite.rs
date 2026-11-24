//! Integrations-Testsuite für strukturierte Metadaten-Filterausdrücke.
//! Verifiziert alle Vergleichsoperatoren, logische Verknüpfungen und Null-Wert-Behandlung.

//! Comprehensive filter evaluation and precedence test suite for `contextra-types`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_types::filter::FilterExpr;
use serde_json::json;

#[test]
/// Prüft exakte Gleichheits- (`Eq`) und Ungleichheitsoperatoren (`Ne`) gegen Metadaten.
fn test_filter_equality_and_inequality_operators() {
    let meta = json!({
        "status": "active",
        "version": 2,
        "score": 85.5,
        "enabled": true,
        "tags": ["prod", "db", "rust"]
    });

    let eq_match = FilterExpr::Eq {
        field: "status".to_string(),
        value: json!("active"),
    };
    assert!(eq_match.evaluate(&meta));

    let eq_miss = FilterExpr::Eq {
        field: "status".to_string(),
        value: json!("inactive"),
    };
    assert!(!eq_miss.evaluate(&meta));

    let ne_match = FilterExpr::Ne {
        field: "status".to_string(),
        value: json!("pending"),
    };
    assert!(ne_match.evaluate(&meta));

    let ne_miss = FilterExpr::Ne {
        field: "status".to_string(),
        value: json!("active"),
    };
    assert!(!ne_miss.evaluate(&meta));
}

#[test]
/// Prüft numerische Vergleichsoperatoren (`Gt`, `Gte`, `Lt`, `Lte`) auf Ganz- und Fließkommawerten.
fn test_filter_numeric_comparisons() {
    let meta = json!({
        "count": 50,
        "price": 19.99
    });

    let gt_ok = FilterExpr::Gt {
        field: "count".to_string(),
        value: json!(40),
    };
    assert!(gt_ok.evaluate(&meta));

    let gt_fail = FilterExpr::Gt {
        field: "count".to_string(),
        value: json!(50),
    };
    assert!(!gt_fail.evaluate(&meta));

    let gte_ok1 = FilterExpr::Gte {
        field: "count".to_string(),
        value: json!(50),
    };
    assert!(gte_ok1.evaluate(&meta));

    let gte_ok2 = FilterExpr::Gte {
        field: "count".to_string(),
        value: json!(49),
    };
    assert!(gte_ok2.evaluate(&meta));

    let lt_ok = FilterExpr::Lt {
        field: "price".to_string(),
        value: json!(20.0),
    };
    assert!(lt_ok.evaluate(&meta));

    let lte_ok = FilterExpr::Lte {
        field: "price".to_string(),
        value: json!(19.99),
    };
    assert!(lte_ok.evaluate(&meta));
}

#[test]
/// Prüft Mengen- (`In`, `NotIn`) und Existenzoperatoren (`Exists`) sowie Array-Enthaltensein.
fn test_filter_set_and_contains_operators() {
    let meta = json!({
        "environment": "production",
        "features": ["auth", "logging", "metrics"]
    });

    let in_ok = FilterExpr::In {
        field: "environment".to_string(),
        values: vec![json!("staging"), json!("production")],
    };
    assert!(in_ok.evaluate(&meta));

    let in_fail = FilterExpr::In {
        field: "environment".to_string(),
        values: vec![json!("dev"), json!("test")],
    };
    assert!(!in_fail.evaluate(&meta));

    let not_in_ok = FilterExpr::NotIn {
        field: "environment".to_string(),
        values: vec![json!("dev"), json!("test")],
    };
    assert!(not_in_ok.evaluate(&meta));

    let contains_ok = FilterExpr::In {
        field: "features".to_string(),
        values: vec![json!("auth")],
    };
    assert!(contains_ok.evaluate(&meta));

    let contains_fail = FilterExpr::In {
        field: "features".to_string(),
        values: vec![json!("billing")],
    };
    assert!(!contains_fail.evaluate(&meta));

    let exists_ok = FilterExpr::Exists {
        field: "environment".to_string(),
        exists: true,
    };
    assert!(exists_ok.evaluate(&meta));
}

#[test]
/// Prüft logische Verknüpfungen mit `And`, `Or` und Negation (`Not`).
fn test_filter_logical_combinators_and_not() {
    let meta = json!({
        "env": "prod",
        "tier": 1,
        "region": "eu-central"
    });

    let and_expr = FilterExpr::And(
        Box::new(FilterExpr::Eq {
            field: "env".to_string(),
            value: json!("prod"),
        }),
        Box::new(FilterExpr::Eq {
            field: "tier".to_string(),
            value: json!(1),
        }),
    );
    assert!(and_expr.evaluate(&meta));

    let or_expr = FilterExpr::Or(
        Box::new(FilterExpr::Eq {
            field: "env".to_string(),
            value: json!("staging"),
        }),
        Box::new(FilterExpr::Eq {
            field: "region".to_string(),
            value: json!("eu-central"),
        }),
    );
    assert!(or_expr.evaluate(&meta));

    let not_expr = FilterExpr::Not(Box::new(FilterExpr::Eq {
        field: "env".to_string(),
        value: json!("dev"),
    }));
    assert!(not_expr.evaluate(&meta));
}

#[test]
/// Prüft das Auswertungsverhalten bei `null`-Werten und nicht existierenden Feldern.
fn test_filter_null_and_missing_field_evaluations() {
    let meta = json!({
        "present": "value",
        "null_field": null
    });

    let eq_missing = FilterExpr::Eq {
        field: "missing_field".to_string(),
        value: json!("value"),
    };
    assert!(!eq_missing.evaluate(&meta));

    let eq_null = FilterExpr::Eq {
        field: "null_field".to_string(),
        value: json!(null),
    };
    assert!(eq_null.evaluate(&meta));

    let null_meta = json!(null);
    let eq_against_null_meta = FilterExpr::Eq {
        field: "present".to_string(),
        value: json!("value"),
    };
    assert!(!eq_against_null_meta.evaluate(&null_meta));
}
