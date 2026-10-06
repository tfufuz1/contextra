// FILE-CONTEXT
// STAND: 2026-10-06T00:00:00Z
// ZWECK: Integration test for contextra-infer-onnx symbol verification and removal closure (J29).

use contextra_infer_onnx::{
    default_model_cache_dir, CrossEncoderReranker, RerankConfig,
};

#[test]
fn test_onnx_crate_exports_and_removal_verification() {
    let cache_dir = default_model_cache_dir("nomic-embed-text");
    assert!(cache_dir.to_string_lossy().contains("nomic-embed-text"));

    #[cfg(feature = "onnx")]
    {
        let config = contextra_infer_onnx::TextEmbedderConfig::default();
        assert_eq!(config.max_sequence_length, 512);
    }

    let reranker_config = RerankConfig::default();
    let reranker = CrossEncoderReranker::passthrough_with_config(reranker_config);
    assert!(!reranker.is_calibrated());
}
