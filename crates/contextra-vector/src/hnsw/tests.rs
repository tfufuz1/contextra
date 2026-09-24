use contextra_core::{ContextraError, DistanceMetric, DocId, TxId, VectorIndex};
use super::*;

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
