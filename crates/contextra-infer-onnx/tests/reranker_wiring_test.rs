use contextra_infer_onnx::wiring::{RerankWiring, RerankWiringDeps};
use contextra_infer_onnx::RerankConfig;
use contextra_types::ContextraError;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[tokio::test]
async fn test_rerank_wiring_disabled_returns_passthrough() {
    let config = RerankConfig::default();
    let deps = RerankWiringDeps {
        config,
        enabled: false,
    };

    let wiring = RerankWiring::build(deps).expect("Wiring build should succeed when disabled");
    assert!(!wiring.enabled());

    let reranker = wiring.reranker();
    let candidates = vec!["doc1".to_string(), "doc2".to_string()];
    let res = reranker.rerank("query", &candidates).await.unwrap();
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].original_index, 0);
    assert_eq!(res[1].original_index, 1);
}

#[test]
fn test_rerank_wiring_enabled_missing_model_fails() {
    let mut config = RerankConfig::default();
    config.model_path = std::path::PathBuf::from("non_existent_model.onnx");
    config.tokenizer_path = std::path::PathBuf::from("non_existent_tokenizer.json");

    let deps = RerankWiringDeps {
        config,
        enabled: true,
    };

    let res = RerankWiring::build(deps);
    assert!(
        res.is_err(),
        "Wiring build must fail on missing model when enabled"
    );
    let err = res.err().unwrap();
    assert!(
        matches!(
            err,
            ContextraError::InvalidInput(_) | ContextraError::NotFound(_)
        ),
        "Expected InvalidInput or NotFound error, got {:?}",
        err
    );
}

#[test]
fn test_rerank_wiring_enabled_with_fake_model_or_passthrough(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let model_path = dir.path().join("model.onnx");
    let tokenizer_path = dir.path().join("tokenizer.json");

    File::create(&model_path)?.write_all(b"fake_onnx")?;
    File::create(&tokenizer_path)?.write_all(b"{}")?;

    let mut config = RerankConfig::default();
    config.model_path = model_path;
    config.tokenizer_path = tokenizer_path;

    let deps = RerankWiringDeps {
        config,
        enabled: true,
    };

    let res = RerankWiring::build(deps);
    if cfg!(feature = "onnx") {
        assert!(res.is_err());
    } else {
        assert!(res.is_ok());
    }

    Ok(())
}
