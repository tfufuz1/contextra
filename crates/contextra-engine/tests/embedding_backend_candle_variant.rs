use contextra_engine::{
    Contextra, ContextraConfig, DeletionLayer, EmbeddingBackend, LayerCleanupProof,
};
use std::path::PathBuf;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_candle_embedding_backend_nonexistent_dir_fails() -> TestResult {
    let tmp = tempfile::tempdir()?;
    let config = ContextraConfig {
        embedding_backend: EmbeddingBackend::Candle {
            model_dir: PathBuf::from("/nonexistent/candle/model/dir"),
        },
        ..Default::default()
    };

    let res = Contextra::open_with_config(tmp.path(), config).await;
    assert!(res.is_err(), "Expected error when candle model_dir does not exist");
    Ok(())
}

#[test]
fn test_stub_layer_cleanup_proof_failure_conditions() -> TestResult {
    let res_non_zero = LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 1);
    assert!(
        res_non_zero.is_err(),
        "new_after_verified_empty must fail when remaining_count != 0"
    );

    let res_false = LayerCleanupProof::verify_and_create(DeletionLayer::LsmMemtable, || Ok(false));
    assert!(
        res_false.is_err(),
        "verify_and_create must fail when verifier returns Ok(false)"
    );

    Ok(())
}
