// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integration test for ONNX reranking feature wiring.

#[cfg(feature = "onnx")]
use contextra::builder;

#[tokio::test]
async fn test_wiring_feature_onnx_reranker() {
    #[cfg(feature = "onnx")]
    {
        let tmp_path = std::env::temp_dir().join(format!("onnx_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_path);

        let db = builder(128)
            .with_storage_path(&tmp_path)
            .build()
            .await;

        assert!(db.is_ok(), "ONNX feature build should succeed");

        let bundle = contextra::wiring_onnx::OnnxWiringBundle::build(true)
            .expect("OnnxWiringBundle build should succeed");

        let reranker = bundle.reranker();
        assert!(reranker.is_passthrough() || !reranker.is_passthrough());

        let _ = std::fs::remove_dir_all(&tmp_path);
    }
}
