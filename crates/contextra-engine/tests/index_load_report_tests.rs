#![cfg(not(loom))]

use contextra_engine::collection::{IndexLoadReport, LoadPolicy};
use contextra_engine::{Contextra, ContextraConfig};
use contextra_ports::TestMetricsSink;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn index_load_strict_fails_on_corrupt_doc() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let col = db.collection("default").await?;

    // Insert 1 valid doc
    let vec1 = vec![1.0, 0.0, 0.0, 0.0];
    col.insert(
        "valid-1",
        &vec1,
        Some(serde_json::json!({"text": "Hello world"})),
    )
    .await?;

    // Inject corrupted document bytes directly into storage under user prefix key
    let corrupt_payload = b"CORRUPTED_SECRET_RECORD_DATA_NOT_JSON";
    let storage_key = col.namespaced_key(b"corrupt-doc-key", 0);
    let tx = col.allocate_tx()?;
    col.storage().put(tx, &storage_key, corrupt_payload).await?;
    col.storage().commit(tx).await?;

    // Strict policy must fail on load
    let result = col.load_index_with_report(LoadPolicy::Strict).await;
    assert!(
        result.is_err(),
        "Strict load policy must fail on corrupted document"
    );

    let err_msg = result.unwrap_err().to_string();
    let expected_hash = blake3::hash(&storage_key).to_hex()[..16].to_string();

    assert!(
        err_msg.contains(&expected_hash),
        "Error message should contain blake3 short hash {expected_hash}, got: {err_msg}"
    );
    assert!(
        !err_msg.contains("CORRUPTED_SECRET_RECORD_DATA_NOT_JSON"),
        "Error message must never leak record content"
    );

    // Default load_index() wrapper must also fail
    let wrapper_result = col.load_index().await;
    assert!(
        wrapper_result.is_err(),
        "load_index() wrapper must fail under Strict policy"
    );

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn index_load_lenient_counts_skips_and_fails_over_limit() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let col = db.collection("test_col").await?;

    // Insert 2 valid docs
    col.insert("valid-1", &[1.0, 0.0, 0.0, 0.0], None).await?;
    col.insert("valid-2", &[0.0, 1.0, 0.0, 0.0], None).await?;

    // Inject 1 corrupt doc
    let storage_key = col.namespaced_key(b"bad-doc", 0);
    let tx = col.allocate_tx()?;
    col.storage().put(tx, &storage_key, b"invalid json").await?;
    col.storage().commit(tx).await?;

    // Lenient with max_skipped = 0 must fail (1 skip > 0 limit)
    let res_limit_0 = col
        .load_index_with_report(LoadPolicy::Lenient { max_skipped: 0 })
        .await;
    assert!(
        res_limit_0.is_err(),
        "Lenient with max_skipped=0 must fail when 1 doc is corrupt"
    );

    // Lenient with max_skipped = 1 must succeed
    let report = col
        .load_index_with_report(LoadPolicy::Lenient { max_skipped: 1 })
        .await?;

    assert_eq!(report.scanned, 3);
    assert_eq!(report.decoded, 2);
    assert_eq!(report.skipped_undecodable, 1);
    assert_eq!(report.failed_inserts, 0);

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn index_load_report_counts_match() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let metrics_sink = Arc::new(TestMetricsSink::new());

    let config = ContextraConfig {
        dimension: 4,
        metrics_sink: Some(metrics_sink.clone()),
        ..Default::default()
    };

    let db = Contextra::open_with_config(dir.path(), config).await?;
    let col = db.collection("report_col").await?;
    col.set_metrics_sink(metrics_sink.clone());

    // Insert 3 valid docs
    col.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None).await?;
    col.insert("doc-2", &[0.0, 1.0, 0.0, 0.0], None).await?;
    col.insert("doc-3", &[0.0, 0.0, 1.0, 0.0], None).await?;

    // Inject 2 corrupt docs
    for i in 1..=2 {
        let key_name = format!("corrupt-{i}");
        let storage_key = col.namespaced_key(key_name.as_bytes(), 0);
        let tx = col.allocate_tx()?;
        col.storage().put(tx, &storage_key, b"{bad_json:").await?;
        col.storage().commit(tx).await?;
    }

    let report = col
        .load_index_with_report(LoadPolicy::Lenient { max_skipped: 10 })
        .await?;

    let expected_report = IndexLoadReport {
        scanned: 5,
        decoded: 3,
        skipped_undecodable: 2,
        failed_inserts: 0,
        took: report.took,
    };

    assert_eq!(report, expected_report);

    // Verify last_load_report() returns matching report
    let last_report = col.last_load_report();
    assert_eq!(last_report, Some(expected_report));

    // Verify metric event was emitted for skipped items
    let events = metrics_sink.events();
    let skipped_events: Vec<_> = events
        .iter()
        .filter(|e| match e {
            contextra_ports::MetricEvent::Counter { name, .. } => {
                name == "contextra_index_load_skipped_total"
            }
            _ => false,
        })
        .collect();

    assert_eq!(
        skipped_events.len(),
        2,
        "Expected 2 counter metric events for 2 skipped items"
    );

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn index_rebuild_is_deterministic() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config(dir.path(), config).await?;
    let col = db.collection("default").await?;

    // Insert corpus of documents
    col.insert(
        "doc-a",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"title": "A"})),
    )
    .await?;
    col.insert(
        "doc-b",
        &[0.0, 1.0, 0.0, 0.0],
        Some(serde_json::json!({"title": "B"})),
    )
    .await?;
    col.insert(
        "doc-c",
        &[0.5, 0.5, 0.0, 0.0],
        Some(serde_json::json!({"title": "C"})),
    )
    .await?;

    let query = vec![0.8, 0.2, 0.0, 0.0];

    // Search 1: Baseline
    let res_baseline = col.search(&query, 3).await?;
    assert_eq!(res_baseline.len(), 3);

    // Load 1
    let report1 = col.load_index_with_report(LoadPolicy::Strict).await?;
    assert_eq!(report1.scanned, 3);
    assert_eq!(report1.decoded, 3);
    assert_eq!(report1.skipped_undecodable, 0);
    assert_eq!(report1.failed_inserts, 0);

    let res_load1 = col.search(&query, 3).await?;
    assert_eq!(
        res_baseline, res_load1,
        "Search results must be identical after 1st load"
    );

    // Load 2
    let report2 = col.load_index_with_report(LoadPolicy::Strict).await?;
    assert_eq!(report2.scanned, 3);
    assert_eq!(report2.decoded, 3);

    let res_load2 = col.search(&query, 3).await?;
    assert_eq!(
        res_baseline, res_load2,
        "Search results must be identical after 2nd load"
    );

    db.close().await?;
    Ok(())
}
