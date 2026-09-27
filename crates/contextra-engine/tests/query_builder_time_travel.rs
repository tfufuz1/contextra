use contextra_engine::{Contextra, ContextraConfig};
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn test_query_builder_as_of_historical_time_travel_filtering() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("time_travel_col").await?;

    let vec = [0.1, 0.2, 0.3, 0.4];

    // Version 1 of document: valid for business time window [10, 50)
    col.insert(
        "policy_doc_v1",
        &vec,
        Some(json!({
            "valid_from": 10,
            "valid_until": 50,
            "version": 1
        })),
    )
    .await?;

    // Version 2 of document: valid for business time window [50, 100)
    col.insert(
        "policy_doc_v2",
        &vec,
        Some(json!({
            "valid_from": 50,
            "valid_until": 100,
            "version": 2
        })),
    )
    .await?;

    // 1. Time-travel query at t = 30 (when Version 1 was active)
    let results_t30 = col
        .query()
        .vector(&vec)
        .as_of(30)
        .execute()
        .await?;

    assert_eq!(
        results_t30.len(),
        1,
        "Query at as_of(30) must return exactly 1 document"
    );
    assert_eq!(
        results_t30[0].id, "policy_doc_v1",
        "Query at as_of(30) must return Version 1"
    );

    // 2. Time-travel query at t = 60 (when Version 2 was active)
    let results_t60 = col
        .query()
        .vector(&vec)
        .as_of(60)
        .execute()
        .await?;

    assert_eq!(
        results_t60.len(),
        1,
        "Query at as_of(60) must return exactly 1 document"
    );
    assert_eq!(
        results_t60[0].id, "policy_doc_v2",
        "Query at as_of(60) must return Version 2"
    );

    // 3. Boundary test: Query at t = 5 (before any document was valid)
    let results_t5 = col
        .query()
        .vector(&vec)
        .as_of(5)
        .execute()
        .await?;

    assert!(
        results_t5.is_empty(),
        "Query at as_of(5) prior to valid_from must return empty results"
    );

    // 4. Boundary test: Query at t = 100 (exact upper bound, exclusive)
    let results_t100 = col
        .query()
        .vector(&vec)
        .as_of(100)
        .execute()
        .await?;

    assert!(
        results_t100.is_empty(),
        "Query at as_of(100) at exact valid_until upper bound must return empty results"
    );

    db.close().await?;
    Ok(())
}
