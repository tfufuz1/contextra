use contextra_engine::metrics_names::{COMMIT_LATENCY, REBUILD_DURATION};
use contextra_engine::{Contextra, ContextraConfig};
use contextra_ports::{MetricEvent, MetricsSink, TestMetricsSink};
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
