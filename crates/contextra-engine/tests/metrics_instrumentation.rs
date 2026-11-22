use contextra_engine::metrics_names::{
    COMMIT_LATENCY, INSERT_LATENCY, REBUILD_DURATION, SEARCH_LATENCY_TEXT, SEARCH_LATENCY_VECTOR,
};
use contextra_engine::{Contextra, ContextraConfig};
use contextra_ports::MetricsSink;
use contextra_testkit::{MetricEvent, TestMetricsSink};
use std::sync::Arc;
use tempfile::tempdir;

struct PanickingMetricsSink;

impl MetricsSink for PanickingMetricsSink {
    fn record_counter(&self, _name: &str, _value: u64, _labels: &[(&str, &str)]) {
        panic!("PanickingMetricsSink triggered in counter");
    }

    fn record_gauge(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {
        panic!("PanickingMetricsSink triggered in gauge");
    }

    fn record_histogram(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {
        panic!("PanickingMetricsSink triggered in histogram");
    }
}

#[tokio::test]
async fn test_insert_emits_commit_latency_metric() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let test_sink = Arc::new(TestMetricsSink::new());

    let config = ContextraConfig {
        dimension: 4,
        metrics_sink: Some(test_sink.clone() as Arc<dyn MetricsSink>),
        ..Default::default()
    };

    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("test_metrics").await?;

    let doc_id = "doc-1";
    let vector = vec![0.1, 0.2, 0.3, 0.4];
    let meta = serde_json::json!({"text": "hello observability"});

    collection.insert(doc_id, &vector, Some(meta)).await?;

    let events = test_sink.events();
    let commit_latency_event = events.iter().find(|e| match e {
        MetricEvent::Histogram { name, .. } => name == COMMIT_LATENCY,
        _ => false,
    });

    assert!(
        commit_latency_event.is_some(),
        "Expected commit.latency histogram metric to be recorded on insert"
    );

    if let Some(MetricEvent::Histogram { value, labels, .. }) = commit_latency_event {
        assert!(*value >= 0.0, "Latency value should be non-negative");
        assert!(
            labels.iter().any(|(k, v)| k == "status" && v == "success"),
            "Expected status=success label"
        );
    }

    let insert_latency_event = events.iter().find(|e| match e {
        MetricEvent::Histogram { name, .. } => name == INSERT_LATENCY,
        _ => false,
    });
    assert!(
        insert_latency_event.is_some(),
        "Expected insert.latency histogram metric to be recorded on insert"
    );

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_search_emits_signal_latency_metrics_without_pii() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let test_sink = Arc::new(TestMetricsSink::new());

    let config = ContextraConfig {
        dimension: 4,
        metrics_sink: Some(test_sink.clone() as Arc<dyn MetricsSink>),
        ..Default::default()
    };

    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("test_search_metrics").await?;

    let secret_prompt = "TOP_SECRET_PROMPT_DO_NOT_LEAK";
    let vector = vec![0.1, 0.2, 0.3, 0.4];

    collection.insert("doc-secret", &vector, Some(serde_json::json!({"text": secret_prompt}))).await?;

    let _res = collection
        .query()
        .text(secret_prompt)
        .vector(&vector)
        .k(5)
        .execute()
        .await?;

    let events = test_sink.events();

    let has_vec_lat = events.iter().any(|e| match e {
        MetricEvent::Histogram { name, .. } => name == SEARCH_LATENCY_VECTOR,
        _ => false,
    });
    assert!(has_vec_lat, "Expected search.latency.vector metric");

    let has_text_lat = events.iter().any(|e| match e {
        MetricEvent::Histogram { name, .. } => name == SEARCH_LATENCY_TEXT,
        _ => false,
    });
    assert!(has_text_lat, "Expected search.latency.text metric");

    // Verify no secret prompt or document content appears in any event labels
    for event in &events {
        let labels = match event {
            MetricEvent::Counter { labels, .. }
            | MetricEvent::Gauge { labels, .. }
            | MetricEvent::Histogram { labels, .. } => labels,
        };
        for (k, v) in labels {
            assert!(
                !k.contains(secret_prompt) && !v.contains(secret_prompt),
                "Label ({k}={v}) contains secret prompt content!"
            );
        }
    }

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_repair_emits_rebuild_duration_metric() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let test_sink = Arc::new(TestMetricsSink::new());

    let config = ContextraConfig {
        dimension: 4,
        metrics_sink: Some(test_sink.clone() as Arc<dyn MetricsSink>),
        ..Default::default()
    };

    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("test_repair_metrics").await?;

    collection.repair().await?;

    let events = test_sink.events();
    let rebuild_event = events.iter().find(|e| match e {
        MetricEvent::Histogram { name, .. } => name == REBUILD_DURATION,
        _ => false,
    });

    assert!(
        rebuild_event.is_some(),
        "Expected rebuild.duration histogram metric to be recorded on repair"
    );

    if let Some(MetricEvent::Histogram { value, labels, .. }) = rebuild_event {
        assert!(
            *value >= 0.0,
            "Rebuild duration value should be non-negative"
        );
        assert!(
            labels.iter().any(|(k, v)| k == "status" && v == "success"),
            "Expected status=success label"
        );
    }

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_default_noop_metrics_sink_runs_without_errors() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default() // metrics_sink = None (Noop)
    };

    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("test_noop").await?;

    let vector = vec![1.0, 0.0, 0.0, 0.0];
    collection.insert("doc-noop", &vector, None).await?;
    collection.repair().await?;

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_panicking_metrics_sink_does_not_fail_commit() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let panicking_sink = Arc::new(PanickingMetricsSink);

    let config = ContextraConfig {
        dimension: 4,
        metrics_sink: Some(panicking_sink as Arc<dyn MetricsSink>),
        ..Default::default()
    };

    let db = Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("test_panicking_sink").await?;

    let vector = vec![0.5, 0.5, 0.5, 0.5];
    let res = collection.insert("doc-panic-test", &vector, None).await;

    assert!(
        res.is_ok(),
        "Transaction commit must succeed best-effort even if MetricsSink panics"
    );

    let fetched = collection.get("doc-panic-test").await?;
    assert!(fetched.is_some(), "Document must be readable after commit");

    db.close().await?;
    Ok(())
}
