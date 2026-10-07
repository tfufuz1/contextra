use super::*;
use contextra_core::{ContextraError, DistanceMetric, DocId, TxId, VectorIndex};
use contextra_ports::Rng;
use parking_lot::Mutex;

fn test_config(dim: usize) -> HnswConfig {
    HnswConfig {
        dimension: dim,
        max_elements: 10_000,
        m: 8,
        ef_construction: 100,
        ef_search: 64,

        distance_metric: DistanceMetric::Euclidean,
        rebuild_threshold: 0.8,
        quantize: false,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_hnsw_topology_and_search_determinism() {
    let dim = 8;
    let n = 100;
    let seed = 424242u64;

    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(64)
        .ef_search(32)
        .build()
        .expect("valid config");

    let rng1 = std::sync::Arc::new(contextra_ports::SeededRng::new(seed));
    let rng2 = std::sync::Arc::new(contextra_ports::SeededRng::new(seed));

    let idx1 = HnswIndex::try_new_with_rng(config.clone(), rng1).expect("idx1 creation");
    let idx2 = HnswIndex::try_new_with_rng(config, rng2).expect("idx2 creation");

    let sm = contextra_ports::SeededRng::new(1001);
    let mut vecs = Vec::with_capacity(n);
    for _ in 0..n {
        let v: Vec<f32> = (0..dim).map(|_| sm.next_unit_f64() as f32).collect();
        vecs.push(v);
    }

    let tx = TxId::new(1);
    for (i, v) in vecs.iter().enumerate() {
        let doc_id = DocId::from((i as u64 + 1) as u64);
        idx1.insert(tx, doc_id, v).await.expect("idx1 insert");
        idx2.insert(tx, doc_id, v).await.expect("idx2 insert");
    }
    idx1.commit(tx).await.expect("idx1 commit");
    idx2.commit(tx).await.expect("idx2 commit");

    // 1. Verify exact layer assignments (topology) across all documents
    let docs_and_layers1 = idx1.all_doc_ids_and_layers();
    let docs_and_layers2 = idx2.all_doc_ids_and_layers();

    assert_eq!(
        docs_and_layers1.len(),
        n,
        "index 1 doc count matches inserted batch"
    );
    assert_eq!(
        docs_and_layers1, docs_and_layers2,
        "HNSW layer assignments must be identical when seeded with identical RNG"
    );

    // 2. Verify search results are bitwise/identical
    let query: Vec<f32> = (0..dim).map(|_| sm.next_unit_f64() as f32).collect();
    let res1 = idx1.search(&query, 10).await.expect("idx1 search");
    let res2 = idx2.search(&query, 10).await.expect("idx2 search");

    assert_eq!(res1.len(), res2.len());
    for (r1, r2) in res1.iter().zip(res2.iter()) {
        assert_eq!(r1.doc_id, r2.doc_id);
        assert_eq!(
            r1.score.to_bits(),
            r2.score.to_bits(),
            "Search scores must match identically"
        );
    }
}

#[test]
fn test_try_new_invalid_config_fails_immediately() {
    let config = HnswConfig {
        ef_construction: 1,
        m: 100,
        ..test_config(4)
    };
    let result = HnswIndex::try_new(config);
    assert!(
        result.is_err(),
        "try_new must fail immediately on invalid config"
    );
    let err_msg = format!("{}", result.err().unwrap());
    assert!(
        err_msg.contains("ef_construction (1) must be >= m (100)"),
        "Unexpected error message: {}",
        err_msg
    );
}

#[test]
fn test_hnsw_config_builder_and_validation() {
    let config = HnswConfigBuilder::new(128)
        .max_elements(1000)
        .m(32)
        .ef_construction(128)
        .ef_search(64)
        .distance_metric(DistanceMetric::Cosine)
        .quantize(true)
        .quantizer_recalibration_sample_size(500)
        .build()
        .expect("valid builder config");

    assert_eq!(config.dimension, 128);
    assert_eq!(config.max_elements, 1000);
    assert_eq!(config.m, 32);
    assert_eq!(config.ef_construction, 128);
    assert_eq!(config.ef_search, 64);
    assert_eq!(config.distance_metric, DistanceMetric::Cosine);
    assert!(config.quantize);
    assert_eq!(config.quantizer_recalibration_sample_size, 500);

    let res_ef_c = HnswConfig {
        m: 16,
        ef_construction: 8,
        ..Default::default()
    }
    .validate();
    assert!(matches!(res_ef_c, Err(ContextraError::InvalidInput(_))));

    let res_builder_err = HnswConfigBuilder::new(128).m(16).ef_construction(8).build();
    assert!(matches!(
        res_builder_err,
        Err(ContextraError::InvalidInput(_))
    ));
}

#[tokio::test]
async fn test_compact_seq_log() {
    let config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config).expect("valid config");
    let tx = TxId::new(1);
    let doc_id = DocId::from(1u64);
    let vec = vec![1.0, 2.0, 3.0, 4.0];

    index.insert(tx, doc_id, &vec).await.expect("insert");
    index.commit(tx).await.expect("commit");

    index.compact_seq_log(10);
    assert_eq!(index.len().await, 1);
}

#[tokio::test]
async fn test_invalid_config_error() {
    let config = HnswConfig {
        ef_construction: 5,
        m: 10,
        ..test_config(4)
    };
    #[allow(deprecated)]
    let index = HnswIndex::new(config);
    let tx = TxId::new(1);
    let result = index
        .insert(tx, DocId::from(1u64), &[1.0, 0.0, 0.0, 0.0])
        .await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Invalid index configuration"));
    assert!(err_msg.contains("ef_construction (5) must be >= m (10)"));
}

#[tokio::test]
async fn test_insert_and_search() {
    let index = HnswIndex::try_new(test_config(4)).unwrap();
    let tx = TxId::new(1);

    index
        .insert(tx, DocId::from(1u64), &[1.0, 0.0, 0.0, 0.0])
        .await
        .expect("insert 1");
    index
        .insert(tx, DocId::from(2u64), &[0.0, 1.0, 0.0, 0.0])
        .await
        .expect("insert 2");
    index
        .insert(tx, DocId::from(3u64), &[0.9, 0.1, 0.0, 0.0])
        .await
        .expect("insert 3");
    index.commit(tx).await.expect("commit");

    let results = index
        .search(&[1.0, 0.0, 0.0, 0.0], 2)
        .await
        .expect("search");
    assert!(!results.is_empty());
    assert_eq!(results[0].doc_id, DocId::from(1u64));
}

#[tokio::test]
async fn test_delete() {
    let index = HnswIndex::try_new(test_config(4)).unwrap();

    let tx1 = TxId::new(1);
    index
        .insert(tx1, DocId::from(1u64), &[1.0, 0.0, 0.0, 0.0])
        .await
        .expect("insert");
    index.commit(tx1).await.expect("commit");

    assert_eq!(index.len().await, 1);

    let tx2 = TxId::new(2);
    index.delete(tx2, DocId::from(1u64)).await.expect("delete");
    index.commit(tx2).await.expect("commit");

    assert_eq!(index.len().await, 0);
}

#[derive(Default)]
struct TestMetricsSink {
    counters: Mutex<Vec<(String, u64, Vec<(String, String)>)>>,
    histograms: Mutex<Vec<(String, f64, Vec<(String, String)>)>>,
}

impl contextra_ports::MetricsSink for TestMetricsSink {
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        let label_vec = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.counters
            .lock()
            .push((name.to_string(), value, label_vec));
    }

    fn record_gauge(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}

    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let label_vec = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.histograms
            .lock()
            .push((name.to_string(), value, label_vec));
    }
}

