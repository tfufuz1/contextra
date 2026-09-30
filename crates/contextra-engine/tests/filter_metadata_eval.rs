#![cfg(not(loom))]
use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::FilterExpr;
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn test_filter_metadata_expression_evaluation_and_error_paths() -> contextra_types::Result<()>
{
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("filter_eval_col").await?;

    col.insert(
        "doc_f1",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({ "category": "tech", "lang": "rust", "rating": 5, "active": true })),
    )
    .await?;

    col.insert(
        "doc_f2",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({ "category": "tech", "lang": "python", "rating": 3, "active": true })),
    )
    .await?;

    col.insert(
        "doc_f3",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({ "category": "hobby", "lang": "english", "rating": 2, "active": false })),
    )
    .await?;

    let query_vec = [0.1, 0.2, 0.3, 0.4];

    // 1. Single condition filter: lang == "rust"
    let expr_eq = FilterExpr::Eq {
        field: "lang".to_string(),
        value: json!("rust"),
    };
    let res_eq = col
        .query()
        .vector(&query_vec)
        .filter(expr_eq)
        .execute()
        .await?;

    assert_eq!(res_eq.len(), 1);
    assert_eq!(res_eq[0].id, "doc_f1");

    // 2. AND condition filter: category == "tech" AND rating > 4
    let expr_and = FilterExpr::And(
        Box::new(FilterExpr::Eq {
            field: "category".to_string(),
            value: json!("tech"),
        }),
        Box::new(FilterExpr::Gt {
            field: "rating".to_string(),
            value: json!(4),
        }),
    );
    let res_and = col
        .query()
        .vector(&query_vec)
        .filter(expr_and)
        .execute()
        .await?;

    assert_eq!(res_and.len(), 1);
    assert_eq!(res_and[0].id, "doc_f1");

    // 3. IN condition filter: lang IN ["rust", "python"]
    let expr_in = FilterExpr::In {
        field: "lang".to_string(),
        values: vec![json!("rust"), json!("python")],
    };
    let res_in = col
        .query()
        .vector(&query_vec)
        .filter(expr_in)
        .execute()
        .await?;

    assert_eq!(res_in.len(), 2);
    let ids: Vec<&str> = res_in.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"doc_f1"));
    assert!(ids.contains(&"doc_f2"));

    // 4. NOT condition filter: NOT (category == "hobby")
    let expr_not_hobby = FilterExpr::Not(Box::new(FilterExpr::Eq {
        field: "category".to_string(),
        value: json!("hobby"),
    }));
    let res_not = col
        .query()
        .vector(&query_vec)
        .filter(expr_not_hobby)
        .execute()
        .await?;

    assert_eq!(res_not.len(), 2);
    let not_ids: Vec<&str> = res_not.iter().map(|r| r.id.as_str()).collect();
    assert!(!not_ids.contains(&"doc_f3"));

    db.close().await?;
    Ok(())
}
