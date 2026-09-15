use memfuse_core::{DistanceMetric, DocId, MemFuseError, TxId, VectorIndex};
use memfuse_index::distance::compute_distance;
use memfuse_index::hnsw::HnswIndex;
use memfuse_index::HnswConfig;

#[tokio::test]
async fn proof_nan_rejected_at_insert_not_at_search() {
    let mut config = HnswConfig::default();
    config.dimension = 4;
    let index = HnswIndex::try_new(config).expect("failed to create hnsw index");
    let tx = TxId::new(1);

    // Vector with NaN must be rejected at insert
    let nan_vec = vec![1.0, f32::NAN, 0.0, 0.0];
    let res_nan = index.insert(tx, DocId::new(1), &nan_vec).await;
    assert!(
        matches!(res_nan, Err(MemFuseError::InvalidInput(_))),
        "Expected InvalidInput for NaN vector insert, got: {:?}",
        res_nan
    );

    // Vector with Inf must be rejected at insert
    let inf_vec = vec![1.0, f32::INFINITY, 0.0, 0.0];
    let res_inf = index.insert(tx, DocId::new(2), &inf_vec).await;
    assert!(
        matches!(res_inf, Err(MemFuseError::InvalidInput(_))),
        "Expected InvalidInput for Inf vector insert, got: {:?}",
        res_inf
    );
}

#[test]
fn proof_nan_scan_not_in_hot_path() {
    let a = vec![1.0, f32::NAN, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0, 0.0];

    // After removing the O(2D) NaN scan from compute_distance,
    // compute_distance no longer inspects each element for NaN and returns Ok(_).
    let res = compute_distance(&a, &b, DistanceMetric::Cosine);
    assert!(
        res.is_ok(),
        "compute_distance in hot-path should trust input and not return Err on NaN check, got: {:?}",
        res
    );
}