#[tokio::test]
async fn test_hnsw_rebuild_metrics_duration() {
    let sink = std::sync::Arc::new(TestMetricsSink::default());
    let config = test_config(4);
    let index = HnswIndex::try_new(config)
        .expect("valid config")
        .with_metrics_sink(sink.clone());

    let tx = TxId::new(1);
    index
        .insert(tx, DocId::from(1u64), &[1.0, 0.0, 0.0, 0.0])
        .await
        .expect("insert");
    index.commit(tx).await.expect("commit");

    index.rebuild().await.expect("rebuild must succeed");

    let hists = sink.histograms.lock();
    assert!(
        hists
            .iter()
            .any(|(name, _, _)| name == "vector_rebuild_duration_seconds"),
        "vector_rebuild_duration_seconds histogram must be recorded"
    );

    let cnts = sink.counters.lock();
    assert!(
        cnts.iter()
            .any(|(name, val, labels)| name == "vector_rebuild_total"
                && *val == 1
                && labels.contains(&("status".to_string(), "success".to_string()))),
        "vector_rebuild_total success counter must be recorded"
    );
}

#[tokio::test]
async fn test_hnsw_rebuild_exponential_backoff_and_alarm() {
    let sink = std::sync::Arc::new(TestMetricsSink::default());
    let config = HnswConfigBuilder::new(4)
        .rebuild_threshold(0.9)
        .backoff_base_delay(std::time::Duration::from_millis(200))
        .backoff_max_delay(std::time::Duration::from_secs(2))
        .backoff_alert_threshold(2)
        .build()
        .expect("valid config");

    let index = HnswIndex::try_new(config)
        .expect("valid index")
        .with_metrics_sink(sink.clone());

    let tx1 = TxId::new(1);
    index
        .insert(tx1, DocId::from(1u64), &[1.0, 0.0, 0.0, 0.0])
        .await
        .expect("insert 1");
    index
        .insert(tx1, DocId::from(2u64), &[0.0, 1.0, 0.0, 0.0])
        .await
        .expect("insert 2");
    index.commit(tx1).await.expect("commit 1");

    // Artificially induce high deletion rate (> 10%)
    let tx2 = TxId::new(2);
    index
        .delete(tx2, DocId::from(1u64))
        .await
        .expect("delete 1");
    index.commit(tx2).await.expect("commit 2");

    assert!(
        index.is_rebuild_required(),
        "rebuild must be required due to high deletion ratio"
    );

    // First rebuild attempt -> succeeds
    index.rebuild().await.expect("first rebuild must succeed");

    // Second rapid rebuild attempt -> suppressed by backoff cooldown
    let res2 = index.rebuild().await;
    assert!(res2.is_err(), "rapid rebuild must fail due to backoff");
    let err_msg2 = format!("{}", res2.unwrap_err());
    assert!(err_msg2.contains("Rebuild rate limited by backoff cooldown"));

    // Third rapid rebuild attempt -> suppressed and triggers alert threshold alarm
    let res3 = index.rebuild().await;
    assert!(
        res3.is_err(),
        "subsequent rapid rebuild must fail due to backoff"
    );

    let cnts = sink.counters.lock();
    assert!(
        cnts.iter()
            .any(|(name, _, _)| name == "vector_rebuild_backoff_suppressed_total"),
        "vector_rebuild_backoff_suppressed_total must be incremented"
    );
    assert!(
        cnts.iter()
            .any(|(name, _, labels)| name == "vector_rebuild_alarm_total"
                && labels
                    .contains(&("reason".to_string(), "excessive_rapid_rebuilds".to_string()))),
        "vector_rebuild_alarm_total alarm metric must be recorded"
    );

    drop(cnts);

    // Wait for exponential backoff cooldown (800ms) to expire
    tokio::time::sleep(std::time::Duration::from_millis(900)).await;

    // Rebuild after cooldown expires -> succeeds
    index
        .rebuild()
        .await
        .expect("rebuild after cooldown must succeed");
}
